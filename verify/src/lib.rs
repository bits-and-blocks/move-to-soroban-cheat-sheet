#![cfg_attr(not(test), no_std)]
use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, panic_with_error,
    symbol_short, token, vec, Address, Env, IntoVal, Map, Symbol, Vec,
};

// ---------- §6.1 errors ----------
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    InsufficientBalance = 2,
    InvalidAmount = 3,
    Paused = 4,
    AssetNotApproved = 5,
    Overflow = 6,
    Insolvent = 7,
}

// ---------- §3.1 DataKey ----------
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Governance,
    Distributor,
    PolicyAddr,
    AttestAddr,
    Paused,
    ApprovedAssets,
    Buckets(Address),
}

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Fuqara,
    Masakin,
    Riqab,
}

#[contracttype]
#[derive(Clone, Default)]
pub struct BucketSet {
    pub fuqara: i128,
    pub masakin: i128,
    pub riqab: i128,
}

impl BucketSet {
    pub fn total(&self, env: &Env) -> i128 {
        let mut t: i128 = 0;
        for b in [self.fuqara, self.masakin, self.riqab] {
            t = match t.checked_add(b) {
                Some(v) => v,
                None => panic_with_error!(env, Error::Overflow),
            };
        }
        t
    }
    pub fn add(&mut self, env: &Env, _c: Category, amount: i128) {
        self.fuqara = match self.fuqara.checked_add(amount) {
            Some(v) => v,
            None => panic_with_error!(env, Error::Overflow),
        };
    }
    pub fn get(&self, _c: Category) -> i128 {
        self.fuqara
    }
}

// ---------- §7 events ----------
#[contractevent]
pub struct Contribution {
    #[topic]
    pub donor: Address,
    pub asset: Address,
    pub amount: i128,
    pub split: BucketSet,
}

#[contract]
pub struct ZakatPool;

#[contractimpl]
impl ZakatPool {
    // §5.4 constructor
    pub fn __constructor(env: Env, governance: Address, distributor: Address) {
        env.storage().instance().set(&DataKey::Governance, &governance);
        env.storage().instance().set(&DataKey::Distributor, &distributor);
    }

    // §1.1 auth + §4.2 token transfer
    pub fn contribute(env: Env, from: Address, asset: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if env
            .storage()
            .instance()
            .get::<_, bool>(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(Error::Paused);
        }
        let assets: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::ApprovedAssets)
            .unwrap_or(vec![&env]);
        if !assets.contains(&asset) {
            return Err(Error::AssetNotApproved);
        }

        token::TokenClient::new(&env, &asset).transfer(
            &from,
            &env.current_contract_address(),
            &amount,
        );

        // §3.3 update, not get+mutate
        env.storage()
            .persistent()
            .update(&DataKey::Buckets(asset.clone()), |cur: Option<BucketSet>| {
                let mut b = cur.unwrap_or_default();
                b.add(&env, Category::Fuqara, amount);
                b
            });

        Contribution {
            donor: from,
            asset: asset.clone(),
            amount,
            split: BucketSet::default(),
        }
        .publish(&env);

        assert_solvent(&env, &asset);
        Ok(())
    }

    // §1.1 narrowed auth
    pub fn narrowed(env: Env, from: Address, asset: Address, amount: i128) {
        from.require_auth_for_args((asset, amount).into_val(&env));
    }

    // §5.7 pure getter
    pub fn bucket_balance(env: Env, asset: Address, category: Category) -> i128 {
        env.storage()
            .persistent()
            .get::<_, BucketSet>(&DataKey::Buckets(asset))
            .unwrap_or_default()
            .get(category)
    }

    // §4.1 solvency view
    pub fn solvency(env: Env, asset: Address) -> (i128, i128) {
        let held = token::TokenClient::new(&env, &asset).balance(&env.current_contract_address());
        let booked = env
            .storage()
            .persistent()
            .get::<_, BucketSet>(&DataKey::Buckets(asset))
            .unwrap_or_default()
            .total(&env);
        (booked, held)
    }

    // §3.5 TTL
    pub fn bump(env: Env, asset: Address) {
        const DAY: u32 = 17_280;
        env.storage().instance().extend_ttl(30 * DAY, 120 * DAY);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Buckets(asset), 30 * DAY, 120 * DAY);
    }

    // §5.5 upgrade
    pub fn upgrade(env: Env, new_wasm_hash: soroban_sdk::BytesN<32>) {
        let admin: Address = env.storage().instance().get(&DataKey::Governance).unwrap();
        admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    // §6.5 env quick hits
    pub fn envstuff(env: Env) -> (u64, u32, soroban_sdk::BytesN<32>) {
        (
            env.ledger().timestamp(),
            env.ledger().sequence(),
            env.ledger().network_id(),
        )
    }

    // §7 symbols
    pub fn sym() -> Symbol {
        symbol_short!("transfer")
    }

    // §8 SAC admin client
    pub fn sac_admin(env: Env, asset: Address, to: Address, amount: i128) {
        token::StellarAssetClient::new(&env, &asset).mint(&to, &amount);
    }

    // §4.3 map usage
    pub fn mapping(env: Env) -> Map<u32, i128> {
        Map::new(&env)
    }
}

// §2.1 role auth (private helper, not an entry point)
fn _require_distributor(env: &Env) {
    let d: Address = env.storage().instance().get(&DataKey::Distributor).unwrap();
    d.require_auth();
}

fn assert_solvent(env: &Env, asset: &Address) {
    let held: i128 = token::TokenClient::new(env, asset).balance(&env.current_contract_address());
    let booked: i128 = env
        .storage()
        .persistent()
        .get::<_, BucketSet>(&DataKey::Buckets(asset.clone()))
        .unwrap_or_default()
        .total(env);
    if booked > held {
        panic_with_error!(env, Error::Insolvent);
    }
}

pub mod advanced;
