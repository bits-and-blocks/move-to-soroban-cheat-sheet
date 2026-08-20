#![cfg(test)]
extern crate std;

use soroban_sdk::testutils::{
    Address as _, AuthorizedFunction, AuthorizedInvocation, Events as _, Ledger as _,
    MockAuth, MockAuthInvoke,
};
use soroban_sdk::testutils::storage::{Instance as _, Persistent as _};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{vec, Address, Env, IntoVal, Symbol};

use verify::{Category, DataKey, Error, ZakatPool, ZakatPoolClient};

fn setup() -> (Env, Address, ZakatPoolClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let gov = Address::generate(&env);
    let dist = Address::generate(&env);
    // §9 register with __constructor args as a tuple
    let id = env.register(ZakatPool, (gov.clone(), dist.clone()));
    let client = ZakatPoolClient::new(&env, &id);
    (env, id, client, gov, dist)
}

#[test]
fn register_with_constructor_args() {
    let (_env, _id, _client, _g, _d) = setup();
}

#[test]
fn ledger_set_timestamp_and_sequence() {
    let (env, _id, _c, _g, _d) = setup();
    env.ledger().set_timestamp(1_700_000_000);
    env.ledger().set_sequence_number(5_000);
    assert_eq!(env.ledger().timestamp(), 1_700_000_000);
    assert_eq!(env.ledger().sequence(), 5_000);
}

#[test]
fn sac_registration_and_transfer() {
    let (env, id, client, _g, _d) = setup();
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer.clone());
    let asset = sac.address();
    let donor = Address::generate(&env);

    StellarAssetClient::new(&env, &asset).mint(&donor, &10_000);

    // approve the asset via direct storage write
    env.as_contract(&id, || {
        env.storage()
            .instance()
            .set(&DataKey::ApprovedAssets, &vec![&env, asset.clone()]);
    });

    client.contribute(&donor, &asset, &1_000);

    assert_eq!(TokenClient::new(&env, &asset).balance(&id), 1_000);
    assert_eq!(client.bucket_balance(&asset, &Category::Fuqara), 1_000);
}

#[test]
fn auths_shape_and_reset() {
    let (env, id, client, _g, _d) = setup();
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let asset = sac.address();
    let donor = Address::generate(&env);
    StellarAssetClient::new(&env, &asset).mint(&donor, &10_000);
    env.as_contract(&id, || {
        env.storage()
            .instance()
            .set(&DataKey::ApprovedAssets, &vec![&env, asset.clone()]);
    });

    client.contribute(&donor, &asset, &1_000);

    // §9: env.auths() -> Vec<(Address, AuthorizedInvocation)>
    let auths = env.auths();
    assert_eq!(auths[0].0, donor);
    let invocation: &AuthorizedInvocation = &auths[0].1;
    match &invocation.function {
        AuthorizedFunction::Contract((addr, fn_name, args)) => {
            assert_eq!(addr, &id);
            assert_eq!(fn_name, &Symbol::new(&env, "contribute"));
            assert_eq!(
                args,
                &vec![
                    &env,
                    donor.to_val(),
                    asset.to_val(),
                    1_000i128.into_val(&env)
                ]
            );
        }
        _ => panic!("expected contract fn"),
    }
    // the nested SAC transfer is a sub_invocation of the root
    assert_eq!(invocation.sub_invocations.len(), 1);

    // §9 TRAP claim: does auths() reset on the next client call, including a read?
    let _ = client.bucket_balance(&asset, &Category::Fuqara);
    std::println!("AUTHS AFTER SUBSEQUENT READ: {}", env.auths().len());
}

#[test]
fn events_all_shape_and_reset() {
    let (env, id, client, _g, _d) = setup();
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let asset = sac.address();
    let donor = Address::generate(&env);
    StellarAssetClient::new(&env, &asset).mint(&donor, &10_000);
    env.as_contract(&id, || {
        env.storage()
            .instance()
            .set(&DataKey::ApprovedAssets, &vec![&env, asset.clone()]);
    });

    client.contribute(&donor, &asset, &1_000);
    let evs = env.events().all();
    std::println!("EVENTS AFTER CONTRIBUTE: {:?}", evs);

    let _ = client.bucket_balance(&asset, &Category::Fuqara);
    std::println!("EVENTS AFTER SUBSEQUENT READ: {:?}", env.events().all());
}

#[test]
fn try_result_nesting() {
    let (env, _id, client, _g, _d) = setup();
    let donor = Address::generate(&env);
    let asset = Address::generate(&env);

    // Exact SDK 27 shape for a fn returning Result<(), Error>:
    let r: Result<Result<(), soroban_sdk::ConversionError>, Result<Error, soroban_sdk::InvokeError>> =
        client.try_contribute(&donor, &asset, &0);
    match r {
        Err(Ok(e)) => std::println!("TYPED CONTRACT ERROR AT Err(Ok(_)): {:?}", e),
        Err(Err(ie)) => std::println!("HOST ERROR AT Err(Err(_)): {:?}", ie),
        Ok(Ok(())) => std::println!("SUCCESS"),
        Ok(Err(ref ce)) => std::println!("CONVERSION: {:?}", ce),
    }
    assert_eq!(r, Err(Ok(Error::InvalidAmount)));

    // And for a fn that does NOT return Result (bucket_balance -> i128):
    let r2: Result<Result<i128, soroban_sdk::Error>, Result<soroban_sdk::Error, soroban_sdk::InvokeError>> =
        client.try_bucket_balance(&asset, &Category::Fuqara);
    std::println!("NON-RESULT FN try_ SHAPE: {:?}", r2);
}

#[test]
fn ttl_reads_and_extend() {
    let (env, id, _client, _g, _d) = setup();
    let asset = Address::generate(&env);
    env.as_contract(&id, || {
        env.storage()
            .persistent()
            .set(&DataKey::Buckets(asset.clone()), &0i128);
        let ttl = env
            .storage()
            .persistent()
            .get_ttl(&DataKey::Buckets(asset.clone()));
        std::println!("PERSISTENT TTL: {}", ttl);
        std::println!("INSTANCE TTL: {}", env.storage().instance().get_ttl());
    });
}

#[test]
fn cost_estimate_surface() {
    let (env, _id, _c, _g, _d) = setup();
    let r = env.cost_estimate().resources();
    std::println!("RESOURCES: {:?}", r);
    let b = env.cost_estimate().budget();
    std::println!("BUDGET CPU: {:?}", b.cpu_instruction_cost());
}

#[test]
fn mock_auths_specific() {
    let (env, id, client, _g, _d) = setup();
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let asset = sac.address();
    let donor = Address::generate(&env);
    StellarAssetClient::new(&env, &asset).mint(&donor, &10_000);
    env.as_contract(&id, || {
        env.storage()
            .instance()
            .set(&DataKey::ApprovedAssets, &vec![&env, asset.clone()]);
    });

    // §9 mock_auths with an explicit tree
    env.mock_auths(&[MockAuth {
        address: &donor,
        invoke: &MockAuthInvoke {
            contract: &id,
            fn_name: "contribute",
            args: (donor.clone(), asset.clone(), 1_000i128).into_val(&env),
            sub_invokes: &[MockAuthInvoke {
                contract: &asset,
                fn_name: "transfer",
                args: (donor.clone(), id.clone(), 1_000i128).into_val(&env),
                sub_invokes: &[],
            }],
        },
    }]);
    client.contribute(&donor, &asset, &1_000);
}
