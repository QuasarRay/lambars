# Lambars formal verification

This tree is the shared metaprogramming and metaverification layer for the production-readiness specification catalog.

## Invariants

1. The catalog contains exactly 2,533 canonical specifications and no duplicate names or IDs.
2. Every specification requires both a Verus and a Kani semantic discharge.
3. Backend-neutral models and proof adapters are single-sourced whenever their semantics are identical.
4. Repetitive law, variant, boundary, persistence, iterator, failure-state, drop, and concurrency proof shapes must use reusable proof combinators or generated adapters instead of copy/paste.
5. Presence of a generated harness is not semantic proof coverage. Coverage is tracked separately from catalog registration.
6. Build/packaging/CI specifications remain executable external gates; Verus/Kani may verify their decision models, but may not replace the real command execution.

## Crates

- `lambars-spec`: backend-neutral specification registry and declarative proof combinators.
- `lambars-formal-macros`: proc-macro, proc-macro2, attribute, derive, and delegate metaprogramming for proof adapters.
- verifier-specific crates are layered on top so ordinary Cargo builds remain independent of verifier toolchains.
