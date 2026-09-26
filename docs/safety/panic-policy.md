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
| `AsyncPool::new`, `with_queue_capacity`, `try_new`, `try_with_queue_capacity` | fallible | invalid capacities return `PoolError::InvalidCapacity`; no constructor panic wrapper remains |
| `ConcurrentLazy::wait_for` | bounded/fallible | `ConcurrentLazyWaitError` |
| `ConcurrentLazy::try_force`, `into_inner` | fallible | initializer unwind and same-thread re-entry return `ConcurrentLazyPoisonedError`; shared thread/Rayon/async decision model verified |
| `ConcurrentLazy::force` | partial convenience | explicitly documented panic behavior remains SC-026 work |
| `Lazy::try_force`, `into_inner` | fallible | initializer panic is caught and returned as `LazyPoisonedError`; `Lazy` is `!Sync` |
| `Lazy::force`, `try_force`, `force_mut`, `get_mut`, `into_inner` | fallible | initialization/re-entry/poison are represented by `LazyPoisonedError`; `Lazy` remains `!Sync` |
| `Freer::try_interpret` | fallible | intermediate/final type mismatches and handler/continuation unwinds return `InterpretError`; thread/Rayon/async decision model verified |
| `Freer::interpret`, `try_interpret` | fallible | both return `InterpretError`; no interpreter panic wrapper remains |
| `PureHandler` impossible-effect branches | internal invariant | must remain unreachable from safe typed construction and requires formal invariant evidence |

## Concurrency, parallelism, and async

A typed failure contract must be invariant under the execution wrapper used by the caller. Threaded, Rayon, and async adapters may change scheduling, but must not convert an error state into a panic-only or success state. Kani and Verus execution-mode models enforce this for the runtime, bounded ConcurrentLazy waiting, hash-security selector, and AsyncPool enqueue decisions already covered by the safety stack.

## Release rule

SC-026 remains release-blocking until every public panic surface above is either converted to a typed operation or explicitly classified as a programmer-error convenience with a separately verified total alternative. Repository regressions must prevent removal of this policy or reintroduction of the confirmed hidden panic paths.
