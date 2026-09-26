# Safety-Critical Qualification Matrix

This file is the durable workspace for the safety-critical audit findings SC-001 through SC-040.

## Evidence rule

A finding is closed only when the applicable evidence exists and CI enforces it:

1. production implementation or repository/process correction;
2. a regression check over the real implementation/configuration;
3. Kani proof where the property is representable over executable Rust/MIR;
4. Verus proof/refinement where the property is representable in Verus;
5. explicit concurrent / parallel / async applicability review;
6. proof/test results tied to the exact commit and release artifact.

**Kani and Verus do not verify the post-codegen machine binary.** Kani reasons over Rust/MIR and Verus over verified Rust/specification models. The release pipeline must therefore bind verifier outputs, compiler/toolchain identity, source commit, packaged crate and binary hashes rather than calling these tools “binary verifiers.”

For repository metadata and GitHub configuration (for example branch protection, license text, URLs, workflow pinning), Kani/Verus are marked **not applicable** rather than replaced with tautological proofs. Those findings require deterministic repository-state or API-state gates.

## Current stack

| Finding | State | Stack evidence |
|---|---|---|
| SC-001 | remediation in PR | #11 |
| SC-002 | remediation in PR | #10 |
| SC-003 | remediation in PR | #12 |
| SC-004 | remediation in PR | #13 |
| SC-005 | remediation in PR | #14 |
| SC-006 | remediation in branch | safety/08-ci-assurance: real Loom protocol models + Kani/Verus transition invariants |
| SC-007 | remediation in branch | safety/06-release-assurance |
| SC-008 | open | requires GitHub branch/ruleset configuration |
| SC-009 | remediation in branch | safety/08-ci-assurance: exact Rust 1.92.0 gate |
| SC-010 | open | requires full semantic coverage/refinement expansion |
| SC-011 | closed | release publish requires qualification + tests + Kani + Verus + build evidence; repository regression enforced |
| SC-012 | remediation in branch | safety/10-supply-chain-pinning: immutable action SHAs + verified executable downloads |
| SC-013 | remediation in branch | safety/22-assurance-matrix: AddressSanitizer + isolated bounded cargo-fuzz + pinned CodeQL, in addition to Miri/audit/deny |
| SC-014 | remediation in branch | safety/22-assurance-matrix: Linux/macOS/Windows host matrix plus exact-MSRV x86_64/aarch64 compile matrix |
| SC-015 | remediation in branch | safety/08-ci-assurance: 80% line floor + gating Codecov upload |
| SC-016 | remediation in branch | safety/06-release-assurance |
| SC-017 | remediation in branch | safety/06-release-assurance |
| SC-018 | remediation in branch | safety/06-release-assurance |
| SC-019 | remediation in branch | safety/06-release-assurance |
| SC-020 | remediation in branch | safety/06-release-assurance |
| SC-021 | partial remediation | safety/12-governance: CODEOWNERS + maintainers + fail-closed governance; GitHub enforcement remains SC-008 |
| SC-022 | closed | explicit package include + executable `cargo package --list` surface checker for both public crates |
| SC-023 | remediation in branch | safety/07-proc-macro-hygiene |
| SC-024 | remediation in branch | safety/07-proc-macro-hygiene |
| SC-025 | remediation in branch | safety/06-release-assurance |
| SC-026 | partial remediation | safety/17-panic-contract: explicit panic policy; AsyncPool enqueue closure is typed; Kani/Verus execution-mode proofs; Lazy and ConcurrentLazy result paths fixed; partial force/force_mut and panic-only convenience surfaces remain; Freer try_interpret fixed |
| SC-027 | remediation in branch | safety/16-runtime-fallibility: fallible global/handle/blocking APIs + unwind-to-error + thread/Rayon/async classifier proofs |
| SC-028 | remediation in branch | safety/14-concurrent-lazy-bounded: bounded thread/Rayon wait + Tokio spawn-blocking bridge + Kani/Verus classifier proofs |
| SC-029 | remediation in branch | safety/11-hash-hardening: insecure hashers retired; legacy flags are no-op; Kani/Verus keyed-mode invariants |
| SC-030 | remediation in PR | #11 |
| SC-031 | partial remediation | safety/08-ci-assurance: PR/main/manual formal workflows; required-status policy still open |
| SC-032 | open | direct executable refinement expansion required |
| SC-033 | open | Kani coverage expansion required |
| SC-034 | remediation in branch | safety/08-ci-assurance: bounded Loom interleavings + Kani/Verus state invariants |
| SC-035 | implemented | safety/24-safe-lazy-storage: Lazy→OnceCell, ConcurrentLazy→OnceLock, root/Cargo `forbid(unsafe_code)`, zero-unsafe checker, Kani/Verus semantic preservation |
| SC-036 | remediation in branch | safety/15-dependency-remediation: patched root lock, maintained paste fork, profiler-only advisories isolated; RustSec remains fail-closed |
| SC-037 | remediation in branch | safety/15-dependency-remediation: IAI profiler in standalone workspace with exact version; workflows use isolated manifest |
| SC-038 | closed | double-package hash comparison + SBOM + SHA256 + Sigstore attestation are release evidence requirements |
| SC-039 | remediation in branch | safety/08-ci-assurance: excluded verifier crates are explicit CI smoke targets |
| SC-040 | closed | machine-readable exact 40-item fail-closed release gate is a publish prerequisite |

## Concurrency / parallel / async rule

For each code-semantic finding, proof scope must distinguish:

- **sequential behavior**;
- **shared-thread/concurrent behavior** when the type is `Send`/`Sync`;
- **parallel behavior** for Rayon-enabled APIs;
- **async behavior** for `AsyncIO`, futures, runtime bridges and async macros.

A finding is not promoted to “closed for all Lambars functionality” merely because one representative function is proven. Coverage expansion is tracked separately under SC-010/032/033/034.
