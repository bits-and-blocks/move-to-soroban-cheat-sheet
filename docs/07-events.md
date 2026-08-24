# 7. Events

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

## 7.1 Reading events on-chain → ∅ on **both** chains

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

[← Cheat sheet index](../README.md) · [← 6. Execution semantics](06-execution-semantics.md) · [8. Tokens →](08-tokens.md)
