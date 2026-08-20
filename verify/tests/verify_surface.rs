#![cfg(test)]
extern crate std;

use soroban_sdk::testutils::arbitrary::SorobanArbitrary;
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{vec, Address, Bytes, Env, Event as _, IntoVal, Symbol, TryFromVal};

use verify::{BucketSet, Category, Contribution, DataKey, ZakatPool, ZakatPoolClient};

#[test]
fn extend_ttl_with_limits_exists() {
    let env = Env::default();
    let id = env.register(
        ZakatPool,
        (Address::generate(&env), Address::generate(&env)),
    );
    let asset = Address::generate(&env);
    env.as_contract(&id, || {
        env.storage()
            .persistent()
            .set(&DataKey::Buckets(asset.clone()), &0i128);
        // §3.5 claim: extend_ttl_with_limits caps caller-forced rent
        env.storage().persistent().extend_ttl_with_limits(
            &DataKey::Buckets(asset.clone()),
            100,
            1000,
            2000,
        );
        env.storage().instance().extend_ttl_with_limits(100, 1000, 2000);
    });
}

#[test]
fn muxed_address_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let asset = sac.address();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    StellarAssetClient::new(&env, &asset).mint(&a, &1_000);

    // §8 claim: transfer's destination is a MuxedAddress that a plain Address converts into
    let muxed: soroban_sdk::MuxedAddress = b.clone().into();
    TokenClient::new(&env, &asset).transfer(&a, &muxed, &100);
    assert_eq!(TokenClient::new(&env, &asset).balance(&b), 100);
}

#[test]
fn event_to_xdr_for_assertion() {
    let env = Env::default();
    env.mock_all_auths();
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let asset = sac.address();
    let donor = Address::generate(&env);
    StellarAssetClient::new(&env, &asset).mint(&donor, &10_000);
    let id = env.register(
        ZakatPool,
        (Address::generate(&env), Address::generate(&env)),
    );
    env.as_contract(&id, || {
        env.storage()
            .instance()
            .set(&DataKey::ApprovedAssets, &vec![&env, asset.clone()]);
    });
    let client = ZakatPoolClient::new(&env, &id);
    client.contribute(&donor, &asset, &1_000);

    // §7.1 claim: assert an event by comparing to e.to_xdr(&env, &contract_id)
    let expected = Contribution {
        donor: donor.clone(),
        asset: asset.clone(),
        amount: 1_000,
        split: BucketSet::default(),
    };
    let expected_xdr = expected.to_xdr(&env, &id);
    // ContractEvents impls PartialEq<std::vec::Vec<xdr::ContractEvent>>
    assert_eq!(
        env.events().all().filter_by_contract(&id),
        std::vec![expected_xdr]
    );
    // .events() gives the raw slice
    std::println!("RAW EVENT COUNT: {}", env.events().all().events().len());
}

#[test]
fn prng_and_crypto_surface() {
    let env = Env::default();
    let id = env.register(
        ZakatPool,
        (Address::generate(&env), Address::generate(&env)),
    );
    env.as_contract(&id, || {
        // §6.5 claim: env.prng() exists and is ledger-seeded
        let n: u64 = env.prng().gen();
        std::println!("PRNG: {}", n);
        // §1.4 claim: sha256 over canonical xdr
        let b = Bytes::from_array(&env, &[1u8; 8]);
        let _h = env.crypto().sha256(&b);
        // §4.4 claim: contracttype structs get .to_xdr(&env)
        let bs = BucketSet::default();
        let _x: Bytes = bs.to_xdr(&env);
    });
}

#[test]
fn fuzz_arbitrary_trait() {
    // §9 claim: #[contracttype] types implement SorobanArbitrary
    fn assert_arb<T: SorobanArbitrary>() {}
    assert_arb::<BucketSet>();
    assert_arb::<Category>();
    assert_arb::<DataKey>();
    assert_arb::<i128>();

    // and the prototype round-trips back into the SDK type
    let env = Env::default();
    let mut u = arbitrary::Unstructured::new(&[7u8; 256]);
    let proto: <BucketSet as SorobanArbitrary>::Prototype =
        arbitrary::Arbitrary::arbitrary(&mut u).unwrap();
    let v = BucketSet::try_from_val(&env, &proto);
    std::println!("ARBITRARY ROUNDTRIP OK: {}", v.is_ok());
}

#[test]
fn mock_variants_and_limits() {
    let env = Env::default();
    // §9 claims
    env.mock_all_auths_allowing_non_root_auth();
    env.cost_estimate().budget().reset_unlimited();
    let _ = Symbol::new(&env, "a_symbol_longer_than_nine");
    std::println!("MOCK VARIANTS OK");
}

#[test]
fn symbol_short_limit() {
    let env = Env::default();
    // §4.4 claim: Symbol::new allows up to 32; symbol_short! up to 9
    let s = Symbol::new(&env, "abcdefghijklmnopqrstuvwxyz123456");
    let v: soroban_sdk::Val = s.into_val(&env);
    std::println!("32-CHAR SYMBOL OK: {:?}", v.get_payload() != 0);
}
