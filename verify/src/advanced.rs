use soroban_sdk::auth::{
    Context, ContractContext, CustomAccountInterface, InvokerContractAuthEntry,
    SubContractInvocation,
};
use soroban_sdk::{
    contract, contractimpl, symbol_short, vec, Address, BytesN, Env, IntoVal, Symbol,
    Vec,
};

use crate::Error;

#[contract]
pub struct Advanced;

#[contractimpl]
impl Advanced {
    // §1.3 authorize_as_current_contract
    pub fn deep_call(env: Env, token: Address, dest: Address, amt: i128) {
        let me = env.current_contract_address();
        env.authorize_as_current_contract(vec![
            &env,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: token,
                    fn_name: symbol_short!("transfer"),
                    args: (me, dest, amt).into_val(&env),
                },
                sub_invocations: vec![&env],
            }),
        ]);
    }

    // §1.4 attestation as custom account
    pub fn attested(env: Env, attest: Address, recipient: Address, amount: i128) {
        attest.require_auth_for_args((recipient, amount).into_val(&env));
    }

    // §5.2 unknown-interface dispatch
    pub fn raw_invoke(env: Env, target: Address, amount: i128) -> i128 {
        env.invoke_contract(&target, &symbol_short!("allocate"), vec![&env, amount.into_val(&env)])
    }

    // §2.2 factory deploy
    pub fn factory(env: Env, wasm_hash: BytesN<32>, salt: BytesN<32>, arg: Address) -> Address {
        let d = env.deployer().with_current_contract(salt);
        let _precomputed: Address = d.deployed_address();
        d.deploy_v2(wasm_hash, (arg,))
    }

    // §6.4 cost estimate is testutils-only? checked separately
    pub fn noop(_env: Env) {}
}

// §1.4 custom account
#[contract]
pub struct AttestAccount;

#[contractimpl]
impl CustomAccountInterface for AttestAccount {
    type Error = Error;
    type Signature = Vec<BytesN<64>>;

    fn __check_auth(
        env: Env,
        signature_payload: soroban_sdk::crypto::Hash<32>,
        signatures: Vec<BytesN<64>>,
        auth_contexts: Vec<Context>,
    ) -> Result<(), Error> {
        // enforce policy from auth_contexts
        for c in auth_contexts.iter() {
            match c {
                Context::Contract(cc) => {
                    let _fn_name: Symbol = cc.fn_name;
                    let _who: Address = cc.contract;
                }
                Context::CreateContractHostFn(_) => return Err(Error::NotInitialized),
                Context::CreateContractWithCtorHostFn(_) => return Err(Error::NotInitialized),
            }
        }
        // verify the host-supplied payload itself
        for sig in signatures.iter() {
            env.crypto().ed25519_verify(
                &BytesN::from_array(&env, &[0u8; 32]),
                &signature_payload.clone().to_bytes().into(),
                &sig,
            );
        }
        Ok(())
    }
}

// Does `Signature` exist as a re-export we reference anywhere?

