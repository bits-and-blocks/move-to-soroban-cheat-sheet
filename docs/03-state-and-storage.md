# 3. State & storage

## 3.1 Global storage operators → `env.storage()`

| Move | Soroban | Notes |
|---|---|---|
| `move_to(&s, r)` | `storage().<tier>().set(&key, &val)` | no signer needed — it's *your* storage (§3.2) |
| `borrow_global<T>(a)` | `get::<K, V>(&key) -> Option<V>` | **owned copy**, not a borrow ⚠ §3.3 |
| `borrow_global_mut<T>(a)` | `update(&key, \|cur: Option<V>\| { … })` | the only safe read-modify-write idiom |
| `exists<T>(a)` | `has(&key)` | |
| `move_from<T>(a)` | `get` + `remove` | `remove` returns nothing; atomicity holds within the invocation anyway |

[Move: global storage operators](https://aptos.dev/build/smart-contracts/book/global-storage-operators) · [Soroban: persisting data](https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage/persisting-data)

Keys are anything `#[contracttype]` — always a dedicated enum:

```rust
#[contracttype] #[derive(Clone)]
pub enum DataKey {
    Governance, Distributor, PolicyAddr, AttestAddr, Paused,
    ApprovedAssets,
    Buckets(Address),           // all eight category balances for one asset, one entry
}
```

Untyped/ad-hoc keys can silently collide — the enum is the schema, and adding variants is the forward-compatible way to grow it.

## 3.2 Ownership inversion: state lives under the contract

**Move** — resources live *under user accounts*; your module is the API over state it doesn't hold (`Balance` under each user's address). The user pays their own storage; `move_to` needs their signer.
[Move: global storage structure](https://aptos.dev/build/smart-contracts/book/global-storage-structure)

**Soroban** — a contract reads and writes **only its own storage**. "The user's balance" is *your* entry `Balance(Address)`. Consequences: no signer needed to write (auth is orthogonal, §1.1); no `exists`-under-user checks or per-user initialization (`unwrap_or(default)` replaces the register-then-use dance); the contract's footprint is the universe — nothing of yours lives anywhere else; and *you* carry the growth and rent of per-user state (§3.5) rather than distributing it to users. Bounding what grows is now an architecture decision, not a billing default.

## 3.3 `get` returns a copy ⚠ FALSE FRIEND

The first-hour bug, and it produces no error of any kind:

```rust
// WRONG — compiles, reads back correctly within this call, never touches the ledger
let mut b: BucketSet = env.storage().persistent().get(&DataKey::Buckets(asset)).unwrap();
b.riqab += amount;          // mutated a deserialized local
// …no set → the write is lost
```

`get` deserializes an owned value out of the host; there is no `borrow_global_mut`, no write-through reference, and the borrow checker cannot help because nothing is borrowed. Codebase rule worth enforcing with review, not memory: **`get` + field mutation is banned; every read-modify-write goes through `update`/`try_update`:**

```rust
env.storage().persistent().update(&DataKey::Buckets(asset), |cur: Option<BucketSet>| {
    let mut b = cur.unwrap_or_default();
    b.add(category, amount);           // checked_add inside
    b
});
```

Also ⚠: `#[contract] pub struct ZakatPool { … }` fields do **not** persist — the struct is a namespace, not a resource. All state goes through `env.storage()`.

## 3.4 `acquires` → footprints, and parallelism

**Move** — `acquires` (explicit, or inferred by the Move 2 compiler) is function-level bookkeeping of global-storage access; Block-STM discovers conflicts optimistically at runtime.
[Move 2 release notes](https://aptos.dev/build/smart-contracts/book/move-2)

**Soroban** — nothing at the function level. Every **transaction** declares its read/write footprint up front (computed by simulation, embedded in the tx). Transactions touching the same read-write entry serialize; disjoint ones parallelize — *pessimistic* declaration vs. Block-STM's optimistic detection. Design consequences:

- **Fine-grained keys are the parallelism lever.** `Balance(Address)` per user, never one giant `Map` — a giant map is one entry, so every caller conflicts with every other *and* pays to read the whole thing.
- Everything touched must be in the footprint whether used or not, and entries-per-tx is capped (mainnet: 200 read / 200 write). Unbounded iteration over entries is both a fee and a ceiling problem.
- If actual state diverges from simulation (concurrent writes between simulate and submit), declared costs can be wrong → leave headroom.

## 3.5 Storage durability, TTL, archival — ∅

Nothing in Move prepares you for this: Aptos state persists indefinitely once paid (deposit refunded on deletion). Soroban storage is **rented**, in three tiers:

| Tier | Lifetime | On expiry | Use for |
|---|---|---|---|
| `instance()` | one TTL shared with the contract instance; **single entry**, ≤64KB serialized | archived with the contract; restorable | admin/config/small globals — one `extend_ttl` bumps everything |
| `persistent()` | per-key TTL | **archived** — restorable, and since protocol 23 auto-restored when in a tx's read-write footprint (you re-pay rent) | balances, anything that must survive |
| `temporary()` | per-key TTL | **deleted, permanently** | caches, quotes, inherently time-bounded data |

TTL is counted in ledgers (~5s, 17,280/day). Mainnet at time of writing: new persistent entries ~120 days, temporary ~1 day, ceiling ~180 days — network-configured, check live. `extend_ttl(threshold, extend_to)` is floor-only and idempotent: no-op unless current TTL < threshold. Standard pattern: extend the touched entry on every write ("active users pay for their own state"), extend instance TTL at the top of busy entry points. Protocol 26 adds `extend_ttl_with_limits` to stop callers forcing arbitrary rent on you.

```rust
const DAY: u32 = 17_280;
env.storage().instance().extend_ttl(30 * DAY, 120 * DAY);
env.storage().persistent().extend_ttl(&DataKey::Buckets(asset), 30 * DAY, 120 * DAY);
```

- ⚠ **FALSE FRIEND** — `temporary()` for anything load-bearing = permanent, silent data loss. Expired means gone, no restore path.
- ⚠ **FALSE FRIEND** — TTL is **not a security mechanism**. *Anyone* can extend *any* entry's TTL via the `ExtendFootprintTTLOp` transaction op, no contract auth involved. "This permission dies when its entry expires" is broken by design — store an explicit deadline in the value and check it.
- **Operational, not launch-time:** max TTL ~180 days means an immutable custody contract stays alive only if a keep-alive job runs forever. That job is part of the system, with an owner and a runbook — an unowned cron is how "nothing load-bearing archives" quietly stops being true.
- Never assume an entry you wrote is present later: `unwrap_or(default)` / `has()` where absence is meaningful.

[State archival](https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage/state-archival) · [Choosing the right storage](https://developers.stellar.org/docs/build/guides/storage/choosing-the-right-storage)

---

[← Cheat sheet index](index.md) · [← 2. Capabilities & the object model](02-capabilities-and-the-object-model.md) · [4. Types & abilities →](04-types-and-abilities.md)
