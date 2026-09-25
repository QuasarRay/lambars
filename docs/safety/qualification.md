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
| SC-006 | open | planned concurrency-model layer |
| SC-007 | remediation in branch | safety/06-release-assurance |
| SC-008 | open | requires GitHub branch/ruleset configuration |
| SC-009 | open | planned CI/MSRV layer |
| SC-010 | open | requires full semantic coverage/refinement expansion |
| SC-011 | open | planned release-gate layer |
| SC-012 | open | planned supply-chain pinning layer |
| SC-013 | open | planned security-test layer |
| SC-014 | open | planned portability matrix |
| SC-015 | open | planned coverage threshold |
| SC-016 | remediation in branch | safety/06-release-assurance |
| SC-017 | remediation in branch | safety/06-release-assurance |
| SC-018 | remediation in branch | safety/06-release-assurance |
| SC-019 | remediation in branch | safety/06-release-assurance |
| SC-020 | remediation in branch | safety/06-release-assurance |
| SC-021 | open | planned governance layer |
| SC-022 | open | planned package-content gate |
| SC-023 | remediation in branch | safety/07-proc-macro-hygiene |
| SC-024 | remediation in branch | safety/07-proc-macro-hygiene |
| SC-025 | remediation in branch | safety/06-release-assurance |
| SC-026 | open | planned panic/totality review layer |
| SC-027 | open | planned runtime-fallibility layer |
| SC-028 | open | planned bounded ConcurrentLazy API layer |
| SC-029 | open | planned trusted-hasher type boundary |
| SC-030 | remediation in PR | #11 |
| SC-031 | partial | Kani + Verus workflows exist in stack; release/status gating still open |
| SC-032 | open | direct executable refinement expansion required |
| SC-033 | open | Kani coverage expansion required |
| SC-034 | open | real Loom/model interleaving layer required |
| SC-035 | open | unsafe boundary policy/gating layer required |
| SC-036 | open | dependency policy layer required |
| SC-037 | open | benchmark/reproducibility pinning layer required |
| SC-038 | open | SBOM/provenance/source-to-package equivalence layer required |
| SC-039 | open | workspace verification aggregation layer required |
| SC-040 | partial | this matrix exists; release must gate on machine-readable completion |

## Concurrency / parallel / async rule

For each code-semantic finding, proof scope must distinguish:

- **sequential behavior**;
- **shared-thread/concurrent behavior** when the type is `Send`/`Sync`;
- **parallel behavior** for Rayon-enabled APIs;
- **async behavior** for `AsyncIO`, futures, runtime bridges and async macros.

A finding is not promoted to “closed for all Lambars functionality” merely because one representative function is proven. Coverage expansion is tracked separately under SC-010/032/033/034.
