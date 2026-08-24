# 12. Verification

Every Soroban snippet in this document exists as compiling code under [`verify/`](https://github.com/bits-and-blocks/move-to-soroban-cheat-sheet/tree/main/verify), and
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

[← Cheat sheet index](../README.md) · [← 11. Porting reflexes — the grep-able checklist](11-porting-reflexes.md) · [13. Primary sources →](13-primary-sources.md)
