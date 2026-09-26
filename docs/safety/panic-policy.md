# Safety-Critical Panic Contract

This policy defines which Lambars APIs may unwind and which APIs are required to represent expected failure with typed results.

## Qualification rule

A public operation used in a safety-critical path must satisfy one of these conditions:

1. it is total over its documented input domain; or
2. expected failure is represented by a typed `Result` / `Option`; or
3. partiality is explicitly named and documented, with a non-panicking alternative available.

An internal `panic!`, `expect`, `unwrap`, or `unreachable!` is acceptable only when it represents an invariant that callers cannot violate and the invariant has executable/formal evidence. `panic=abort` does not make a partial API safe; it increases the consequence of an unexpected unwind.

## Current public surfaces

| Surface | Safety-critical status | Typed alternative / evidence |
|---|---|---|
| `runtime::global`, `runtime::handle` | fallible | return `RuntimeInitializationError` |
| `runtime::run_blocking`, `try_run_blocking` | fallible | return `BlockingError`; unwind converted to `ExecutionPanicked` |
| `AsyncPool::spawn`, `try_spawn` | fallible | `PoolClosed` and `QueueFull`; no internal queue/semaphore `expect` |
| `AsyncPool::new`, `with_queue_capacity` | partial convenience | use `try_new` / `try_with_queue_capacity`; conversion of convenience constructors remains SC-026 work |
| `ConcurrentLazy::wait_for` | bounded/fallible | `ConcurrentLazyWaitError` |
| `ConcurrentLazy::try_force`, `into_inner`, `try_map`, `try_flat_map`, `try_zip`, `try_zip_with` | fallible | initializer unwind/re-entry/poisoning are typed; thread/Rayon/async decision and reader paths verified |
| `ConcurrentLazy::force`, `map`, `flat_map`, `zip`, `zip_with` | partial convenience | matching total `try_*` / Result-returning counterparts exist for safety-critical use |
| `Lazy::try_force`, `try_force_mut`, `try_get_mut`, `into_inner`, `try_map`, `try_flat_map`, `try_zip`, `try_zip_with` | fallible | initializer panic/poisoning is typed as `LazyPoisonedError`; `Lazy` is `!Sync` |
| `Lazy::force`, `force_mut`, `get_mut`, `map`, `flat_map`, `zip`, `zip_with` | partial convenience | each has a total `try_*` or Result-returning counterpart for safety-critical use |
| `Freer::try_interpret` | fallible | intermediate/final type mismatches and handler/continuation unwinds return `InterpretError`; thread/Rayon/async decision model verified |
| `Freer::interpret` | partial convenience | delegates to `try_interpret` and panics on returned error |
| `PureHandler` impossible-effect branches | internal invariant | must remain unreachable from safe typed construction and requires formal invariant evidence |

## Concurrency, parallelism, and async

A typed failure contract must be invariant under the execution wrapper used by the caller. Threaded, Rayon, and async adapters may change scheduling, but must not convert an error state into a panic-only or success state. Kani and Verus execution-mode models enforce this for the runtime, bounded ConcurrentLazy waiting, hash-security selector, and AsyncPool enqueue decisions already covered by the safety stack.

## Release rule

SC-026 is implementable only when every public panic surface above is either converted to a typed operation or explicitly classified as a convenience with a separately verified total alternative. The current inventory satisfies that rule; release closure still requires the qualification evidence gates. Repository regressions must prevent removal of this policy or reintroduction of the confirmed hidden panic paths.
