# 10. Toolchain quick map

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

[← Cheat sheet index](../README.md) · [← 9. Testing & assurance](09-testing-and-assurance.md) · [11. Porting reflexes — the grep-able checklist →](11-porting-reflexes.md)
