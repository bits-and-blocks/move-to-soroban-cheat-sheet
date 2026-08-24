# 11. Porting reflexes — the grep-able checklist

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

[← Cheat sheet index](../README.md) · [← 10. Toolchain quick map](10-toolchain.md) · [12. Verification →](12-verification.md)
