# 9. Testing & assurance

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

[← Cheat sheet index](../README.md) · [← 8. Tokens](08-tokens.md) · [10. Toolchain quick map →](10-toolchain.md)
