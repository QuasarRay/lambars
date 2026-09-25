# Lambars formal verification

This directory contains the canonical metaverification architecture for the production-readiness catalog.

## Non-negotiable invariants

1. Every catalog entry has one canonical specification ID: its test name.
2. Every canonical specification has both a Verus obligation and a Kani obligation.
3. Backend-specific code may strengthen implementation checks but may not weaken the canonical predicate.
4. Kani bounds are part of the proof statement and must be explicit.
5. Verus proofs must not encode the desired conclusion through an `assume`, external axiom, or trusted escape hatch.
6. Repeated proof shapes belong in reusable metaverification infrastructure.
7. Normal `cargo build` and the public API of `lambars` must not depend on either verifier.

## Layout

- `specs/`: generated normative specifications and registry.
- `tools/`: deterministic catalog/spec generation and repetition analysis.
- `kani/`: Kani harness package.
- `verus/`: Verus verification package.
- `../lambars-verify-macros/`: proc-macro2-based reusable generation infrastructure.

The Verus and Kani packages are intentionally verifier-specific. Shared semantic predicates live in ordinary safe Rust whenever possible so both backends check the same definition.
