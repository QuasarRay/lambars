# Governance

Lambars is being hardened for safety-critical production use. Repository governance is therefore fail-closed.

## Change control

- Changes to `src/**`, `lambars-derive/**`, `verification/**`, dependency manifests, security policy, or release/CI workflows require review.
- Direct writes to the protected release branch are prohibited by policy. GitHub branch/ruleset enforcement is tracked separately by SC-008 and is not considered satisfied by this document alone.
- A change that alters a safety contract must add or update regression evidence and the qualification matrix.
- Code-semantic safety findings require executable regression coverage and applicable Kani/Verus obligations.
- Repository/process findings require deterministic repository/API-state checks and must feed the fail-closed qualification gate.

## Release authority

Only the gated release workflow may publish a qualified release. A release is not safety-qualified unless every release-blocking SC-001..SC-040 entry is closed and the exact tag commit has passing source tests, Kani, Verus, dependency policy, package-surface checks, reproducibility evidence, SBOM, provenance, and artifact hashes.

## Review independence

When another qualified reviewer is available, safety-significant changes should receive independent review. Until the project has multiple maintainers, the repository must not represent single-maintainer approval as independent assurance; automated formal, concurrency, dependency, and release gates remain mandatory.

## Incident handling

Security issues follow `SECURITY.md`. Correctness or assurance regressions must be recorded as issues and linked to the relevant SC finding or a newly assigned safety finding.
