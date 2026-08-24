# Move -> Soroban: the differential cheat sheet

For Aptos Move developer shipping Soroban contracts. Each entry is composed in the following structure:

- What it is in Move
- What it is in Soroban
- Where the analogy breaks.
- The breakage is the payload.

**Legend** — ⚠ **FALSE FRIEND**: your Move instinct produces compiling, working-looking Soroban code that is wrong. ∅ **NO ANALOGUE**: nothing on the other side — don't force one. 🔧 **DIRECT SWAP**: mechanical rename, low risk.

> **Verified against `soroban-sdk` 27.0.6 · rustc 1.91.1 · Aptos Move 9.4.0**
> Every Soroban snippet in this document compiles and, where it asserts behaviour, runs green — see [`verify/`](#12-verification). Move snippets are illustrative and not compiled.

**Version note.** `soroban-sdk` major = Stellar protocol version (SDK 27 ↔ protocol 27). Network limits and TTL floors are set by validator vote; the numbers in §6.4 match `InvocationResourceLimits::mainnet()` as baked into SDK 27.0.6 (a snapshot dated 2026-07-10 — the SDK itself notes these are hardcoded and require an SDK update to refresh). Check live values before relying on them: [Stellar Lab network limits](https://lab.stellar.org/network-limits) · `stellar network settings --network mainnet` · [crates.io/soroban-sdk](https://crates.io/crates/soroban-sdk).

---

## 0. Quick map

| # | Concept | Move | Soroban | § |
|---|---------|------|---------|---|
| 1 | Transaction authority | `&signer` | `Address` param + `addr.require_auth()` | [1.1](#11-signer--address--require_auth) ⚠ |
| 2 | Caller identity | `signer::address_of(s)` | the `Address` param itself | [1.1](#11-signer--address--require_auth) 🔧 |
| 3 | Multi-party authorization | multi-agent tx (N signers) | N auth entries in one tx | [1.2](#12-multi-agent-transactions--multiple-auth-entries) |
| 4 | Programmatic signing | resource account + `SignerCapability` | custom account (`__check_auth`) / invoker auth | [1.4](#14-resource-accounts--signercapability--custom-accounts--invoker-auth) |
| 5 | Module-to-module access | `friend` / `public(friend)` | ∅ — auth checks replace visibility | [1.5](#15-friend-visibility----auth-replaces-visibility) ∅ |
| 6 | Delegated authority | capability structs (`MintCap`, `AdminCap`…) | role `Address` in storage + `require_auth` | [2.1](#21-capability-structs--role-addresses--require_auth-) ⚠ |
| 7 | Composable asset primitive | Aptos object model (`Object<T>`, refs) | ∅ — keyed entries or factory instances | [2.2](#22-the-aptos-object-model----decomposition) ∅ |
| 8 | Enforced obligation | hot potato (no-ability struct) | ∅ — runtime checks in one invocation | [2.3](#23-hot-potato--) ∅ |
| 9 | Storage operations | `move_to` / `borrow_global` / `exists` / `move_from` | `storage().{set,get,has,remove,update}` | [3.1](#31-global-storage-operators--envstorage) ⚠ |
| 10 | State ownership | resources under user accounts | all state under the contract | [3.2](#32-ownership-inversion-state-lives-under-the-contract) |
| 11 | Mutable state access | `borrow_global_mut` writes through | `get` returns a copy; write only on `set` | [3.3](#33-get-returns-a-copy--false-friend) ⚠ |
| 12 | Access declaration | `acquires` (explicit or inferred) | transaction footprint, from simulation | [3.4](#34-acquires--footprints-and-parallelism) |
| 13 | State lifetime | state persists once paid | durability tiers + TTL + archival | [3.5](#35-storage-durability-ttl-archival--) ∅ ⚠ |
| 14 | Type-level guarantees | abilities `key/store/copy/drop` | ∅ — every value is plain data | [4.1](#41-abilities----the-headline-difference) ∅ ⚠ |
| 15 | Asset representation | `Coin<T>` value in hand | ∅ value objects — funds move only via token calls | [4.2](#42-no-value-objects-funds-never-pass-through-your-code) ⚠ |
| 16 | Type-level asset segregation | generic entry fns, phantom params | monomorphic ABI; asset identity = `Address` | [4.3](#43-generics--monomorphic-abi-asset--address-value) ⚠ |
| 17 | Data modelling | Move 2 enums / positional structs | `#[contracttype]` enums / tuple structs | [4.4](#44-structs-enums-receiver-functions-) 🔧 |
| 18 | Numeric semantics | signed types since Move 2.3, but amounts are `u64` | token amount `i128` is **signed** | [4.5](#45-integers-signedness--overflow--false-friend-2) ⚠ |
| 19 | Code deployment unit | module published at an address | Wasm hash (upload) + instances (deploy) | [5.1](#51-module--wasm-hash--contract-instance) |
| 20 | Dependency linking | compile-time `use other_module` | runtime client to an address held in storage | [5.2](#52-cross-contract-compile-time-linking--runtime-dispatch--false-friend) ⚠ |
| 21 | Function visibility | `entry` / `public` / private | one exported namespace, all-public | [5.3](#53-visibility-one-flat-exported-namespace) |
| 22 | Initialization | `init_module(&signer)` | `__constructor(env, args…)` | [5.4](#54-init_module--__constructor--and-strictly-better) 🔧 |
| 23 | Upgradeability | upgrade policy in `Move.toml` | upgrade entry point compiled in — or immutable by omission | [5.5](#55-upgrade-policy--compiled-in-upgrade-fn) |
| 24 | Transaction composition | scripts compose calls in one tx | exactly one invocation per tx | [5.6](#56-transaction-shape-one-invocation-per-transaction-) ∅ |
| 25 | Failure signalling | `abort` / `assert!` codes | `#[contracterror]` + `Result` / `panic_with_error!` | [6.1](#61-abort-codes--contracterror) |
| 26 | Failure propagation | abort always propagates | callee errors catchable via `try_` clients | [6.2](#62-catchable-callee-failures-) ∅ |
| 27 | Overflow safety | overflow aborts, always | Rust semantics behind a profile flag | [4.5](#45-integers-signedness--overflow--false-friend-2) ⚠ |
| 28 | Call dispatch | static dispatch, reentrancy impossible | dynamic dispatch, host blocks reentrancy | [6.3](#63-dispatch-and-reentrancy) |
| 29 | Metering model | gas schedule, no declaration | multidimensional **declared** resources | [6.4](#64-gas--declared-resources-and-hard-ceilings) |
| 30 | Off-chain observability | `#[event]` + `event::emit` | `#[contractevent]`, topics vs data, ~7-day retention | [7](#7-events) ⚠ |
| 31 | Fungible token standard | Coin / Fungible Asset standards | SEP-41 + Stellar Asset Contract | [8.1](#81-fungible-coinfa--sep-41--sac) ⚠ |
| 32 | Non-fungible token standard | Digital Asset standard (token objects + refs) | SEP-50 (**draft**) — and no SAC-equivalent trustable tier | [8.2](#82-non-fungible-digital-assets--sep-50) ⚠ |
| 33 | Unit testing | `#[test(a = @0x1)]`, `expected_failure` | `Env` testutils, `mock_all_auths` | [9](#9-testing--assurance) ⚠ |
| 34 | Formal verification | Move Prover (first-party, in-language `spec {}`) | no first-party tool — third-party Certora Sunbeam, Komet | [9](#9-testing--assurance) ⚠ |
| 35 | Read-only queries | `#[view]` + `/view` endpoint | ∅ attribute — any fn via `simulateTransaction`; view is a call mode | [5.7](#57-view-functions--any-function-invoked-in-simulation) ∅ ⚠ |
| 36 | Customizing token behaviour | static dispatch + DFA escape hatch (AIP-73) | dynamic dispatch is the *only* mechanism; the token is the hook | [5.8](#58-dispatchable-fas-aip-73--dynamic-dispatch--the-only-dispatch-there-is) ⚠ |
| 37 | Event access on-chain | reading emitted events on-chain | ∅ on both — write-only in production, readable in tests only | [7.1](#71-reading-events-on-chain---on-both-chains) ∅ ⚠ |

---

## 1. Identity & authorization

### 1.1 `&signer` → `Address` + `require_auth()`

**Move** — `signer` is an unforgeable capability minted only by the VM for actual transaction signers; possession *is* the proof of consent. It has only `drop` — even its scarcity is ability-encoded.
[Move: signer](https://aptos.dev/build/smart-contracts/book/signer) · [Soroban: authorization](https://developers.stellar.org/docs/learn/fundamentals/contract-development/authorization)

**Soroban** — there is no `msg.sender` and no signer type. The acting party is an ordinary `Address` parameter (plain, cloneable data), and consent is a separate runtime host query:

```move
// Move: the type is the gate
public entry fun contribute(donor: &signer, amount: u64) { … }
```

```rust
// Soroban: the parameter is data; the gate is a call you must remember to make
pub fn contribute(env: Env, from: Address, asset: Address, amount: i128) {
    from.require_auth();   // did `from` sign THIS contract, THIS fn, THESE args, in this tx?
    // require_auth_for_args((&asset, amount).into_val(&env)) narrows the signed payload
}
```

`signer::address_of(&s)` 🔧 disappears — the `Address` is already what you have.

**Where it breaks — internalize all four:**

- ⚠ **FALSE FRIEND** — an `Address` parameter proves nothing by itself. Forgetting `require_auth` compiles, runs, and passes naive tests. Every `Address` arg forces a decision: does this call spend, reduce, or reconfigure something this address owns? If yes → auth it. Reading public data / crediting a balance → no.
- ⚠ **FALSE FRIEND** — `who.require_auth()` on a caller-supplied `who` proves the caller controls `who`, **not** that `who` is privileged. For admin paths, load the role address from storage and auth *that* (Move analogue: `assert!(address_of(s) == @admin)` — same discipline, but here it's the *only* line of defense).
- **Auth is a signed invocation tree, and it does not cascade.** The user signs "`A.foo(args)`, and within it `B.bar(args)`". `require_auth` deep in the stack passes only if the actual call path matches the signed tree; each node matches at most once per tx. Refactoring your call graph changes the tree shape and **silently breaks integrators** signing the old shape — the tree is ABI.
- ⚠ **FALSE FRIEND (the #1 real-world Soroban auth bug)** — auth replay through middleware. If `settle(user, amt)` calls `token.transfer(&user, …)` but never calls `user.require_auth()`, anyone can call `settle(victim, …)` and consume victim's pre-signed inner transfer auth. Re-auth at **every** layer that exercises an address's authority.

Replay protection (nonces, expirations on auth entries) is host-managed — you never track nonces.
[Auth starter guide](https://developers.stellar.org/docs/build/guides/auth/contract-authorization) · [Aptos security guidelines (signer-check discipline)](https://aptos.dev/build/smart-contracts/move-security-guidelines)

### 1.2 Multi-agent transactions → multiple auth entries

**Move** — a script/entry fn takes several `&signer`s; all sign the one transaction.
**Soroban** — one transaction carries multiple `SorobanAuthorizationEntry`s, one per authorizing `Address`; the contract just calls `require_auth()` on each party (atomic swap: `a.require_auth(); b.require_auth();`). Only the tx *source* pays fees; other parties sign auth entries, not the envelope.
[Move: signer (multi-signer scripts)](https://aptos.dev/build/smart-contracts/book/signer) · [Soroban: authorization](https://developers.stellar.org/docs/learn/fundamentals/contract-development/authorization)

### 1.3 Contracts as authorizing parties (invoker auth)

∅ in Move (a module is code, not a party; you mint signers for that — §1.4). In Soroban a contract address **is** a first-class auth party. When contract A directly calls B, B's `a_address.require_auth()` passes automatically — the direct call is the authorization. That covers B's immediate frame only; if B will make a *deeper* call needing A's authority, A pre-authorizes it:

```rust
env.authorize_as_current_contract(vec![&env, InvokerContractAuthEntry::Contract(
    SubContractInvocation {
        context: ContractContext { contract: token, fn_name: symbol!("transfer"),
                                   args: (me, dest, amt).into_val(&env) },
        sub_invocations: vec![&env],
    })]);
```

This is your `zakat_pool → SAC.transfer` path when the pool pays out: the pool's own address authorizes the transfer by being the direct invoker.

### 1.4 Resource accounts / `SignerCapability` → custom accounts + invoker auth

**Move** — a resource account is a keyless account whose `SignerCapability` lets a module mint its signer programmatically; the capability struct is the authority.
[Aptos: resource accounts](https://aptos.dev/build/smart-contracts/resource-accounts)

**Soroban** — split by what you used it for:

- *"My module acts as its own account"* → free. `env.current_contract_address()` holds tokens and passes auth as invoker (§1.3). No capability object to store or guard.
- *"Programmable signing policy"* (multisig, passkeys, spend limits) → a **custom account**: a contract implementing `CustomAccountInterface`; the host calls its `__check_auth(env, signature_payload: crypto::Hash<32>, signature, auth_contexts: Vec<Context>)` whenever that contract address must authorize something. Note the payload type: **`crypto::Hash<32>`, not `BytesN<32>`** — a host-constructed type you cannot forge in safe code, which is the type system doing the one job here it still does. Rules: verify that payload itself (verifying anything else authorizes arbitrary calls); enforce policy from `auth_contexts` (match on `Context::Contract` for the invoked contract and `fn_name`) or the policy is decorative; keep it lean — its cost rides on every tx the account signs. Protocol 27 (CAP-71) adds `delegate_auth` for modular signer contracts — delegate addresses arrive **unsanitized**, check registration first.
[Custom account example](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/custom-account)

This is the natural home for an attestation-authority quorum: the pool does `attest_addr.require_auth_for_args((recipient, asset, category, amount).into_val(&env))` and the 2-of-3 policy lives in `__check_auth` — host-managed nonces replace a hand-rolled sequence scheme and remove its head-of-line blocking.

### 1.5 `friend` visibility → ∅ — auth replaces visibility

**Move** — `public(friend)` / `public(package)` restrict callers at the type level.
[Move: friends](https://aptos.dev/build/smart-contracts/book/friends)

**Soroban** — every `pub fn` in `#[contractimpl]` is callable by anyone, externally and cross-contract. No friend, no package visibility. Replacements, in order of preference:

1. Design the function to be safe for any caller (a pure `policy.allocate(amount)` computation needs no gate — the *caller* validates the result).
2. Role auth from storage (§2.1) for privileged paths.
3. Pin the expected caller: store contract B's `Address`, and `expected_caller.require_auth()` — passes only when B is the direct invoker (§1.3).

Private helpers = ordinary non-`pub` Rust fns, or fns outside `#[contractimpl]`.

---

## 2. Capabilities & the object model

### 2.1 Capability structs → role addresses + `require_auth` ⚠

**Move** — authority as a scarce value: `MintCapability`, `BurnCapability`, your `AdminCap`. Scarcity is type-enforced — no `copy`, constructor private to the defining module, possession = permission.
[Move: structs & resources](https://aptos.dev/build/smart-contracts/book/structs-and-resources) · [abilities](https://aptos.dev/build/smart-contracts/book/abilities)

**Soroban** — ⚠ **FALSE FRIEND: there are no scarce values.** Any code can construct any `#[contracttype]` value; a `DistributorCap` struct passed as an argument is trivially forgeable, and one *stored* by your contract grants nothing by existing — nothing checks for it. Porting the capability pattern literally produces decorative security.

The replacement is **where + who**, not **what**: authority = an `Address` held in your contract's storage, consumed via `require_auth`:

```rust
// The whole of "DistributorCap":
fn require_distributor(env: &Env) {
    let d: Address = env.storage().instance().get(&DataKey::Distributor).unwrap();
    d.require_auth();
}
```

What you lose vs. Move: capabilities were *transferable, storable, divisible* values (hand a `MintCap` to another module, wrap it, time-lock it). The Soroban equivalents: rotate the role address (governance setter), point the role at a **custom account** contract encoding the fancier policy (§1.4), or split into multiple role keys — exactly the governance / distributor / attestation-authority triple.

### 2.2 The Aptos object model → ∅ — decomposition

**Move/Aptos** — `Object<T>`: an address-identified resource group with `ObjectCore` (ownership, transferability), capability refs minted from `ConstructorRef` (`ExtendRef`, `TransferRef`, `DeleteRef`), objects owning objects — the substrate under FA, DA, and AIP-76 composability.
[Aptos: objects](https://aptos.dev/build/smart-contracts/objects) · [creating](https://aptos.dev/build/smart-contracts/object/creating-objects) / [using](https://aptos.dev/build/smart-contracts/object/using-objects) · [AIP-76](https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-76.md)

**Soroban** — no object model, and no middle layer at all: the platform has exactly **contracts** and **flat KV entries**. Aptos pulled contract-shaped things *down* into the data model; on Soroban you push object-shaped things *up or down* — each object use decomposes separately:

| Object usage in Aptos | Soroban replacement |
|---|---|
| Identity + attached resources (state bundle) | Entries under a `#[contracttype]` key in the managing contract — the default, and almost always right |
| Ownable / transferable thing | `owner: Address` field in the entry; mutations gated by `owner.require_auth()` |
| Autonomous actor holding assets | **Factory-deployed contract instance** — deterministic address from `(deployer, salt)` via `env.deployer().with_address(me, salt).deploy_v2(wasm_hash, args)`; `deployed_address()` precomputes it. The instance holds tokens and is an auth party |
| `ConstructorRef` → `Extend/Transfer/DeleteRef` | ∅ capability values. Role addresses + auth checks; "delete" = `storage().remove()` |
| Objects owning objects (AIP-76 composition) | Contract addresses as owners/actors; composition is cross-contract calls, not resource attachment |

[Deployer example](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/deployer)

Rule of thumb: reach for a keyed entry first; deploy an instance only when the thing must *independently* hold tokens or authorize.

### 2.3 Hot potato → ∅

A no-ability struct that must be consumed in the same transaction (flash-loan receipts, forced-callback patterns) has no Soroban equivalent — no abilities means no un-droppable values. Two partial substitutes: a Soroban tx is a single invocation anyway (§5.6), so "same-tx" is often structural; and obligations become explicit runtime checks before returning (assert the loan repaid, assert the invariant). Type-enforced obligations are simply gone.

---

## 3. State & storage

### 3.1 Global storage operators → `env.storage()`

| Move | Soroban | Notes |
|---|---|---|
| `move_to(&s, r)` | `storage().<tier>().set(&key, &val)` | no signer needed — it's *your* storage (§3.2) |
| `borrow_global<T>(a)` | `get::<K, V>(&key) -> Option<V>` | **owned copy**, not a borrow ⚠ §3.3 |
| `borrow_global_mut<T>(a)` | `update(&key, \|cur: Option<V>\| { … })` | the only safe read-modify-write idiom |
| `exists<T>(a)` | `has(&key)` | |
| `move_from<T>(a)` | `get` + `remove` | `remove` returns nothing; atomicity holds within the invocation anyway |

[Move: global storage operators](https://aptos.dev/build/smart-contracts/book/global-storage-operators) · [Soroban: persisting data](https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage/persisting-data)

Keys are anything `#[contracttype]` — always a dedicated enum:

```rust
#[contracttype] #[derive(Clone)]
pub enum DataKey {
    Governance, Distributor, PolicyAddr, AttestAddr, Paused,
    ApprovedAssets,
    Buckets(Address),           // all eight category balances for one asset, one entry
}
```

Untyped/ad-hoc keys can silently collide — the enum is the schema, and adding variants is the forward-compatible way to grow it.

### 3.2 Ownership inversion: state lives under the contract

**Move** — resources live *under user accounts*; your module is the API over state it doesn't hold (`Balance` under each user's address). The user pays their own storage; `move_to` needs their signer.
[Move: global storage structure](https://aptos.dev/build/smart-contracts/book/global-storage-structure)

**Soroban** — a contract reads and writes **only its own storage**. "The user's balance" is *your* entry `Balance(Address)`. Consequences: no signer needed to write (auth is orthogonal, §1.1); no `exists`-under-user checks or per-user initialization (`unwrap_or(default)` replaces the register-then-use dance); the contract's footprint is the universe — nothing of yours lives anywhere else; and *you* carry the growth and rent of per-user state (§3.5) rather than distributing it to users. Bounding what grows is now an architecture decision, not a billing default.

### 3.3 `get` returns a copy ⚠ FALSE FRIEND

The first-hour bug, and it produces no error of any kind:

```rust
// WRONG — compiles, reads back correctly within this call, never touches the ledger
let mut b: BucketSet = env.storage().persistent().get(&DataKey::Buckets(asset)).unwrap();
b.riqab += amount;          // mutated a deserialized local
// …no set → the write is lost
```

`get` deserializes an owned value out of the host; there is no `borrow_global_mut`, no write-through reference, and the borrow checker cannot help because nothing is borrowed. Codebase rule worth enforcing with review, not memory: **`get` + field mutation is banned; every read-modify-write goes through `update`/`try_update`:**

```rust
env.storage().persistent().update(&DataKey::Buckets(asset), |cur: Option<BucketSet>| {
    let mut b = cur.unwrap_or_default();
    b.add(category, amount);           // checked_add inside
    b
});
```

Also ⚠: `#[contract] pub struct ZakatPool { … }` fields do **not** persist — the struct is a namespace, not a resource. All state goes through `env.storage()`.

### 3.4 `acquires` → footprints, and parallelism

**Move** — `acquires` (explicit, or inferred by the Move 2 compiler) is function-level bookkeeping of global-storage access; Block-STM discovers conflicts optimistically at runtime.
[Move 2 release notes](https://aptos.dev/build/smart-contracts/book/move-2)

**Soroban** — nothing at the function level. Every **transaction** declares its read/write footprint up front (computed by simulation, embedded in the tx). Transactions touching the same read-write entry serialize; disjoint ones parallelize — *pessimistic* declaration vs. Block-STM's optimistic detection. Design consequences:

- **Fine-grained keys are the parallelism lever.** `Balance(Address)` per user, never one giant `Map` — a giant map is one entry, so every caller conflicts with every other *and* pays to read the whole thing.
- Everything touched must be in the footprint whether used or not, and entries-per-tx is capped (mainnet: 200 read / 200 write). Unbounded iteration over entries is both a fee and a ceiling problem.
- If actual state diverges from simulation (concurrent writes between simulate and submit), declared costs can be wrong → leave headroom.

### 3.5 Storage durability, TTL, archival — ∅

Nothing in Move prepares you for this: Aptos state persists indefinitely once paid (deposit refunded on deletion). Soroban storage is **rented**, in three tiers:

| Tier | Lifetime | On expiry | Use for |
|---|---|---|---|
| `instance()` | one TTL shared with the contract instance; **single entry**, ≤64KB serialized | archived with the contract; restorable | admin/config/small globals — one `extend_ttl` bumps everything |
| `persistent()` | per-key TTL | **archived** — restorable, and since protocol 23 auto-restored when in a tx's read-write footprint (you re-pay rent) | balances, anything that must survive |
| `temporary()` | per-key TTL | **deleted, permanently** | caches, quotes, inherently time-bounded data |

TTL is counted in ledgers (~5s, 17,280/day). Mainnet at time of writing: new persistent entries ~120 days, temporary ~1 day, ceiling ~180 days — network-configured, check live. `extend_ttl(threshold, extend_to)` is floor-only and idempotent: no-op unless current TTL < threshold. Standard pattern: extend the touched entry on every write ("active users pay for their own state"), extend instance TTL at the top of busy entry points. Protocol 26 adds `extend_ttl_with_limits` to stop callers forcing arbitrary rent on you.

```rust
const DAY: u32 = 17_280;
env.storage().instance().extend_ttl(30 * DAY, 120 * DAY);
env.storage().persistent().extend_ttl(&DataKey::Buckets(asset), 30 * DAY, 120 * DAY);
```

- ⚠ **FALSE FRIEND** — `temporary()` for anything load-bearing = permanent, silent data loss. Expired means gone, no restore path.
- ⚠ **FALSE FRIEND** — TTL is **not a security mechanism**. *Anyone* can extend *any* entry's TTL via the `ExtendFootprintTTLOp` transaction op, no contract auth involved. "This permission dies when its entry expires" is broken by design — store an explicit deadline in the value and check it.
- **Operational, not launch-time:** max TTL ~180 days means an immutable custody contract stays alive only if a keep-alive job runs forever. That job is part of the system, with an owner and a runbook — an unowned cron is how "nothing load-bearing archives" quietly stops being true.
- Never assume an entry you wrote is present later: `unwrap_or(default)` / `has()` where absence is meaningful.

[State archival](https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage/state-archival) · [Choosing the right storage](https://developers.stellar.org/docs/build/guides/storage/choosing-the-right-storage)

---

## 4. Types & abilities

### 4.1 Abilities → ∅ — the headline difference

`key/store/copy/drop` did two structurally different jobs, and both are gone:

1. **Conservation.** `Coin<T>` without `copy`/`drop` cannot be forged, duplicated, or silently discarded — the type checker was your solvency proof.
2. **Authority.** `signer` without `store`/`copy` cannot be minted or stashed — possession was proof.

Every `#[contracttype]` value is plain Rust data: cloneable, droppable, constructible by anyone. Both invariants collapse into **runtime checks you write and test**: conservation becomes checked arithmetic plus an asserted invariant (below); authority becomes `require_auth` (§1) and role addresses (§2.1). This is the axis where Soroban is genuinely *worse* than Move — the two most expensive Soroban bug classes in the wild (missing auth, balance-accounting drift) are exactly what abilities used to prevent, and nothing warns you they're missing.

The replacement for `Coin` linearity in a pool, concretely — a compiled artifact, not a comment:

```rust
fn assert_solvent(env: &Env, asset: &Address) {
    let held: i128 = token::TokenClient::new(env, asset)
        .balance(&env.current_contract_address());
    let booked: i128 = env.storage().persistent()
        .get::<_, BucketSet>(&DataKey::Buckets(asset.clone()))
        .unwrap_or_default().total();          // checked_add inside; traps on overflow
    if booked > held { panic_with_error!(env, Error::Insolvent); }  // > : surplus is legal
}
```

Call it at the end of every value-moving function, and make it the oracle for property tests (§9).
[Move: abilities](https://aptos.dev/build/smart-contracts/book/abilities) · [Soroban: custom types example](https://developers.stellar.org/docs/build/smart-contracts/example-contracts)

### 4.2 No value objects: funds never pass through your code

**Move** — `coin::withdraw` puts a `Coin` *in your hand*; you hold it, split it, merge it, deposit it. Escrow = storing the `Coin` in your resource.

**Soroban** — ⚠ there is nothing to hold. A token is an external contract keeping balances internally; "moving funds" is calling it, and the transfer completes *inside that call*:

```rust
token::TokenClient::new(&env, &asset)
    .transfer(&from, &env.current_contract_address(), &amount);
// no value returned; nothing in hand; your address's balance in the token contract went up
```

Escrow/custody = the token contract says your address holds N, plus **your own bookkeeping of who it's for** — the eight-bucket accounting *is* the replacement for held `Coin`s, which is why §4.1's invariant is load-bearing. "Split" and "merge" are arithmetic on your books, not operations on values.

### 4.3 Generics → monomorphic ABI; asset = `Address` value

**Move** — `Coin<USDC>` and `Coin<EURC>` are distinct types; phantom parameters give type-level asset segregation, and mixing them is a compile error.
[Move: generics](https://aptos.dev/build/smart-contracts/book/generics)

**Soroban** — contract entry points are monomorphic; nothing generic crosses the ABI (Rust generics are fine internally). Asset identity is an `Address` **value** at runtime. Type-level segregation becomes **key discipline**: `Buckets(Address)` per asset, an `approved_assets` allowlist at the boundary, and per-asset invariants. ⚠ The compiler will never again catch USDC amounts credited to EURC buckets — only your keys and your tests do. (Deterministic SAC addresses help: pin the USDC/EURC addresses as reviewed constants per network rather than discovering them at runtime.)

### 4.4 Structs, enums, receiver functions 🔧

- Move 2 enums → `#[contracttype]` enums (unit + tuple variants; the idiomatic `DataKey`). Adding variants is forward-compatible; reshaping existing ones breaks stored-data decoding (§5.5).
- Positional structs → Rust tuple structs (supported by `#[contracttype]`).
- Receiver-style `value.method()` → ordinary Rust methods internally; **exported entry points** are associated fns in `#[contractimpl]` taking `Env` first — no receiver syntax at the ABI.
- Type names crossing the ABI are exported flat into the contract spec — keep them unique per contract.
- SDK types replace std (`#![no_std]`): `soroban_sdk::{String, Vec, Map, Bytes, BytesN, Symbol}` — host-object handles, not std collections. `Symbol` ≤32 chars `[a-zA-Z0-9_]`; `symbol_short!` ≤9.

[Move 2 release notes](https://aptos.dev/build/smart-contracts/book/move-2) · [docs.rs: contracttype](https://docs.rs/soroban-sdk/latest/soroban_sdk/attr.contracttype.html)

### 4.5 Integers: signedness & overflow ⚠ FALSE FRIEND ×2

**Move** — signed types **do** exist as of Move 2.3 (`i8…i256` alongside `u8…u256`, literals like `-1i8`), but the money path never meets them: `Coin`/FA amounts are `u64`, so a negative amount stays unrepresentable exactly where it would hurt. And every arithmetic op — signed and unsigned alike — **aborts** on overflow, underflow, or divide-by-zero with no opt-out; division truncates, negating a type's minimum aborts, and `as` casts abort when the value doesn't fit.
[Move: primitive types](https://aptos-labs.github.io/move-book/primitive-types.html) · [Move 2.3 release notes](https://aptos-labs.github.io/move-book/move-versions.html) · [abort & assert](https://aptos.dev/build/smart-contracts/book/abort-and-assert)

**Soroban** — two independent traps:

- ⚠ The canonical token amount is `i128` — **signed**. `transfer(from, to, -1000)` is a withdrawal *from `to`* if unchecked. Validate `amount > 0` at **every** entry point taking an amount. Having `i128` in the language is not the same as meeting it on the money path: Aptos amounts are `u64`, so no Move reflex was ever trained against this attack.
- ⚠ Overflow safety is a **build-profile flag**, not a language guarantee: Rust wraps in release unless `overflow-checks = true` is set (the Stellar scaffold sets it; a copied `Cargo.toml` without it wraps silently). Use explicit `checked_add/sub/mul` on every money path regardless — it survives profile mistakes and fails with *your* typed error. `U256`/`I256` have `checked_*` (protocol 26+).

---

## 5. Modules vs. contracts

### 5.1 Module → Wasm hash + contract instance

**Move** — publishing puts module bytes at an address; code identity ≡ address; one deployment per address.
[Move: modules & scripts](https://aptos.dev/build/smart-contracts/book/modules-and-scripts) · [packages](https://aptos.dev/build/smart-contracts/book/packages)

**Soroban** — two-level, content-addressed: **upload** Wasm once (identified by hash, deduplicated network-wide), then **deploy** instances (address + wasm-hash pointer + instance storage). Many instances per binary — ∅ in Aptos Move, and the substrate for factories (§2.2). `stellar contract upload` for code-only (feeds factories and upgrades); `stellar contract deploy` for an instance. Addresses: accounts `G…`, contracts `C…` — both unify as `Address`.
[Getting started: deploy](https://developers.stellar.org/docs/build/smart-contracts/getting-started)

### 5.2 Cross-contract: compile-time linking → runtime dispatch ⚠ FALSE FRIEND

**Move** — `use other_addr::vault;` is static linking: callee identity fixed at compile time, verified at publish, dispatch static.

**Soroban** — every cross-contract call is dynamic dispatch to an `Address` you hold *at runtime*, usually from your own storage:

```rust
mod policy_contract { soroban_sdk::contractimport!(file = "policy.wasm"); }  // typed client from wasm

let policy: Address = env.storage().instance().get(&DataKey::PolicyAddr).unwrap();
let split = policy_contract::Client::new(&env, &policy).allocate(&amount);

// SEP-41 tokens need no import — the SDK ships the client:
let t = soroban_sdk::token::TokenClient::new(&env, &asset);

// Unknown interface at compile time:
let x: i128 = env.invoke_contract(&target, &symbol, args);
```

- ⚠ **FALSE FRIEND** — a callee address is **arbitrary code** wearing an interface. It can lie, trap, burn budget, and emit misleading events (it cannot re-enter you — §6.3). Any user-supplied contract address must be allowlisted (`approved_assets`), and any callee return value must be validated — sum/range/length-check the allocation, freshness-check the oracle. "Untrusted collaborator" is the correct posture even for contracts you deployed, since governance can repoint them.
- The pinned-address-in-storage pattern is also your inter-contract *upgrade* mechanism (§5.5).

[Cross-contract example](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/cross-contract-call) · [docs.rs: contractimport](https://docs.rs/soroban-sdk/latest/soroban_sdk/macro.contractimport.html)

### 5.3 Visibility: one flat exported namespace

`entry` vs `public` vs private → gone. Every `pub fn` in `#[contractimpl]` is simultaneously the external entry point and the cross-contract interface; there is no "callable by transactions but not by contracts" or vice versa. Access control is done with auth (§1.5), never with visibility — because there is none to do it with.
[Move: functions](https://aptos.dev/build/smart-contracts/book/functions)

### 5.4 `init_module` → `__constructor` 🔧 (and strictly better)

**Move** — `init_module(&signer)` runs at publish, takes only the publisher's signer; real parameterization needs a follow-up call.

**Soroban** — `__constructor(env, args…)` (exact name) runs **once, atomically, at deploy, with arbitrary typed args** passed after `--` in the deploy command. Failure aborts the deployment; it never re-runs (not on upgrade). This kills the deploy-then-`initialize` front-running window — prefer it always. A contract deployed without one can't gain one retroactively; if forced into a guarded `initialize` (legacy), check-and-set an `Initialized` flag or anyone can capture admin.

### 5.5 Upgrade policy → compiled-in upgrade fn

**Move** — policy declared in the manifest: `compatible` (layout/signature-compatible upgrades) or `immutable`.
[Move: package upgrades](https://aptos.dev/build/smart-contracts/book/package-upgrades)

**Soroban** — no manifest, no compatibility checker. A contract is mutable **iff** it exports a function calling `env.deployer().update_current_contract_wasm(new_hash)` — gate it with admin auth. **Immutability = omission**: no such call site anywhere, no other path exists (no delegatecall). Address and all storage survive an upgrade; `__constructor` does not re-run.

⚠ **FALSE FRIEND** — there is no `compatible`-policy safety net: new code reading an old key whose `#[contracttype]` shape changed **fails to decode at runtime**. Schema migration is entirely on you: store a schema version, add enum variants rather than reshaping, ship an idempotent admin-gated `migrate` enforcing `new > current`. And error codes are ABI — never renumber (§6.1). The three-contract split gives the complementary mechanism: repoint a stored dependency address (§5.2) instead of mutating code — a contract should have one upgrade path, not both.
[Upgrading contracts](https://developers.stellar.org/docs/build/guides/conventions/upgrading-contracts)

### 5.6 Transaction shape: one invocation per transaction ∅

**Move** — a script sequences arbitrary calls atomically; entry fns compose framework calls freely.
[Move: scripts](https://aptos.dev/build/smart-contracts/scripts/writing-scripts)

**Soroban** — a transaction carries **exactly one** contract invocation (one `InvokeHostFunction` op; Soroban txs are single-operation). There is no client-side multicall. Atomic multi-step = a contract function making the calls (a periphery/router contract if the steps span protocols). Anything your docs describe as "then the client calls X and Y" is actually one contract function or two transactions — decide which.

Adjacent quick hit: Aptos fee-payer/sponsored txs ↔ Stellar **fee-bump transactions** (fees) + **sponsored reserves** (account/trustline reserves) — protocol features, not contract code. [Sponsored reserves](https://developers.stellar.org/docs/learn/encyclopedia/transactions-specialized/sponsored-reserves)

### 5.7 `#[view]` functions → any function, invoked in simulation

**Move/Aptos** — `#[view]` is an opt-in marker gating the node's free read path: only tagged fns are callable through the fullnode `/view` endpoint or `aptos move view` (calling an untagged fn there fails). No signer, no gas; the API discards any state mutation (`#[view]` is *not* compile-time purity — the read path just drops writes). It exists partly because entry-fn return values are inaccessible to the submitter, so computed reads need a dedicated surface.
[Aptos: fullnode REST API & view functions](https://aptos.dev/build/apis/fullnode-rest-api)

**Soroban** — ∅ attribute, and none is needed: **"view" is a call mode, not a function property.** Any `pub fn` can be executed through RPC `simulateTransaction` — no signature (auth runs in *recording* mode: `require_auth` is noted, never enforced), no fee, execution effects discarded, return value in the simulation result. Return values are also accessible from *submitted* txs, so the entry/view split never existed: one `bucket_balance(asset, category)` serves cross-contract callers, off-chain readers, and transactions alike.

```rust
pub fn bucket_balance(env: Env, asset: Address, category: Category) -> i128 {
    env.storage().persistent()
        .get::<_, BucketSet>(&DataKey::Buckets(asset))
        .unwrap_or_default()
        .get(category)          // pure: no writes, no TTL bumps — see the ⚠ below
}
```

```bash
stellar contract invoke --id C… --source-account any --network mainnet -- \
  bucket_balance --asset C… --category Fuqara     # answered from simulation, never submitted
```

Read paths side by side:

| Read | Aptos | Soroban |
|---|---|---|
| Computed (runs code) | `/view` endpoint, `aptos move view` | `simulateTransaction`; CLI `invoke` answers read calls from simulation (`--send=no` forces); TS bindings resolve read calls from simulation, no `signAndSend` |
| Raw storage | `/accounts/{addr}/resource/{type}` | `getLedgerEntries` with the XDR-encoded contract-data key |
| History | indexer | `getEvents` (~7-day RPC window) + history archives — §7 |

**Breaks:**

- **No marker, no enforcement, no ABI signal.** Aptos at least gates the read endpoint on the attribute; the Soroban contract spec carries no read-only flag, so tooling distinguishes readers from writers only by simulating and inspecting the footprint. Purity is a review-enforced convention.
- ⚠ **FALSE FRIEND — side effects in getters silently don't happen.** Simulation discards writes, and nobody *submits* pure reads (why pay?). So `extend_ttl` inside a getter — the reflexive Soroban keep-alive idiom — never lands for view traffic: "read traffic keeps the state alive" is false. Same for lazy migration or access counters on read. Getters stay pure; TTL bumps live in mutating entry points plus the owned ops job (§3.5). (Aptos's API also discards view-path mutations, but Move norms never pushed writes into getters, so the instinct isn't there to betray you.)
- **Simulate-anything cuts both ways.** Recording-mode auth means anyone can dry-run *any* function — admin paths included — and see exactly what it would do, which auths it needs, what it costs. Superb pre-flight UX; never evidence of authorization. And auth-gating a getter hides nothing: every entry is publicly readable via `getLedgerEntries` regardless (true of Aptos resources too — neither chain has read ACLs).
- ∅ **Views depend on rent.** A read can hit an archived persistent entry (simulation answers with a `restorePreamble` — restore, then read) or an expired temporary one (absence, permanently). An Aptos resource is always readable; a Soroban answer is conditional on TTL — one more reason getters `unwrap_or(default)` where absence is meaningful.
- Simulation runs against the RPC node's recent ledger snapshot — slightly stale is possible; `getLedgerEntries` reads latest confirmed state.

Free simulation is worth designing *for*: a `solvency(asset) -> (booked, held)` getter lets anyone verify the §4.1 invariant at zero cost, with no account.

[Transaction simulation](https://developers.stellar.org/docs/learn/fundamentals/contract-development/contract-interactions/transaction-simulation) · [simulateTransaction deep dive](https://developers.stellar.org/docs/build/guides/transactions/simulateTransaction-Deep-Dive)

### 5.8 Dispatchable FAs (AIP-73) & dynamic dispatch → the only dispatch there is

**Move/Aptos** — static dispatch is the rule; AIP-73's Dispatchable Fungible Asset is the fenced exception. An issuer registers hooks **once, at asset creation**: `dispatchable_fungible_asset::register_dispatch_functions(&constructor_ref, withdraw, deposit, derived_balance)` (plus `register_derive_supply_dispatch_function`), each an `Option<FunctionInfo>` naming module address/name/function. Signatures are type-verified at registration; integrators must call the `dispatchable_fungible_asset::withdraw/deposit` wrappers (raw `fungible_asset::` aborts on hook-bearing tokens — `EINVALID_DISPATCHABLE_OPERATIONS`); the native dispatcher guards against re-entrant dispatch. APT itself bypasses dispatch, and the Confidential Asset standard flatly rejects dispatchable FAs.
[FA standard, DFA section](https://aptos.dev/build/smart-contracts/fungible-asset) · [AIP-73](https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-73.md) · [dispatchable_fungible_asset](https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-framework/doc/dispatchable_fungible_asset.md)

**Soroban** — the defaults invert: **dynamic dispatch is the only cross-contract mechanism there is** (§5.2). DFA's entire apparatus dissolves into it:

| AIP-73 concept | Soroban |
|---|---|
| Hook registration at creation | ∅ — **the token is the hook**: every DFA use case (tax/deflation, allowlist/KYC, predicated transfer, loyalty, rebasing `derived_balance`) is just an implementation of SEP-41 |
| `dispatchable_fungible_asset::withdraw/deposit` wrappers | `TokenClient` — always dispatching; no vanilla/dispatchable split to route around, no `EINVALID_DISPATCHABLE_OPERATIONS` analogue |
| `FunctionInfo` (addr, module, fn); fixed hook names/signatures | an `Address` — plus a runtime-chosen `Symbol` via `env.invoke_contract`, which is *more* dynamic than DFA allows |
| Signature verification at registration | ∅ ⚠ — conformance is checked only at call time, per call, as host decode errors |
| APT exempt from dispatch (trustable fast path) | the SAC: platform-fixed token code, **never hookable** — the mirror image (Aptos: framework token, issuer-hookable; Stellar: built-in asset contract, hookable by no one) |
| The dispatcher's reentrancy guard | the host's general reentrancy block (§6.3) — covers all calls, not just token dispatch |

```rust
// A "predicated-transfer" DFA on Soroban is not a hook — it's the token:
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    from.require_auth();
    require_allowlisted(&env, &to);          // the "hook", inline in the SEP-41 impl
    move_balance(&env, &from, &to, amount);  // + emit the standard transfer event (§7)
}
```

**Breaks:**

- ⚠ **FALSE FRIEND — every unknown token is DFA-grade.** Aptos partitions tokens into vanilla (framework code, trustable) and dispatchable (arbitrary code — even Aptos's own Confidential Asset refuses them). Soroban has no vanilla tier among custom contracts: any token address may tax transfers, lie in `balance()`, or gate recipients. The token-consumer checklist (§8.1) is the DFA-integrator posture applied to *everything* — except allowlisted SACs, whose behavior is fixed protocol code. That certainty is load-bearing: it is what lets a pool book `amount` on a SAC transfer without measuring the balance delta. Admit one non-SAC token to the allowlist and that shortcut dies — measure before/after instead.
- ⚠ **No registration step means no conformance check until the call.** `contractimport!` type-checks your call sites against the wasm you *built against*, not what is deployed at the address — an upgrade or governance repoint changes the callee under a still-compiling client, and the failure is a runtime `InvokeError`, not a deploy-time rejection. Rehearse repoints with fork tests against the actually deployed wasm (`stellar contract fetch`, §9); a timelock on repointing is what buys the window to do it.
- **Can't hook a SAC → stand in front of it.** DFA-style compliance hooks on USDC have exactly one shape on Stellar: a custody contract whose entry points wrap the transfers. The hook layer is enforced by *holding the funds*, not by dispatch registration.
- ∅ **No function values.** Nothing function-shaped crosses the ABI or enters storage; a "callback" is a stored `Address` implementing an expected interface. The platform's own hook points work exactly this way — `__check_auth` (§1.4) *is* Soroban's AIP-73: register a contract, the host natively dispatches into a fixed signature under special rules. For strategy variation inside one contract, prefer `enum` + `match` (static, cheap, auditable) over a `Map<Symbol, Address>` handler table: every dynamic hop is full invocation overhead and widens both the trust surface and the auth tree.

---

## 6. Execution semantics

### 6.1 Abort codes → `#[contracterror]`

```move
// Move
const E_INSUFFICIENT: u64 = 2;
assert!(bal >= amt, E_INSUFFICIENT);        // or: abort E_INSUFFICIENT
```

```rust
// Soroban
#[contracterror] #[derive(Copy, Clone, Debug, Eq, PartialEq)] #[repr(u32)]
pub enum Error { NotInitialized = 1, InsufficientBalance = 2, InvalidAmount = 3 }

pub fn distribute(…) -> Result<(), Error> {
    if bal < amt { return Err(Error::InsufficientBalance); }   // preferred: typed Result
    // or: panic_with_error!(&env, Error::InsufficientBalance);
    Ok(())
}
```

- ⚠ **FALSE FRIEND** — bare `panic!("msg")`, `unwrap()`, `expect()` surface as **opaque host errors** callers can't match on. Move's `abort` always carried your code; here only `panic_with_error!` / `Err` do. Ban `unwrap`/`expect` on any externally reachable path.
- Error codes are public ABI — never renumber across upgrades. With multiple contracts, partition now (pool 1xx, policy 2xx, attestation 3xx) so errors are attributable across `try_` boundaries.
- An `Err`/panic rolls back all state changes of the failing invocation, including its nested calls' writes.

[Move: abort & assert](https://aptos.dev/build/smart-contracts/book/abort-and-assert) · [docs.rs: contracterror](https://docs.rs/soroban-sdk/latest/soroban_sdk/attr.contracterror.html) · [errors example](https://developers.stellar.org/docs/build/smart-contracts/example-contracts)

### 6.2 Catchable callee failures ∅

Move has no try/catch: any abort anywhere kills the transaction. Soroban generated clients expose both `foo()` (panics on failure) and `try_foo()` returning a **doubly** nested `Result`. The shape depends on whether the contract fn returns `Result`, and the typed error lands in `Err(Ok(_))` — not `Ok(Err(_))`, which is the intuitive guess and is wrong:

```rust
// fn contribute(..) -> Result<(), Error>
let r: Result<Result<(), ConversionError>, Result<Error, InvokeError>>
     = client.try_contribute(&donor, &asset, &0);
assert_eq!(r, Err(Ok(Error::InvalidAmount)));   // ← your #[contracterror], verified

// fn bucket_balance(..) -> i128   (no Result)
let r2: Result<Result<i128, soroban_sdk::Error>, Result<soroban_sdk::Error, InvokeError>>
      = client.try_bucket_balance(&asset, &Category::Fuqara);   // Ok(Ok(0))
```

Read it as: outer `Err` = the call failed, and its payload distinguishes *your* typed error (`Ok(e)`) from a host-level failure (`Err(InvokeError)`: budget, bad auth, type mismatch). The inner `Ok` position is the conversion result for the return value. A caught callee failure rolls back **the callee's** writes; the caller continues with its own state intact. Budget exhaustion ends the whole transaction regardless — it is not catchable.

Design consequence with no Move reflex: a caller can *tolerate* a failing dependency (skip a broken oracle, degrade gracefully). Conversely ⚠: `try_` on a token transfer followed by continuing on `Err` is exactly the silent-accounting-error shape — on money paths, use the panicking client or make the `Err` arm abort. And note `TokenClient::transfer` returns `()` — "checking the transfer result" isn't a thing; a failed transfer *traps*, which is the guarantee.

### 6.3 Dispatch and reentrancy

**Move** — static dispatch; reentrancy structurally impossible; the compiler sees the whole call graph.
**Soroban** — dynamic dispatch by address (§5.2), but the **host blocks reentrancy** at runtime, direct and indirect, and there is no `delegatecall` (no foreign bytecode in your context — proxy hijacks don't exist). Same guarantee as Move, enforced one layer later. Still write checks-effects-interactions: external calls remain failure/budget/side-effect boundaries, callees are arbitrary code (§5.2), and the pattern survives a platform change the assumption wouldn't.

### 6.4 Gas → declared resources, and hard ceilings

**Move** — one gas meter, pay-as-you-go, plus a refundable storage deposit; no pre-declaration.
[Aptos: gas & txn fees](https://aptos.dev/network/blockchain/gas-txn-fee)

**Soroban** — multidimensional and **declared up front**: CPU instructions, ledger entries + bytes read/written, tx size, events size, rent — computed by simulation, embedded in the tx; exceed your declaration and the tx fails (rent/events refundable if unused; instructions/IO charged as declared). Per-transaction mainnet ceilings, from `InvocationResourceLimits::mainnet()` in SDK 27.0.6 (network-configured — check live): 400M instructions · 40MiB memory · 200 disk-read entries / 200 write entries / **400 ledger entries total** · 200KB read / 132,096B written · 65,536B per data entry (the *whole* instance-storage entry included; keys ≤250B) · 16,384B events + return value · 131,072B contract code.

What actually moves the needle: minimize distinct entries touched (fixed cost each, and footprint members cost whether used or not); never loop over user-controlled collection sizes (budget DoS / fee-griefing — cap explicitly); events instead of storage for off-chain-only data; bound signature counts in `__check_auth`. Profile with `stellar contract invoke … --send=no` or `env.cost_estimate().resources()` in tests.
[Resource limits & fees](https://developers.stellar.org/docs/networks/resource-limits-fees)

### 6.5 Environment quick hits

- `timestamp::now_seconds()` 🔧 → `env.ledger().timestamp()` (`u64` seconds); ledger sequence via `env.ledger().sequence()`.
- ⚠ Aptos has a secure on-chain randomness API; `env.prng()` is seeded from ledger state and **predictable** — never for auth, keys, or stakes.
- No I/O, no networking, no floats, no clock beyond the ledger. Everything reaches the world through `Env`.

---

## 7. Events

**Move** — `#[event]` struct + `event::emit(e)`; indexers consume typed module events; events persist in the indexer's history.
[Aptos framework: event](https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-framework/doc/event.md)

**Soroban** —

```rust
#[contractevent]                      // topics: ("contribution", donor); data: {asset, amount, split}
pub struct Contribution {
    #[topic] pub donor: Address,
    pub asset: Address, pub amount: i128, pub split: BucketSet,
}
Contribution { donor, asset, amount, split }.publish(&env);
```

- Structure is split and load-bearing: `#[topic]` fields (plus the snake-cased struct name, first by default) are what indexers **filter on**; the rest is the data payload. Participant addresses belong in topics.
- 🔧 the older `env.events().publish(topics, data)` is deprecated — use the macro.
- ⚠ **FALSE FRIEND** — events are **ephemeral at the RPC layer**: typical retention ~7 days; full history lives in Stellar's history archives. An "events + indexer" record design must ship the archive-ingestion path, and anything needed on-chain long-term must be derivable from state, not events. Your event schema is ABI for the indexer — version it deliberately.
- Diagnostic `log!` output is off by default on nodes and outside consensus — never logic-bearing.
- Emitting is far cheaper than storing: the correct home for audit-trail data (per-contribution records) that contracts never re-read.
- Token movements must emit the **standard SEP-41 event shapes** exactly (`transfer`/`mint`/`burn`/`approve`/`clawback`) — wallets and indexers depend on them; the SAC emits them for you.

[Events guide](https://developers.stellar.org/docs/build/guides/events) · [docs.rs: contractevent](https://docs.rs/soroban-sdk/latest/soroban_sdk/attr.contractevent.html)

### 7.1 Reading events on-chain → ∅ on **both** chains

Symmetric dead end, and the symmetry is the useful fact: **no contract on either platform can read emitted events during execution.** Aptos states the mechanism plainly — events live in a per-transaction event accumulator, a separate merkle tree that is ephemeral and independent of the state tree, so the MoveVM has no read access to them in production. Soroban is the same by construction: events land in transaction *meta*, never in ledger state; `Env` exposes `publish` and no reader. Both are write-only sinks feeding indexers.

Both do expose event reading **in tests only**, and they line up almost exactly:

| | Aptos (test-only) | Soroban (`testutils`) |
|---|---|---|
| All events of a type | `event::emitted_events<T>(): vector<T>` | `env.events().all()` → `ContractEvents` (most recent invocation only) |
| Assert one was emitted | `event::was_event_emitted<T>(&e): bool` | compare against `e.to_xdr(&env, &contract_id)` |

```rust
use soroban_sdk::{testutils::Events as _, Event as _};
let expected = Contribution { donor, asset, amount: 1_000, split };
// ContractEvents impls PartialEq<std::vec::Vec<xdr::ContractEvent>>, so compare directly.
// filter_by_contract is usually needed: a token transfer emits the SAC's event too.
assert_eq!(
    env.events().all().filter_by_contract(&contract_id),
    std::vec![expected.to_xdr(&env, &contract_id)],
);
// Raw slice if you need to inspect: env.events().all().events()  — note: no .len(), no .iter()
```

⚠ Note the Soroban asymmetry: `env.events().all()` covers **only the most recent invocation** and resets on the next client call (like `env.auths()`, §9), whereas `emitted_events<T>()` accumulates across the test. Assert immediately after the call under test.

**The design rule this forces (same on both):** anything a contract must *decide on* has to be state; events are for parties outside the VM. If a value is load-bearing for contract logic, store it — and if the audit trail must also carry it, emit it too and accept the duplication. Concretely: your `distribution_seq`, category balances, and pending-governance-change record are storage precisely because `distribute` and `contribute` branch on them; the per-contribution record is events-only precisely because no contract ever reads it back. That split is correct and is the same split you'd draw in Move.

⚠ **FALSE FRIEND — the indexer is not a read path.** "The indexer reads our events and feeds a total back in" turns a convenience service into a trusted oracle: it becomes a signer whose word moves money, with all of §5.2's untrusted-callee problems plus off-chain compromise. If a contract needs a running total, maintain it in storage as you go. Reserve the event trail for reconstruction *by humans and services*, where you already have it right.

*(If you're thinking of a recent Aptos AIP here: AIP-44 Module Events replaced `EventHandle` streams with `#[event]` struct-typed module events — better emission, indexing, and parallelism, plus the test-only readers above. It did not grant contracts read access, and no AIP has.)*

[Aptos: events](https://aptos.dev/network/blockchain/events) · [AIP-44 Module Events](https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-44.md) · [aptos_framework::event](https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-framework/doc/event.md)

---

## 8. Tokens

### 8.1 Fungible: Coin/FA → SEP-41 + SAC

**Move** — assets are typed values (`Coin<T>`) or FA objects (`Metadata` + `FungibleStore`s) living in *holders'* storage; your code holds and moves the value itself (§4.2).
[Aptos: fungible asset](https://aptos.dev/build/smart-contracts/fungible-asset) · [standards overview](https://aptos.dev/build/smart-contracts/aptos-standards)

**Soroban** — a token is an external contract implementing **SEP-41**; balances live inside it; you call it (§4.2). Three things may stand behind one "token address": a custom contract, native XLM's SAC, or a classic Stellar asset's **Stellar Asset Contract** — a built-in SEP-41 wrapper at a deterministic per-asset address. Circle's USDC on Stellar is a classic asset ⇒ your custody path *is* the SAC path.

```rust
use soroban_sdk::token::{TokenClient, StellarAssetClient};
let t = TokenClient::new(&env, &asset);          // SEP-41: balance/transfer/approve/burn…
t.transfer(&from, &to, &amount);
let sac = StellarAssetClient::new(&env, &asset); // SAC admin surface: mint/clawback/set_admin…
```

What has no Move reflex behind it:

- **Auth pattern**: `transfer`/`approve`/`burn` auth `from`; `transfer_from`/`burn_from` auth the **spender** (the allowance pre-authorized the `from` side). Reads need no auth.
- **Allowances expire**: `approve(from, spender, amount, expiration_ledger)`. Re-approving overwrites → the classic front-run race (spend old + new); approve-to-0 first, or prefer exact-amount auth (`require_auth_for_args`) over standing allowances.
- ⚠ **FALSE FRIEND — the SAC drags classic-asset semantics into your contract.** Account (`G…`) balances live in **trustlines**: recipient missing one, or unauthorized under `AUTH_REQUIRED`, ⇒ the transfer **traps** (also: trustline balances cap at i64). Issuer flags apply — freezes (`AUTH_REVOCABLE`), clawback. Transfers *to* the issuer burn; *from* the issuer mint. Contract (`C…`) balances live in contract storage with full `i128` and no trustline. A distribution path paying `G…` recipients must treat "recipient not ready" as an expected failure mode, not an anomaly.
- Amounts `i128` and signed — §4.5. Decimals: query per token at registration, never assume 7. `transfer`'s destination is a `MuxedAddress` (plain `Address` converts) for exchange sub-account IDs.

[SEP-41](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md) · [Token interface](https://developers.stellar.org/docs/tokens/token-interface) · [SAC](https://developers.stellar.org/docs/tokens/stellar-asset-contract)

### 8.2 Non-fungible: Digital Assets → SEP-50

**Move/Aptos** — the **Digital Asset** standard, built directly on the object model (§2.2): a `Collection` object plus `Token` objects, each token holding a reference to its parent collection (the collection does *not* own the token). Authority is capability refs minted **at creation and only then** — `MutatorRef` (edit description/URI), `BurnRef`, `TransferRef` — royalties configurable per collection or per token, `mint_soul_bound` for untransferable tokens, and NFTs that can own NFTs. It supersedes legacy Token v1, whose transfers required recipient opt-in.
[Aptos: digital asset](https://aptos.dev/build/smart-contracts/digital-asset) · [aptos-token-objects](https://aptos.dev/build/smart-contracts/aptos-standards)

**Soroban** — **SEP-50**, an ERC-721-shaped contract interface: `balance` / `owner_of` / `transfer` / `transfer_from` / `approve` / `approve_for_all` / `get_approved` / `is_approved_for_all`, plus `name` / `symbol` / `token_uri`. It deviates from ERC-721 deliberately: `transfer()` is added (owner-initiated transfers are the common case and cheaper than the approve-then-pull dance), `safeTransferFrom` is dropped (auth entries already cover the class of mistake it guarded), and `token_id` is format-agnostic — sequential, UUID, or hash.

Where the analogy breaks:

- ⚠ **FALSE FRIEND — there is no SAC for NFTs.** SEP-41's saving grace is that the most important fungible tokens are SACs: fixed protocol code, behaviour you can *assume* (§5.8). SEP-50 has no protocol-level implementation at all. Every NFT you touch is arbitrary contract code that may lie in `owner_of`, charge on transfer, or refuse recipients. The §5.8 "every unknown token is DFA-grade" posture applies here with **no** trustable tier to fall back on.
- ⚠ **SEP-50 is a Draft (0.1.0, March 2025), not a settled standard** — unlike SEP-41. Authored by OpenZeppelin with Boyan Barakov and Özgün Özerk. Treat the interface as likely-stable rather than frozen, and pin the implementation you integrated against.
- ∅ **Refs don't port** — `MutatorRef`/`BurnRef`/`TransferRef` are capability values, and §2.1 already settled that: no scarce values, so authority becomes role `Address`es in storage plus `require_auth`. "Mint-time only" scarcity becomes "set once in `__constructor`, never expose a setter" (§5.4).
- ∅ **NFTs owning NFTs** has no substrate (§2.2). An owner is an `Address`; since a contract address is an `Address`, composition is a cross-contract relationship you design, not resource attachment you get.
- ⚠ **Every token is a rent liability.** Each NFT's ownership entry is a persistent ledger entry with its own TTL (§3.5), so a 10,000-piece collection is 10,000 entries someone must keep alive — an operational cost that scales with supply and has no Aptos analogue, where state persists once paid. This is the consideration that should drive variant choice, not gas.
- **Implementation**: OpenZeppelin's `stellar-contracts` non-fungible module — **Base** (most cases), **Consecutive** (batch minting; stores ownership only at range boundaries and infers the rest — materially cheaper for large drops), **Enumerable** (on-chain enumeration, which you pay for in entries), plus `Burnable`, `Royalties`, and `Votes` extensions. The variants override each other's internals, so mixing them freely is explicitly discouraged.
- The pre-Soroban path still exists: **SEP-39** NFTs as classic assets with supply 1 and off-chain metadata. Cheap and needs no contract, but the "fungibility" is nominal and metadata lives off-chain — fine for tickets and badges, wrong for anything wanting on-chain behaviour.

[SEP-50](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0050.md) · [OpenZeppelin non-fungible](https://docs.openzeppelin.com/stellar-contracts/tokens/non-fungible/non-fungible) · [NFT example contract](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/non-fungible-token)

---

## 9. Testing & assurance

**Move** — `#[test(admin = @0x1)]` injects real signers; `#[expected_failure(abort_code = …)]`; `aptos move test`; the Prover for specs.
[Move: unit testing](https://aptos.dev/build/smart-contracts/book/unit-testing) · [Move Prover](https://aptos.dev/build/smart-contracts/prover)

**Soroban** — native Rust tests against a full in-process host (`extern crate std;` is fine in tests):

```rust
let env = Env::default();
env.mock_all_auths();
let admin = Address::generate(&env);
let id = env.register(ZakatPool, (admin.clone(),));   // tuple = __constructor args
let client = ZakatPoolClient::new(&env, &id);
```

The differential traps:

- ⚠ **FALSE FRIEND** — `mock_all_auths()` approves *everything*, so it **hides missing `require_auth`** — the §1.1 bug ships green under it. Always pair with assertions, immediately after the call under test:

  ```rust
  client.contribute(&donor, &usdc, &1_000);
  let auths = env.auths();                 // most-recent invocation ONLY — resets on the next
  assert_eq!(auths[0].0, donor);           // client call, including reads
  ```

  Assert the full tree (fn, args, sub-invocations) on money paths, or use `mock_auths(&[MockAuth{…}])` to approve one specific invocation. `env.events().all()` has the same reset-per-call behavior. Contracts using `authorize_as_current_contract` need `mock_all_auths_allowing_non_root_auth()`.
- `expected_failure` → assert on `try_` results' typed-error position (§6.2); no attribute equivalent.
- Time/ledger: `env.ledger().set_timestamp(t)` / `set_sequence_number(n)` — set, act, advance, act.
- TTL is testable, but `get_ttl` lives on traits you must import by name — there is no `testutils::Storage` umbrella:

  ```rust
  use soroban_sdk::testutils::storage::{Instance as _, Persistent as _};
  env.as_contract(&id, || env.storage().persistent().get_ttl(&key));   // instance(): get_ttl() takes no key
  ```

- Cross-contract against **real Wasm**: `env.register(other::WASM, args)` after `contractimport!` — test the actual binary, not a mock.
- **Mainnet resource limits are enforced in unit tests by default** (`InvocationResourceLimits::mainnet()`): a budget-blowing function panics in `cargo test`, naming the limit exceeded, before it fails in production. Read usage with `env.cost_estimate().resources()` → `InvocationResources { instructions, mem_bytes, disk_read_entries, memory_read_entries, write_entries, disk_read_bytes, write_bytes, contract_events_size_bytes, persistent_rent_ledger_bytes, … }`. Override with `env.cost_estimate().enforce_resource_limits(limits)`, or `env.cost_estimate().disable_resource_limits()` for deliberately heavy experiments.
- The default assurance stack is **fuzz → property → mutation → fork**: `#[contracttype]` types implement `SorobanArbitrary` (cargo-fuzz works out of the box on `try_contribute`/`try_distribute` with the solvency invariant as oracle); `proptest` locks findings into CI; `cargo mutants` finds untested code; `Env::from_ledger_snapshot_file` replays real network state (`stellar snapshot create`) — including upgrade rehearsals against the deployed Wasm (`stellar contract fetch`). Static analysis: CoinFabrik Scout (`cargo scout-audit`).

**Formal verification is the one real regression, and it's a tooling gap rather than a semantic one.** ⚠ Do not read the stack above as a Prover substitute: fuzzing samples the input space, the Prover discharges proof obligations over *all* inputs. Different guarantees, and the difference is exactly what you'd reach for the Prover to get.

| | Move Prover | Soroban |
|---|---|---|
| Provenance | first-party, ships with the Aptos toolchain (`aptos move prove`) | none first-party — two third-party tools |
| Specs live | in-language `spec {}` blocks beside the code, versioned with it | outside the language: Certora **Sunbeam** (`#[rule]` fns using the CVLR macro library — `cvlr_assert!`, `cvlr_assume!`, `cvlr_satisfy!`); Runtime Verification **Komet** (property tests written as Rust test *contracts*) |
| Verifies | Move bytecode, via Boogie/Z3 | Wasm bytecode — Sunbeam via SMT, Komet via K/KWasm symbolic execution |
| Maturity | mature, widely used on the framework itself | Sunbeam used in production audits (e.g. Blend); Komet SCF-funded and newer. Both hit timeouts/`Unknown` on complex functions; Sunbeam's invariant and parameterized-rule support is still thinner than CVL's on EVM |

Verifying Wasm has one structural advantage over verifying Move bytecode: Soroban's guest/host split moves most complexity into the host, so contract code is small and self-contained, and verification reduces to the contract *given* a correct host. Practical read for a solvency invariant: express it as a Sunbeam rule if you want a proof, and keep the fuzz harness regardless — it's cheap, it runs in CI, and it catches the same class of bug faster during development.

- Test snapshots (`test_snapshots/` JSON of events + final state) — commit them; diffs expose unintended behavior changes.

[Fuzzing](https://developers.stellar.org/docs/build/guides/testing/fuzzing) · [Fork testing](https://developers.stellar.org/docs/build/guides/testing/fork-testing) · [Mutation](https://developers.stellar.org/docs/build/guides/testing/mutation-testing) · [Snapshots](https://developers.stellar.org/docs/build/guides/testing/differential-tests-with-test-snapshots) · [Certora Sunbeam](https://docs.certora.com/en/latest/docs/sunbeam/index.html) · [Komet](https://docs.runtimeverification.com/komet) · [Move Prover](https://aptos.dev/build/smart-contracts/prover)

---

## 10. Toolchain quick map

| | Move (Aptos) | Soroban |
|---|---|---|
| Build | `aptos move compile` | `stellar contract build` (→ `target/wasm32v1-none/release/*.wasm`, optimized) |
| Test | `aptos move test` | `cargo test` (native host, debugger-friendly) |
| Publish | `aptos move publish` | `stellar contract deploy` (instance) / `upload` (code only) |
| Call | `aptos move run` | `stellar contract invoke --id C… -- fn --arg v` |
| Dry-run/profile | simulation API | `stellar contract invoke … --send=no` (instructions, IO, fees) |
| Manifest | `Move.toml`, named addresses | `Cargo.toml` — `crate-type = ["lib","cdylib"]`, `overflow-checks = true` mandatory in release |
| Target | Move bytecode | Wasm, `wasm32v1-none` (Rust ≥1.84), **128KB** limit (`cargo bloat` when over) |
| Local net | `aptos node run-local-testnet` | `stellar container start local` (RPC `localhost:8000/soroban/rpc`) |
| Keys | profiles | `stellar keys generate alice --network testnet --fund` |
| Testnet | persistent | **resets quarterly** — script every deployment; runs the *next* protocol before mainnet, so it's your upgrade rehearsal |
| Explorer | Aptos Explorer | [Stellar Lab](https://lab.stellar.org) — state, TTLs, archived-entry restore, live network limits |

[Stellar CLI manual](https://developers.stellar.org/docs/tools/cli/stellar-cli)

---

## 11. Porting reflexes — the grep-able checklist

1. Every `Address` parameter → an explicit auth decision; privileged paths auth a **stored** role address, never the parameter.
2. Every layer that exercises an address's authority calls `require_auth` — including the outermost one (middleware replay).
3. `get` + mutate is banned; read-modify-write goes through `update`.
4. `checked_*` on every money path; `amount > 0` at every amount-taking entry point (`i128` is signed).
5. No `unwrap`/`expect`/bare `panic!` on externally reachable paths — `panic_with_error!` or `Err` only; error codes frozen forever.
6. Nothing load-bearing in `temporary()`; TTL is never a permission; deadlines live in values.
7. Fine-grained storage keys; no per-tx iteration over user-controlled collection sizes; instance storage stays small and global.
8. Every user-supplied contract address is allowlisted; every callee return value is validated (length, sign, sum, freshness).
9. `extend_ttl` in hot paths **and** an owned, runbooked keep-alive job — max TTL makes liveness an operational property.
10. `mock_all_auths` never appears without an `env.auths()` assertion on the same invocation.
11. Solvency (and every conservation law the type system used to give you) is asserted in code and is the fuzz oracle.
12. One upgrade path per contract: either a gated `update_current_contract_wasm` or a repointable address in a caller — never both, ideally neither on custody.
13. Getters are pure — no writes, no TTL bumps: view traffic is simulation, and simulation discards writes.
14. Nothing the contract branches on lives only in events — events are write-only in production; if logic reads it, store it.

---

## 12. Verification

Every Soroban snippet in this document exists as compiling code under [`verify/`](verify), and
every behavioural claim is an assertion rather than prose. CI runs it on push and weekly, so
SDK drift shows up as a red build instead of a quietly stale doc.

```bash
cd verify && cargo test
```

`src/lib.rs` holds contract-side code (storage, auth, events, errors, TTL, tokens);
`src/advanced.rs` holds auth trees, the deployer, and the custom account; `tests/` holds the
runtime assertions — auth/event reset behaviour, `try_` nesting, TTL reads, cost estimates,
`contractimport!` against a real Wasm binary.

Four claims in the first draft of this document were wrong and were caught by compiling them.
They are listed here because the *shape* of the errors is instructive — all four are places
where the plausible guess is wrong:

| Claim as first written | Actual (SDK 27.0.6) |
|---|---|
| `try_` typed error at `Ok(Err(e))` | `Err(Ok(e))` — outer `Err` means "call failed"; its payload splits your error from host failure |
| `__check_auth(signature_payload: BytesN<32>, …)` | `crypto::Hash<32>` — host-constructed, unforgeable in safe code |
| `get_ttl` via "testutils traits" | needs `use soroban_sdk::testutils::storage::{Instance as _, Persistent as _}`; there is no `testutils::Storage` |
| `assert_eq!(env.events().all(), vec![…])` | needs `.filter_by_contract(&id)` — a token transfer also emits the SAC's own event |

Move snippets are illustrative and are not compiled.

---

## 13. Primary sources

**Soroban** — [skills.stellar.org / smart-contracts](https://skills.stellar.org/skills/smart-contracts/SKILL.md) (+ its [development](https://skills.stellar.org/skills/smart-contracts/development.md), [testing](https://skills.stellar.org/skills/smart-contracts/testing.md), [security](https://skills.stellar.org/skills/smart-contracts/security.md) deep-dives) · [developer docs](https://developers.stellar.org/docs/build/smart-contracts) · [soroban-examples](https://github.com/stellar/soroban-examples) · [docs.rs/soroban-sdk](https://docs.rs/soroban-sdk) · [example contracts index](https://developers.stellar.org/docs/build/smart-contracts/example-contracts)

**Move** — [Move Book on Aptos](https://aptos.dev/build/smart-contracts/book/summary) · [smart-contracts hub](https://aptos.dev/build/smart-contracts) · [security guidelines](https://aptos.dev/build/smart-contracts/move-security-guidelines)

*Link note: developers.stellar.org and aptos.dev/build/smart-contracts/book/* paths verified Aug 2026; a handful of Aptos-specific deep links (prover, gas, randomness) follow the current nav structure but reorganize occasionally — one hop from the hub pages above if any has moved.*

---

## Contributing

Corrections are the most valuable contribution here — a wrong mapping is worse than a missing
one. Missing Move concepts are equally welcome as issues even without an answer. See
[CONTRIBUTING.md](CONTRIBUTING.md); the one hard rule is that Soroban snippets must compile.