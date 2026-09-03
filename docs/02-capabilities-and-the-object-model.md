# 2. Capabilities & the object model

## 2.1 Capability structs → role addresses + `require_auth` ⚠

**Move** — authority as a value: `MintCapability`, `BurnCapability`, your `AdminCap`. Unforgeability comes from the constructor being private to the defining module — possession is permission because you cannot mint one yourself. The ability set is worth reading rather than assuming: `MintCapability<phantom CoinType> has copy, store` is *duplicable* and **not** droppable (hence `coin::destroy_mint_cap`), while a hand-rolled `AdminCap has store` is neither. "Capabilities are linear" is already false on the Move side, before Soroban enters it.

Object and FA refs — `ExtendRef`, `TransferRef`, `MintRef`, `BurnRef` — are the same mechanic wearing the object model's clothes: per-object rather than per-type, minted at creation instead of at module init. They decompose the same way, and carry one extra guarantee Soroban cannot reproduce (§2.2).
[Move: structs & resources](https://aptos.dev/build/smart-contracts/book/structs-and-resources) · [abilities](https://aptos.dev/build/smart-contracts/book/abilities) · [`coin.move`](https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-framework/sources/coin.move)

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

## 2.2 The Aptos object model → ∅ — decomposition

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

**⚠ The creation-time freeze is the part with no Soroban analogue at all.** Refs can only be minted from a `ConstructorRef`, and `struct ConstructorRef has drop` — no `store`, so it cannot outlive the transaction that created the object. The set of refs that will ever exist is therefore fixed at creation, which makes *absence* a permanent, externally verifiable property: "this FA has no `TransferRef`" means nobody can mint one later, including the issuer, and a holder can check that before touching it.

Soroban cannot express this. The nearest construction is an admin contract with no such entry point, which is only as strong as its upgrade policy (§5.5) — a property of the deployed Wasm and of whoever may replace it, not of the asset. Porting a design that leans on a *deliberately absent* ref means rebuilding that guarantee as immutability (no upgrade path at all) and documenting it, because the platform will not enforce it and no counterparty can verify it from the asset alone.

Note the ability sets while you are here, since they invert the classic capabilities in §2.1: `ExtendRef`/`TransferRef`/`DeleteRef`/`DeriveRef` are `drop, store` (storable, droppable, **not** copyable) and `LinearTransferRef` is `drop` alone — one-shot by construction.
[`object.move`](https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-framework/sources/object.move) · [`fungible_asset.move`](https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-framework/sources/fungible_asset.move)

## 2.3 Hot potato → ∅

A no-ability struct that must be consumed in the same transaction (flash-loan receipts, forced-callback patterns) has no Soroban equivalent — no abilities means no un-droppable values. Two partial substitutes: a Soroban tx is a single invocation anyway (§5.6), so "same-tx" is often structural; and obligations become explicit runtime checks before returning (assert the loan repaid, assert the invariant). Type-enforced obligations are simply gone.

---

[← Cheat sheet index](index.md) · [← 1. Identity & authorization](01-identity-and-authorization.md) · [3. State & storage →](03-state-and-storage.md)
