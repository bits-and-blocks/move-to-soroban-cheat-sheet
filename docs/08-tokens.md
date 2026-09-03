# 8. Tokens

## 8.1 Fungible: Coin/FA → SEP-41 + SAC

**Move** — assets are typed values (`Coin<T>`) or FA objects (`Metadata` + `FungibleStore`s) living in *holders'* storage; your code holds and moves the value itself (§4.2).
[Aptos: fungible asset](https://aptos.dev/build/smart-contracts/fungible-asset) · [standards overview](https://aptos.dev/build/smart-contracts/aptos-standards)

**Soroban** — a token is an external contract implementing **SEP-41**; balances live inside it; you call it (§4.2). Three things may stand behind one "token address": a custom contract, native XLM's SAC, or a classic Stellar asset's **Stellar Asset Contract** — a built-in SEP-41 wrapper at a deterministic per-asset address. Circle's USDC on Stellar is a classic asset ⇒ your custody path *is* the SAC path.

```rust
use soroban_sdk::token::{TokenClient, StellarAssetClient};
let t = TokenClient::new(&env, &asset);          // SEP-41: balance/transfer/approve/burn…
t.transfer(&from, &to, &amount);
let sac = StellarAssetClient::new(&env, &asset); // SAC admin surface: mint/clawback/set_admin…
```

What has no Move reflex behind it:

- **Auth pattern**: `transfer`/`approve`/`burn` auth `from`; `transfer_from`/`burn_from` auth the **spender** (the allowance pre-authorized the `from` side). Reads need no auth.
- **Allowances expire**: `approve(from, spender, amount, expiration_ledger)`. Re-approving overwrites → the classic front-run race (spend old + new); approve-to-0 first, or prefer exact-amount auth (`require_auth_for_args`) over standing allowances.
- ⚠ **FALSE FRIEND — the SAC drags classic-asset semantics into your contract.** Account (`G…`) balances live in **trustlines**: recipient missing one, or unauthorized under `AUTH_REQUIRED`, ⇒ the transfer **traps** (also: trustline balances cap at i64). Issuer flags apply — freezes (`AUTH_REVOCABLE`), clawback. Transfers *to* the issuer burn; *from* the issuer mint. Contract (`C…`) balances live in contract storage with full `i128` and no trustline. A distribution path paying `G…` recipients must treat "recipient not ready" as an expected failure mode, not an anomaly.
- Amounts `i128` and signed — §4.5. Decimals: query per token at registration, never assume 7. `transfer`'s destination is a `MuxedAddress` (plain `Address` converts) for exchange sub-account IDs.

[SEP-41](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md) · [Token interface](https://developers.stellar.org/docs/tokens/token-interface) · [SAC](https://developers.stellar.org/docs/tokens/stellar-asset-contract)

## 8.2 Non-fungible: Digital Assets → SEP-50

**Move/Aptos** — the **Digital Asset** standard, built directly on the object model (§2.2): a `Collection` object plus `Token` objects, each token holding a reference to its parent collection (the collection does *not* own the token). Authority is capability refs minted **at creation and only then** — `MutatorRef` (edit description/URI), `BurnRef`, `TransferRef` — royalties configurable per collection or per token, `mint_soul_bound` for untransferable tokens, and NFTs that can own NFTs. It supersedes legacy Token v1, whose transfers required recipient opt-in.
[Aptos: digital asset](https://aptos.dev/build/smart-contracts/digital-asset) · [aptos-token-objects](https://aptos.dev/build/smart-contracts/aptos-standards)

**Soroban** — **SEP-50**, an ERC-721-shaped contract interface: `balance` / `owner_of` / `transfer` / `transfer_from` / `approve` / `approve_for_all` / `get_approved` / `is_approved_for_all`, plus `name` / `symbol` / `token_uri`. It deviates from ERC-721 deliberately: `transfer()` is added (owner-initiated transfers are the common case and cheaper than the approve-then-pull dance), `safeTransferFrom` is dropped (auth entries already cover the class of mistake it guarded), and `token_id` is format-agnostic — sequential, UUID, or hash.

Where the analogy breaks:

- ⚠ **FALSE FRIEND — there is no SAC for NFTs.** SEP-41's saving grace is that the most important fungible tokens are SACs: fixed protocol code, behaviour you can *assume* (§5.8). SEP-50 has no protocol-level implementation at all. Every NFT you touch is arbitrary contract code that may lie in `owner_of`, charge on transfer, or refuse recipients. The §5.8 "every unknown token is DFA-grade" posture applies here with **no** trustable tier to fall back on.
- ⚠ **SEP-50 is a Draft (0.1.0, March 2025), not a settled standard** — unlike SEP-41. Authored by OpenZeppelin with Boyan Barakov and Özgün Özerk. Treat the interface as likely-stable rather than frozen, and pin the implementation you integrated against.
- ∅ **Refs don't port** — `MutatorRef`/`BurnRef`/`TransferRef` are capability values, and §2.1 already settled that: no scarce values, so authority becomes role `Address`es in storage plus `require_auth`. "Mint-time only" scarcity becomes "set once in `__constructor`, never expose a setter" (§5.4).
- ∅ **NFTs owning NFTs** has no substrate (§2.2). An owner is an `Address`; since a contract address is an `Address`, composition is a cross-contract relationship you design, not resource attachment you get.
- ⚠ **Every token is a rent liability.** Each NFT's ownership entry is a persistent ledger entry with its own TTL (§3.5), so a 10,000-piece collection is 10,000 entries someone must keep alive — an operational cost that scales with supply and has no Aptos analogue, where state persists once paid. This is the consideration that should drive variant choice, not gas.
- **Implementation**: OpenZeppelin's `stellar-contracts` non-fungible module — **Base** (most cases), **Consecutive** (batch minting; stores ownership only at range boundaries and infers the rest — materially cheaper for large drops), **Enumerable** (on-chain enumeration, which you pay for in entries), plus `Burnable`, `Royalties`, and `Votes` extensions. The variants override each other's internals, so mixing them freely is explicitly discouraged.
- The pre-Soroban path still exists: **SEP-39** NFTs as classic assets with supply 1 and off-chain metadata. Cheap and needs no contract, but the "fungibility" is nominal and metadata lives off-chain — fine for tickets and badges, wrong for anything wanting on-chain behaviour.

[SEP-50](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0050.md) · [OpenZeppelin non-fungible](https://docs.openzeppelin.com/stellar-contracts/tokens/non-fungible/non-fungible) · [NFT example contract](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/non-fungible-token)

---

[← Cheat sheet index](index.md) · [← 7. Events](07-events.md) · [9. Testing & assurance →](09-testing-and-assurance.md)
