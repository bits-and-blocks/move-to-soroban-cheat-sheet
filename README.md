# Move -> Soroban: the differential cheat sheet

For Aptos Move developers shipping Soroban contracts. Each entry is composed in the following structure:

- What it is in Move
- What it is in Soroban
- Where the analogy breaks.
- The breakage is the payload.

**Legend** — ⚠ **FALSE FRIEND**: your Move instinct produces compiling, working-looking Soroban code that is wrong. ∅ **NO ANALOGUE**: nothing on the other side — don't force one. 🔧 **DIRECT SWAP**: mechanical rename, low risk.

> **Verified against `soroban-sdk` 27.0.6 · rustc 1.91.1 · Aptos Move 9.4.0**
> Every Soroban snippet in this document compiles and, where it asserts behaviour, runs green — see [Verification](docs/12-verification.md). Move snippets are illustrative and not compiled.

**Version note.** `soroban-sdk` major = Stellar protocol version (SDK 27 ↔ protocol 27). Network limits and TTL floors are set by validator vote; the numbers in §6.4 match `InvocationResourceLimits::mainnet()` as baked into SDK 27.0.6 (a snapshot dated 2026-07-10 — the SDK itself notes these are hardcoded and require an SDK update to refresh). Check live values before relying on them: [Stellar Lab network limits](https://lab.stellar.org/network-limits) · `stellar network settings --network mainnet` · [crates.io/soroban-sdk](https://crates.io/crates/soroban-sdk).

---

## Quick map

| # | Concept | Move | Soroban | § |
|---|---------|------|---------|---|
| 1 | Transaction authority | `&signer` | `Address` param + `addr.require_auth()` | [1.1](docs/01-identity-and-authorization.md#11-signer--address--require_auth) ⚠ |
| 2 | Caller identity | `signer::address_of(s)` | the `Address` param itself | [1.1](docs/01-identity-and-authorization.md#11-signer--address--require_auth) 🔧 |
| 3 | Multi-party authorization | multi-agent tx (N signers) | N auth entries in one tx | [1.2](docs/01-identity-and-authorization.md#12-multi-agent-transactions--multiple-auth-entries) |
| 4 | Programmatic signing | resource account + `SignerCapability` | custom account (`__check_auth`) / invoker auth | [1.4](docs/01-identity-and-authorization.md#14-resource-accounts--signercapability--custom-accounts--invoker-auth) |
| 5 | Module-to-module access | `friend` / `public(friend)` | ∅ — auth checks replace visibility | [1.5](docs/01-identity-and-authorization.md#15-friend-visibility----auth-replaces-visibility) ∅ |
| 6 | Delegated authority | capability structs (`MintCap`, `AdminCap`…) | role `Address` in storage + `require_auth` | [2.1](docs/02-capabilities-and-the-object-model.md#21-capability-structs--role-addresses--require_auth-) ⚠ |
| 7 | Composable asset primitive | Aptos object model (`Object<T>`, refs) | ∅ — keyed entries or factory instances | [2.2](docs/02-capabilities-and-the-object-model.md#22-the-aptos-object-model----decomposition) ∅ |
| 8 | Enforced obligation | hot potato (no-ability struct) | ∅ — runtime checks in one invocation | [2.3](docs/02-capabilities-and-the-object-model.md#23-hot-potato--) ∅ |
| 9 | Storage operations | `move_to` / `borrow_global` / `exists` / `move_from` | `storage().{set,get,has,remove,update}` | [3.1](docs/03-state-and-storage.md#31-global-storage-operators--envstorage) ⚠ |
| 10 | State ownership | resources under user accounts | all state under the contract | [3.2](docs/03-state-and-storage.md#32-ownership-inversion-state-lives-under-the-contract) |
| 11 | Mutable state access | `borrow_global_mut` writes through | `get` returns a copy; write only on `set` | [3.3](docs/03-state-and-storage.md#33-get-returns-a-copy--false-friend) ⚠ |
| 12 | Access declaration | `acquires` (explicit or inferred) | transaction footprint, from simulation | [3.4](docs/03-state-and-storage.md#34-acquires--footprints-and-parallelism) |
| 13 | State lifetime | state persists once paid | durability tiers + TTL + archival | [3.5](docs/03-state-and-storage.md#35-storage-durability-ttl-archival--) ∅ ⚠ |
| 14 | Type-level guarantees | abilities `key/store/copy/drop` | ∅ — every value is plain data | [4.1](docs/04-types-and-abilities.md#41-abilities----the-headline-difference) ∅ ⚠ |
| 15 | Asset representation | `Coin<T>` value in hand | ∅ value objects — funds move only via token calls | [4.2](docs/04-types-and-abilities.md#42-no-value-objects-funds-never-pass-through-your-code) ⚠ |
| 16 | Type-level asset segregation | generic entry fns, phantom params | monomorphic ABI; asset identity = `Address` | [4.3](docs/04-types-and-abilities.md#43-generics--monomorphic-abi-asset--address-value) ⚠ |
| 17 | Data modelling | Move 2 enums / positional structs | `#[contracttype]` enums / tuple structs | [4.4](docs/04-types-and-abilities.md#44-structs-enums-receiver-functions-) 🔧 |
| 18 | Numeric semantics | signed types since Move 2.3, but amounts are `u64` | token amount `i128` is **signed** | [4.5](docs/04-types-and-abilities.md#45-integers-signedness--overflow--false-friend-2) ⚠ |
| 19 | Code deployment unit | module published at an address | Wasm hash (upload) + instances (deploy) | [5.1](docs/05-modules-vs-contracts.md#51-module--wasm-hash--contract-instance) |
| 20 | Dependency linking | compile-time `use other_module` | runtime client to an address held in storage | [5.2](docs/05-modules-vs-contracts.md#52-cross-contract-compile-time-linking--runtime-dispatch--false-friend) ⚠ |
| 21 | Function visibility | `entry` / `public` / private | one exported namespace, all-public | [5.3](docs/05-modules-vs-contracts.md#53-visibility-one-flat-exported-namespace) |
| 22 | Initialization | `init_module(&signer)` | `__constructor(env, args…)` | [5.4](docs/05-modules-vs-contracts.md#54-init_module--__constructor--and-strictly-better) 🔧 |
| 23 | Upgradeability | upgrade policy in `Move.toml` | upgrade entry point compiled in — or immutable by omission | [5.5](docs/05-modules-vs-contracts.md#55-upgrade-policy--compiled-in-upgrade-fn) |
| 24 | Transaction composition | scripts compose calls in one tx | exactly one invocation per tx | [5.6](docs/05-modules-vs-contracts.md#56-transaction-shape-one-invocation-per-transaction-) ∅ |
| 25 | Failure signalling | `abort` / `assert!` codes | `#[contracterror]` + `Result` / `panic_with_error!` | [6.1](docs/06-execution-semantics.md#61-abort-codes--contracterror) |
| 26 | Failure propagation | abort always propagates | callee errors catchable via `try_` clients | [6.2](docs/06-execution-semantics.md#62-catchable-callee-failures-) ∅ |
| 27 | Overflow safety | overflow aborts, always | Rust semantics behind a profile flag | [4.5](docs/04-types-and-abilities.md#45-integers-signedness--overflow--false-friend-2) ⚠ |
| 28 | Call dispatch | static dispatch, reentrancy impossible | dynamic dispatch, host blocks reentrancy | [6.3](docs/06-execution-semantics.md#63-dispatch-and-reentrancy) |
| 29 | Metering model | gas schedule, no declaration | multidimensional **declared** resources | [6.4](docs/06-execution-semantics.md#64-gas--declared-resources-and-hard-ceilings) |
| 30 | Off-chain observability | `#[event]` + `event::emit` | `#[contractevent]`, topics vs data, ~7-day retention | [7](docs/07-events.md) ⚠ |
| 31 | Fungible token standard | Coin / Fungible Asset standards | SEP-41 + Stellar Asset Contract | [8.1](docs/08-tokens.md#81-fungible-coinfa--sep-41--sac) ⚠ |
| 32 | Non-fungible token standard | Digital Asset standard (token objects + refs) | SEP-50 (**draft**) — and no SAC-equivalent trustable tier | [8.2](docs/08-tokens.md#82-non-fungible-digital-assets--sep-50) ⚠ |
| 33 | Unit testing | `#[test(a = @0x1)]`, `expected_failure` | `Env` testutils, `mock_all_auths` | [9](docs/09-testing-and-assurance.md) ⚠ |
| 34 | Formal verification | Move Prover (first-party, in-language `spec {}`) | no first-party tool — third-party Certora Sunbeam, Komet | [9](docs/09-testing-and-assurance.md) ⚠ |
| 35 | Read-only queries | `#[view]` + `/view` endpoint | ∅ attribute — any fn via `simulateTransaction`; view is a call mode | [5.7](docs/05-modules-vs-contracts.md#57-view-functions--any-function-invoked-in-simulation) ∅ ⚠ |
| 36 | Customizing token behaviour | static dispatch + DFA escape hatch (AIP-73) | dynamic dispatch is the *only* mechanism; the token is the hook | [5.8](docs/05-modules-vs-contracts.md#58-dispatchable-fas-aip-73--dynamic-dispatch--the-only-dispatch-there-is) ⚠ |
| 37 | Event access on-chain | reading emitted events on-chain | ∅ on both — write-only in production, readable in tests only | [7.1](docs/07-events.md#71-reading-events-on-chain---on-both-chains) ∅ ⚠ |

---

## Contents

1. [Identity & authorization](docs/01-identity-and-authorization.md)
2. [Capabilities & the object model](docs/02-capabilities-and-the-object-model.md)
3. [State & storage](docs/03-state-and-storage.md)
4. [Types & abilities](docs/04-types-and-abilities.md)
5. [Modules vs. contracts](docs/05-modules-vs-contracts.md)
6. [Execution semantics](docs/06-execution-semantics.md)
7. [Events](docs/07-events.md)
8. [Tokens](docs/08-tokens.md)
9. [Testing & assurance](docs/09-testing-and-assurance.md)
10. [Toolchain quick map](docs/10-toolchain.md)
11. [Porting reflexes — the grep-able checklist](docs/11-porting-reflexes.md)
12. [Verification](docs/12-verification.md)
13. [Primary sources](docs/13-primary-sources.md)

---

## Contributing

Corrections are the most valuable contribution here — a wrong mapping is worse than a missing
one. Missing Move concepts are equally welcome as issues even without an answer. See
[CONTRIBUTING.md](CONTRIBUTING.md); the one hard rule is that Soroban snippets must compile.
