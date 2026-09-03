# 5. Modules vs. contracts

## 5.1 Module → Wasm hash + contract instance

**Move**, publishing puts module bytes at an address; code identity ≡ address; one deployment per address.
[Move: modules & scripts](https://aptos.dev/build/smart-contracts/book/modules-and-scripts) · [packages](https://aptos.dev/build/smart-contracts/book/packages)

**Soroban**, two-level, content-addressed: **upload** Wasm once (identified by hash, deduplicated network-wide), then **deploy** instances (address + wasm-hash pointer + instance storage). Many instances per binary, ∅ in Aptos Move, and the substrate for factories (§2.2). `stellar contract upload` for code-only (feeds factories and upgrades); `stellar contract deploy` for an instance. Addresses: accounts `G…`, contracts `C…`, both unify as `Address`.
[Getting started: deploy](https://developers.stellar.org/docs/build/smart-contracts/getting-started)

## 5.2 Cross-contract: compile-time linking → runtime dispatch ⚠ FALSE FRIEND

**Move**, `use other_addr::vault;` is static linking: callee identity fixed at compile time, verified at publish, dispatch static.

**Soroban**, every cross-contract call is dynamic dispatch to an `Address` you hold *at runtime*, usually from your own storage:

```rust
mod policy_contract { soroban_sdk::contractimport!(file = "policy.wasm"); }  // typed client from wasm

let policy: Address = env.storage().instance().get(&DataKey::PolicyAddr).unwrap();
let split = policy_contract::Client::new(&env, &policy).allocate(&amount);

// SEP-41 tokens need no import; the SDK ships the client:
let t = soroban_sdk::token::TokenClient::new(&env, &asset);

// Unknown interface at compile time:
let x: i128 = env.invoke_contract(&target, &symbol, args);
```

- ⚠ **FALSE FRIEND**, a callee address is **arbitrary code** wearing an interface. It can lie, trap, burn budget, and emit misleading events (it cannot re-enter you, §6.3). Any user-supplied contract address must be allowlisted (`approved_assets`), and any callee return value must be validated, sum/range/length-check the allocation, freshness-check the oracle. "Untrusted collaborator" is the correct posture even for contracts you deployed, since governance can repoint them.
- The pinned-address-in-storage pattern is also your inter-contract *upgrade* mechanism (§5.5).

[Cross-contract example](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/cross-contract-call) · [docs.rs: contractimport](https://docs.rs/soroban-sdk/latest/soroban_sdk/macro.contractimport.html)

## 5.3 Visibility: one flat exported namespace

`entry` vs `public` vs private → gone. Every `pub fn` in `#[contractimpl]` is simultaneously the external entry point and the cross-contract interface; there is no "callable by transactions but not by contracts" or vice versa. Access control is done with auth (§1.5), never with visibility, because there is none to do it with.
[Move: functions](https://aptos.dev/build/smart-contracts/book/functions)

## 5.4 `init_module` → `__constructor` 🔧 (and strictly better)

**Move**, `init_module(&signer)` runs at publish, takes only the publisher's signer; real parameterization needs a follow-up call.

**Soroban**, `__constructor(env, args…)` (exact name) runs **once, atomically, at deploy, with arbitrary typed args** passed after `--` in the deploy command. Failure aborts the deployment; it never re-runs (not on upgrade). This kills the deploy-then-`initialize` front-running window; prefer it always. A contract deployed without one can't gain one retroactively; if forced into a guarded `initialize` (legacy), check-and-set an `Initialized` flag or anyone can capture admin.

## 5.5 Upgrade policy → compiled-in upgrade fn

**Move**, policy declared in the manifest: `compatible` (layout/signature-compatible upgrades) or `immutable`.
[Move: package upgrades](https://aptos.dev/build/smart-contracts/book/package-upgrades)

**Soroban**, no manifest, no compatibility checker. A contract is mutable **iff** it exports a function calling `env.deployer().update_current_contract_wasm(new_hash)`; gate it with admin auth. **Immutability = omission**: no such call site anywhere, no other path exists (no delegatecall). Address and all storage survive an upgrade; `__constructor` does not re-run.

⚠ **FALSE FRIEND**, there is no `compatible`-policy safety net: new code reading an old key whose `#[contracttype]` shape changed **fails to decode at runtime**. Schema migration is entirely on you: store a schema version, add enum variants rather than reshaping, ship an idempotent admin-gated `migrate` enforcing `new > current`. And error codes are ABI; never renumber (§6.1). The three-contract split gives the complementary mechanism: repoint a stored dependency address (§5.2) instead of mutating code; a contract should have one upgrade path, not both.
[Upgrading contracts](https://developers.stellar.org/docs/build/guides/conventions/upgrading-contracts)

## 5.6 Transaction shape: one invocation per transaction ∅

**Move**, a script sequences arbitrary calls atomically; entry fns compose framework calls freely.
[Move: scripts](https://aptos.dev/build/smart-contracts/scripts/writing-scripts)

**Soroban**, a transaction carries **exactly one** contract invocation (one `InvokeHostFunction` op; Soroban txs are single-operation). There is no client-side multicall. Atomic multi-step = a contract function making the calls (a periphery/router contract if the steps span protocols). Anything your docs describe as "then the client calls X and Y" is actually one contract function or two transactions; decide which.

Adjacent quick hit: Aptos fee-payer/sponsored txs ↔ Stellar **fee-bump transactions** (fees) + **sponsored reserves** (account/trustline reserves), protocol features, not contract code. [Sponsored reserves](https://developers.stellar.org/docs/learn/encyclopedia/transactions-specialized/sponsored-reserves)

## 5.7 `#[view]` functions → any function, invoked in simulation

**Move/Aptos**, `#[view]` is an opt-in marker gating the node's free read path: only tagged fns are callable through the fullnode `/view` endpoint or `aptos move view` (calling an untagged fn there fails). No signer, no gas; the API discards any state mutation (`#[view]` is *not* compile-time purity; the read path just drops writes). It exists partly because entry-fn return values are inaccessible to the submitter, so computed reads need a dedicated surface.
[Aptos: fullnode REST API & view functions](https://aptos.dev/build/apis/fullnode-rest-api)

**Soroban**, ∅ attribute, and none is needed: **"view" is a call mode, not a function property.** Any `pub fn` can be executed through RPC `simulateTransaction`, no signature (auth runs in *recording* mode: `require_auth` is noted, never enforced), no fee, execution effects discarded, return value in the simulation result. Return values are also accessible from *submitted* txs, so the entry/view split never existed: one `bucket_balance(asset, category)` serves cross-contract callers, off-chain readers, and transactions alike.

```rust
pub fn bucket_balance(env: Env, asset: Address, category: Category) -> i128 {
    env.storage().persistent()
        .get::<_, BucketSet>(&DataKey::Buckets(asset))
        .unwrap_or_default()
        .get(category)          // pure: no writes, no TTL bumps, see the ⚠ below
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
| History | indexer | `getEvents` (~7-day RPC window) + history archives, §7 |

**Breaks:**

- **No marker, no enforcement, no ABI signal.** Aptos at least gates the read endpoint on the attribute; the Soroban contract spec carries no read-only flag, so tooling distinguishes readers from writers only by simulating and inspecting the footprint. Purity is a review-enforced convention.
- ⚠ **FALSE FRIEND, side effects in getters silently don't happen.** Simulation discards writes, and nobody *submits* pure reads (why pay?). So `extend_ttl` inside a getter, the reflexive Soroban keep-alive idiom, never lands for view traffic: "read traffic keeps the state alive" is false. Same for lazy migration or access counters on read. Getters stay pure; TTL bumps live in mutating entry points plus the owned ops job (§3.5). (Aptos's API also discards view-path mutations, but Move norms never pushed writes into getters, so the instinct isn't there to betray you.)
- **Simulate-anything cuts both ways.** Recording-mode auth means anyone can dry-run *any* function, admin paths included, and see exactly what it would do, which auths it needs, what it costs. Superb pre-flight UX; never evidence of authorization. And auth-gating a getter hides nothing: every entry is publicly readable via `getLedgerEntries` regardless (true of Aptos resources too, neither chain has read ACLs).
- ∅ **Views depend on rent.** A read can hit an archived persistent entry (simulation answers with a `restorePreamble`, restore, then read) or an expired temporary one (absence, permanently). An Aptos resource is always readable; a Soroban answer is conditional on TTL; one more reason getters `unwrap_or(default)` where absence is meaningful.
- Simulation runs against the RPC node's recent ledger snapshot, slightly stale is possible; `getLedgerEntries` reads latest confirmed state.

Free simulation is worth designing *for*: a `solvency(asset) -> (booked, held)` getter lets anyone verify the §4.1 invariant at zero cost, with no account.

[Transaction simulation](https://developers.stellar.org/docs/learn/fundamentals/contract-development/contract-interactions/transaction-simulation) · [simulateTransaction deep dive](https://developers.stellar.org/docs/build/guides/transactions/simulateTransaction-Deep-Dive)

## 5.8 Dispatchable FAs (AIP-73) & dynamic dispatch → the only dispatch there is

**Move/Aptos**, static dispatch is the rule; AIP-73's Dispatchable Fungible Asset is the fenced exception. An issuer registers hooks **once, at asset creation**: `dispatchable_fungible_asset::register_dispatch_functions(&constructor_ref, withdraw, deposit, derived_balance)` (plus `register_derive_supply_dispatch_function`), each an `Option<FunctionInfo>` naming module address/name/function. Signatures are type-verified at registration; integrators must call the `dispatchable_fungible_asset::withdraw/deposit` wrappers (raw `fungible_asset::` aborts on hook-bearing tokens, `EINVALID_DISPATCHABLE_OPERATIONS`); the native dispatcher guards against re-entrant dispatch. APT itself bypasses dispatch, and the Confidential Asset standard flatly rejects dispatchable FAs.
[FA standard, DFA section](https://aptos.dev/build/smart-contracts/fungible-asset) · [AIP-73](https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-73.md) · [dispatchable_fungible_asset](https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-framework/doc/dispatchable_fungible_asset.md)

**Soroban**, the defaults invert: **dynamic dispatch is the only cross-contract mechanism there is** (§5.2). DFA's entire apparatus dissolves into it:

| AIP-73 concept | Soroban |
|---|---|
| Hook registration at creation | ∅, **the token is the hook**: every DFA use case (tax/deflation, allowlist/KYC, predicated transfer, loyalty, rebasing `derived_balance`) is just an implementation of SEP-41 |
| `dispatchable_fungible_asset::withdraw/deposit` wrappers | `TokenClient`, always dispatching; no vanilla/dispatchable split to route around, no `EINVALID_DISPATCHABLE_OPERATIONS` analogue |
| `FunctionInfo` (addr, module, fn); fixed hook names/signatures | an `Address`, plus a runtime-chosen `Symbol` via `env.invoke_contract`, which is *more* dynamic than DFA allows |
| Signature verification at registration | ∅ ⚠, conformance is checked only at call time, per call, as host decode errors |
| APT exempt from dispatch (trustable fast path) | the SAC: platform-fixed token code, **never hookable**, the mirror image (Aptos: framework token, issuer-hookable; Stellar: built-in asset contract, hookable by no one) |
| The dispatcher's reentrancy guard | the host's general reentrancy block (§6.3), covers all calls, not just token dispatch |

```rust
// A "predicated-transfer" DFA on Soroban is not a hook; it's the token:
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    from.require_auth();
    require_allowlisted(&env, &to);          // the "hook", inline in the SEP-41 impl
    move_balance(&env, &from, &to, amount);  // + emit the standard transfer event (§7)
}
```

**Breaks:**

- ⚠ **FALSE FRIEND, every unknown token is DFA-grade.** Aptos partitions tokens into vanilla (framework code, trustable) and dispatchable (arbitrary code; even Aptos's own Confidential Asset refuses them). Soroban has no vanilla tier among custom contracts: any token address may tax transfers, lie in `balance()`, or gate recipients. The token-consumer checklist (§8.1) is the DFA-integrator posture applied to *everything*, except allowlisted SACs, whose behavior is fixed protocol code. That certainty is load-bearing: it is what lets a pool book `amount` on a SAC transfer without measuring the balance delta. Admit one non-SAC token to the allowlist and that shortcut dies; measure before/after instead.
- ⚠ **No registration step means no conformance check until the call.** `contractimport!` type-checks your call sites against the wasm you *built against*, not what is deployed at the address, an upgrade or governance repoint changes the callee under a still-compiling client, and the failure is a runtime `InvokeError`, not a deploy-time rejection. Rehearse repoints with fork tests against the actually deployed wasm (`stellar contract fetch`, §9); a timelock on repointing is what buys the window to do it.
- **Can't hook a SAC → stand in front of it.** DFA-style compliance hooks on USDC have exactly one shape on Stellar: a custody contract whose entry points wrap the transfers. The hook layer is enforced by *holding the funds*, not by dispatch registration.
- ∅ **No function values.** Nothing function-shaped crosses the ABI or enters storage; a "callback" is a stored `Address` implementing an expected interface. The platform's own hook points work exactly this way, `__check_auth` (§1.4) *is* Soroban's AIP-73: register a contract, the host natively dispatches into a fixed signature under special rules. For strategy variation inside one contract, prefer `enum` + `match` (static, cheap, auditable) over a `Map<Symbol, Address>` handler table: every dynamic hop is full invocation overhead and widens both the trust surface and the auth tree.

---

[← Cheat sheet index](index.md) · [← 4. Types & abilities](04-types-and-abilities.md) · [6. Execution semantics →](06-execution-semantics.md)
