# 4. Types & abilities

## 4.1 Abilities → ∅ — the headline difference

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

## 4.2 No value objects: funds never pass through your code

**Move** — `coin::withdraw` puts a `Coin` *in your hand*; you hold it, split it, merge it, deposit it. Escrow = storing the `Coin` in your resource.

**Soroban** — ⚠ there is nothing to hold. A token is an external contract keeping balances internally; "moving funds" is calling it, and the transfer completes *inside that call*:

```rust
token::TokenClient::new(&env, &asset)
    .transfer(&from, &env.current_contract_address(), &amount);
// no value returned; nothing in hand; your address's balance in the token contract went up
```

Escrow/custody = the token contract says your address holds N, plus **your own bookkeeping of who it's for** — the eight-bucket accounting *is* the replacement for held `Coin`s, which is why §4.1's invariant is load-bearing. "Split" and "merge" are arithmetic on your books, not operations on values.

## 4.3 Generics → monomorphic ABI; asset = `Address` value

**Move** — `Coin<USDC>` and `Coin<EURC>` are distinct types; phantom parameters give type-level asset segregation, and mixing them is a compile error.
[Move: generics](https://aptos.dev/build/smart-contracts/book/generics)

**Soroban** — contract entry points are monomorphic; nothing generic crosses the ABI (Rust generics are fine internally). Asset identity is an `Address` **value** at runtime. Type-level segregation becomes **key discipline**: `Buckets(Address)` per asset, an `approved_assets` allowlist at the boundary, and per-asset invariants. ⚠ The compiler will never again catch USDC amounts credited to EURC buckets — only your keys and your tests do. (Deterministic SAC addresses help: pin the USDC/EURC addresses as reviewed constants per network rather than discovering them at runtime.)

## 4.4 Structs, enums, receiver functions 🔧

- Move 2 enums → `#[contracttype]` enums (unit + tuple variants; the idiomatic `DataKey`). Adding variants is forward-compatible; reshaping existing ones breaks stored-data decoding (§5.5).
- Positional structs → Rust tuple structs (supported by `#[contracttype]`).
- Receiver-style `value.method()` → ordinary Rust methods internally; **exported entry points** are associated fns in `#[contractimpl]` taking `Env` first — no receiver syntax at the ABI.
- Type names crossing the ABI are exported flat into the contract spec — keep them unique per contract.
- SDK types replace std (`#![no_std]`): `soroban_sdk::{String, Vec, Map, Bytes, BytesN, Symbol}` — host-object handles, not std collections. `Symbol` ≤32 chars `[a-zA-Z0-9_]`; `symbol_short!` ≤9.

[Move 2 release notes](https://aptos.dev/build/smart-contracts/book/move-2) · [docs.rs: contracttype](https://docs.rs/soroban-sdk/latest/soroban_sdk/attr.contracttype.html)

## 4.5 Integers: signedness & overflow ⚠ FALSE FRIEND ×2

**Move** — signed types **do** exist as of Move 2.3 (`i8…i256` alongside `u8…u256`, literals like `-1i8`), but the money path never meets them: `Coin`/FA amounts are `u64`, so a negative amount stays unrepresentable exactly where it would hurt. And every arithmetic op — signed and unsigned alike — **aborts** on overflow, underflow, or divide-by-zero with no opt-out; division truncates, negating a type's minimum aborts, and `as` casts abort when the value doesn't fit.
[Move: primitive types](https://aptos-labs.github.io/move-book/primitive-types.html) · [Move 2.3 release notes](https://aptos-labs.github.io/move-book/move-versions.html) · [abort & assert](https://aptos.dev/build/smart-contracts/book/abort-and-assert)

**Soroban** — two independent traps:

- ⚠ The canonical token amount is `i128` — **signed**. `transfer(from, to, -1000)` is a withdrawal *from `to`* if unchecked. Validate `amount > 0` at **every** entry point taking an amount. Having `i128` in the language is not the same as meeting it on the money path: Aptos amounts are `u64`, so no Move reflex was ever trained against this attack.
- ⚠ Overflow safety is a **build-profile flag**, not a language guarantee: Rust wraps in release unless `overflow-checks = true` is set (the Stellar scaffold sets it; a copied `Cargo.toml` without it wraps silently). Use explicit `checked_add/sub/mul` on every money path regardless — it survives profile mistakes and fails with *your* typed error. `U256`/`I256` have `checked_*` (protocol 26+).

---

[← Cheat sheet index](index.md) · [← 3. State & storage](03-state-and-storage.md) · [5. Modules vs. contracts →](05-modules-vs-contracts.md)
