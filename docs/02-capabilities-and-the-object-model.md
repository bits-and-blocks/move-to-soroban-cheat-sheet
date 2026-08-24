# 2. Capabilities & the object model

## 2.1 Capability structs → role addresses + `require_auth` ⚠

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

## 2.3 Hot potato → ∅

A no-ability struct that must be consumed in the same transaction (flash-loan receipts, forced-callback patterns) has no Soroban equivalent — no abilities means no un-droppable values. Two partial substitutes: a Soroban tx is a single invocation anyway (§5.6), so "same-tx" is often structural; and obligations become explicit runtime checks before returning (assert the loan repaid, assert the invariant). Type-enforced obligations are simply gone.

---

[← Cheat sheet index](../README.md) · [← 1. Identity & authorization](01-identity-and-authorization.md) · [3. State & storage →](03-state-and-storage.md)
