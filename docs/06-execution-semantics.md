# 6. Execution semantics

## 6.1 Abort codes → `#[contracterror]`

```move
// Move
const E_INSUFFICIENT: u64 = 2;
assert!(bal >= amt, E_INSUFFICIENT);        // or: abort E_INSUFFICIENT
```

```rust
// Soroban
#[contracterror] #[derive(Copy, Clone, Debug, Eq, PartialEq)] #[repr(u32)]
pub enum Error { NotInitialized = 1, InsufficientBalance = 2, InvalidAmount = 3 }

pub fn distribute(…) -> Result<(), Error> {
    if bal < amt { return Err(Error::InsufficientBalance); }   // preferred: typed Result
    // or: panic_with_error!(&env, Error::InsufficientBalance);
    Ok(())
}
```

- ⚠ **FALSE FRIEND**, bare `panic!("msg")`, `unwrap()`, `expect()` surface as **opaque host errors** callers can't match on. Move's `abort` always carried your code; here only `panic_with_error!` / `Err` do. Ban `unwrap`/`expect` on any externally reachable path.
- Error codes are public ABI; never renumber across upgrades. With multiple contracts, partition now (pool 1xx, policy 2xx, attestation 3xx) so errors are attributable across `try_` boundaries.
- An `Err`/panic rolls back all state changes of the failing invocation, including its nested calls' writes.

[Move: abort & assert](https://aptos.dev/build/smart-contracts/book/abort-and-assert) · [docs.rs: contracterror](https://docs.rs/soroban-sdk/latest/soroban_sdk/attr.contracterror.html) · [errors example](https://developers.stellar.org/docs/build/smart-contracts/example-contracts)

## 6.2 Catchable callee failures ∅

Move has no try/catch: any abort anywhere kills the transaction. Soroban generated clients expose both `foo()` (panics on failure) and `try_foo()` returning a **doubly** nested `Result`. The shape depends on whether the contract fn returns `Result`, and the typed error lands in `Err(Ok(_))`, not `Ok(Err(_))`, which is the intuitive guess and is wrong:

```rust
// fn contribute(..) -> Result<(), Error>
let r: Result<Result<(), ConversionError>, Result<Error, InvokeError>>
     = client.try_contribute(&donor, &asset, &0);
assert_eq!(r, Err(Ok(Error::InvalidAmount)));   // ← your #[contracterror], verified

// fn bucket_balance(..) -> i128   (no Result)
let r2: Result<Result<i128, soroban_sdk::Error>, Result<soroban_sdk::Error, InvokeError>>
      = client.try_bucket_balance(&asset, &Category::Fuqara);   // Ok(Ok(0))
```

Read it as: outer `Err` = the call failed, and its payload distinguishes *your* typed error (`Ok(e)`) from a host-level failure (`Err(InvokeError)`: budget, bad auth, type mismatch). The inner `Ok` position is the conversion result for the return value. A caught callee failure rolls back **the callee's** writes; the caller continues with its own state intact. Budget exhaustion ends the whole transaction regardless; it is not catchable.

Design consequence with no Move reflex: a caller can *tolerate* a failing dependency (skip a broken oracle, degrade gracefully). Conversely ⚠: `try_` on a token transfer followed by continuing on `Err` is exactly the silent-accounting-error shape; on money paths, use the panicking client or make the `Err` arm abort. And note `TokenClient::transfer` returns `()`; "checking the transfer result" isn't a thing; a failed transfer *traps*, which is the guarantee.

## 6.3 Dispatch and reentrancy

**Move**, static dispatch; reentrancy structurally impossible; the compiler sees the whole call graph.
**Soroban**, dynamic dispatch by address (§5.2), but the **host blocks reentrancy** at runtime, direct and indirect, and there is no `delegatecall` (no foreign bytecode in your context; proxy hijacks don't exist). Same guarantee as Move, enforced one layer later. Still write checks-effects-interactions: external calls remain failure/budget/side-effect boundaries, callees are arbitrary code (§5.2), and the pattern survives a platform change the assumption wouldn't.

## 6.4 Gas → declared resources, and hard ceilings

**Move**, one gas meter, pay-as-you-go, plus a refundable storage deposit; no pre-declaration.
[Aptos: gas & txn fees](https://aptos.dev/network/blockchain/gas-txn-fee)

**Soroban**, multidimensional and **declared up front**: CPU instructions, ledger entries + bytes read/written, tx size, events size, rent, computed by simulation, embedded in the tx; exceed your declaration and the tx fails (rent/events refundable if unused; instructions/IO charged as declared). Per-transaction mainnet ceilings, from `InvocationResourceLimits::mainnet()` in SDK 27.0.6 (network-configured; check live): 400M instructions · 40MiB memory · 200 disk-read entries / 200 write entries / **400 ledger entries total** · 200KB read / 132,096B written · 65,536B per data entry (the *whole* instance-storage entry included; keys ≤250B) · 16,384B events + return value · 131,072B contract code.

What actually moves the needle: minimize distinct entries touched (fixed cost each, and footprint members cost whether used or not); never loop over user-controlled collection sizes (budget DoS / fee-griefing; cap explicitly); events instead of storage for off-chain-only data; bound signature counts in `__check_auth`. Profile with `stellar contract invoke … --send=no` or `env.cost_estimate().resources()` in tests.
[Resource limits & fees](https://developers.stellar.org/docs/networks/resource-limits-fees)

## 6.5 Environment quick hits

- `timestamp::now_seconds()` 🔧 → `env.ledger().timestamp()` (`u64` seconds); ledger sequence via `env.ledger().sequence()`.
- ⚠ Aptos has a secure on-chain randomness API; `env.prng()` is seeded from ledger state and **predictable**; never for auth, keys, or stakes.
- No I/O, no networking, no floats, no clock beyond the ledger. Everything reaches the world through `Env`.

---

[← Cheat sheet index](index.md) · [← 5. Modules vs. contracts](05-modules-vs-contracts.md) · [7. Events →](07-events.md)
