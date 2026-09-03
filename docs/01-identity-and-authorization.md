# 1. Identity & authorization

## 1.1 `&signer` → `Address` + `require_auth()`

**Move**, `signer` is an unforgeable capability minted only by the VM for actual transaction signers; possession *is* the proof of consent. It has only `drop`; even its scarcity is ability-encoded.
[Move: signer](https://aptos.dev/build/smart-contracts/book/signer) · [Soroban: authorization](https://developers.stellar.org/docs/learn/fundamentals/contract-development/authorization)

**Soroban**, there is no `msg.sender` and no signer type. The acting party is an ordinary `Address` parameter (plain, cloneable data), and consent is a separate runtime host query:

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

`signer::address_of(&s)` 🔧 disappears; the `Address` is already what you have.

**Where it breaks, internalize all four:**

- ⚠ **FALSE FRIEND**, an `Address` parameter proves nothing by itself. Forgetting `require_auth` compiles, runs, and passes naive tests. Every `Address` arg forces a decision: does this call spend, reduce, or reconfigure something this address owns? If yes → auth it. Reading public data / crediting a balance → no.
- ⚠ **FALSE FRIEND**, `who.require_auth()` on a caller-supplied `who` proves the caller controls `who`, **not** that `who` is privileged. For admin paths, load the role address from storage and auth *that* (Move analogue: `assert!(address_of(s) == @admin)`, same discipline, but here it's the *only* line of defense).
- **Auth is a signed invocation tree, and it does not cascade.** The user signs "`A.foo(args)`, and within it `B.bar(args)`". `require_auth` deep in the stack passes only if the actual call path matches the signed tree; each node matches at most once per tx. Refactoring your call graph changes the tree shape and **silently breaks integrators** signing the old shape; the tree is ABI.
- ⚠ **FALSE FRIEND (the #1 real-world Soroban auth bug)**, auth replay through middleware. If `settle(user, amt)` calls `token.transfer(&user, …)` but never calls `user.require_auth()`, anyone can call `settle(victim, …)` and consume victim's pre-signed inner transfer auth. Re-auth at **every** layer that exercises an address's authority.

Replay protection (nonces, expirations on auth entries) is host-managed; you never track nonces.
[Auth starter guide](https://developers.stellar.org/docs/build/guides/auth/contract-authorization) · [Aptos security guidelines (signer-check discipline)](https://aptos.dev/build/smart-contracts/move-security-guidelines)

## 1.2 Multi-agent transactions → multiple auth entries

**Move**, a script/entry fn takes several `&signer`s; all sign the one transaction.
**Soroban**, one transaction carries multiple `SorobanAuthorizationEntry`s, one per authorizing `Address`; the contract just calls `require_auth()` on each party (atomic swap: `a.require_auth(); b.require_auth();`). Only the tx *source* pays fees; other parties sign auth entries, not the envelope.
[Move: signer (multi-signer scripts)](https://aptos.dev/build/smart-contracts/book/signer) · [Soroban: authorization](https://developers.stellar.org/docs/learn/fundamentals/contract-development/authorization)

## 1.3 Contracts as authorizing parties (invoker auth)

∅ in Move (a module is code, not a party; you mint signers for that, §1.4). In Soroban a contract address **is** a first-class auth party. When contract A directly calls B, B's `a_address.require_auth()` passes automatically; the direct call is the authorization. That covers B's immediate frame only; if B will make a *deeper* call needing A's authority, A pre-authorizes it:

```rust
env.authorize_as_current_contract(vec![&env, InvokerContractAuthEntry::Contract(
    SubContractInvocation {
        context: ContractContext { contract: token, fn_name: symbol!("transfer"),
                                   args: (me, dest, amt).into_val(&env) },
        sub_invocations: vec![&env],
    })]);
```

This is your `zakat_pool → SAC.transfer` path when the pool pays out: the pool's own address authorizes the transfer by being the direct invoker.

## 1.4 Resource accounts / `SignerCapability` → custom accounts + invoker auth

**Move**, a resource account is a keyless account whose `SignerCapability` lets a module mint its signer programmatically; the capability struct is the authority.
[Aptos: resource accounts](https://aptos.dev/build/smart-contracts/resource-accounts)

**Soroban**, split by what you used it for:

- *"My module acts as its own account"* → free. `env.current_contract_address()` holds tokens and passes auth as invoker (§1.3). No capability object to store or guard.
- *"Programmable signing policy"* (multisig, passkeys, spend limits) → a **custom account**: a contract implementing `CustomAccountInterface`; the host calls its `__check_auth(env, signature_payload: crypto::Hash<32>, signature, auth_contexts: Vec<Context>)` whenever that contract address must authorize something. Note the payload type: **`crypto::Hash<32>`, not `BytesN<32>`**, a host-constructed type you cannot forge in safe code, which is the type system doing the one job here it still does. Rules: verify that payload itself (verifying anything else authorizes arbitrary calls); enforce policy from `auth_contexts` (match on `Context::Contract` for the invoked contract and `fn_name`) or the policy is decorative; keep it lean; its cost rides on every tx the account signs. Protocol 27 (CAP-71) adds `delegate_auth` for modular signer contracts; delegate addresses arrive **unsanitized**, check registration first.
[Custom account example](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/custom-account)

This is the natural home for an attestation-authority quorum: the pool does `attest_addr.require_auth_for_args((recipient, asset, category, amount).into_val(&env))` and the 2-of-3 policy lives in `__check_auth`; host-managed nonces replace a hand-rolled sequence scheme and remove its head-of-line blocking.

## 1.5 `friend` visibility → ∅, auth replaces visibility

**Move**, `public(friend)` / `public(package)` restrict callers at the type level.
[Move: friends](https://aptos.dev/build/smart-contracts/book/friends)

**Soroban**, every `pub fn` in `#[contractimpl]` is callable by anyone, externally and cross-contract. No friend, no package visibility. Replacements, in order of preference:

1. Design the function to be safe for any caller (a pure `policy.allocate(amount)` computation needs no gate; the *caller* validates the result).
2. Role auth from storage (§2.1) for privileged paths.
3. Pin the expected caller: store contract B's `Address`, and `expected_caller.require_auth()`; passes only when B is the direct invoker (§1.3).

Private helpers = ordinary non-`pub` Rust fns, or fns outside `#[contractimpl]`.

---

[← Cheat sheet index](index.md) · [2. Capabilities & the object model →](02-capabilities-and-the-object-model.md)
