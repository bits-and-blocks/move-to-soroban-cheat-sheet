#![cfg(test)]
extern crate std;
use soroban_sdk::{contract, contractimpl, Address, Env};
use soroban_sdk::testutils::Address as _;

// §5.2 claim: contractimport! generates a typed Client from a wasm file
mod dep_contract {
    soroban_sdk::contractimport!(file = "wasm/dep.wasm");
}

#[contract]
pub struct Caller;

#[contractimpl]
impl Caller {
    pub fn call_dep(env: Env, target: Address) -> u64 {
        dep_contract::Client::new(&env, &target).add(&1u64, &2u64)
    }
}

#[test]
fn import_and_register_real_wasm() {
    let env = Env::default();
    // §9 claim: env.register(other::WASM, args) registers the actual binary
    let dep = env.register(dep_contract::WASM, ());
    let caller = env.register(Caller, ());
    let c = CallerClient::new(&env, &caller);
    std::println!("CROSS-CONTRACT RESULT: {}", c.call_dep(&dep));
    let _ = Address::generate(&env);
}
