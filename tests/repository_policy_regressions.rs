//! Repository-state regressions for safety findings that are not program-semantics properties.
//!
//! Kani and Verus cannot prove GitHub metadata, license-file presence, contact channels,
//! or Cargo manifest text in a meaningful executable-semantic sense. These tests bind those
//! repository-state requirements to ordinary CI instead of replacing them with vacuous proofs.

const ROOT_MANIFEST: &str = include_str!("../Cargo.toml");
const DERIVE_MANIFEST: &str = include_str!("../lambars-derive/Cargo.toml");
const SECURITY_POLICY: &str = include_str!("../SECURITY.md");
const TOOLCHAIN: &str = include_str!("../rust-toolchain.toml");
const ASYNC_IO: &str = include_str!("../src/effect/async_io/mod.rs");
const DERIVED_LENSES_SOURCE: &str = include_str!("../lambars-derive/src/lenses.rs");
const DERIVED_PRISMS_SOURCE: &str = include_str!("../lambars-derive/src/prisms.rs");
const CI_WORKFLOW: &str = include_str!("../.github/workflows/ci.yml");
const SECURITY_ASSURANCE_WORKFLOW: &str =
    include_str!("../.github/workflows/security-assurance.yml");
const KANI_WORKFLOW: &str = include_str!("../.github/workflows/kani-verification.yml");
const VERUS_WORKFLOW: &str = include_str!("../.github/workflows/verus-verification.yml");
const LOOM_SUITE: &str = include_str!("concurrent_lazy_loom_tests.rs");
const DENY_POLICY: &str = include_str!("../deny.toml");
const RELEASE_WORKFLOW: &str = include_str!("../.github/workflows/release.yml");
const QUALIFICATION: &str = include_str!("../verification/qualification.json");
const ROOT_LOCK: &str = include_str!("../Cargo.lock");
const IAI_MANIFEST: &str = include_str!("../benches/iai/Cargo.toml");
const PANIC_POLICY: &str = include_str!("../docs/safety/panic-policy.md");
const UNSAFE_BOUNDARY_CHECKER: &str =
    include_str!("../verification/tools/check_unsafe_boundary.py");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const ASYNC_POOL_SOURCE: &str = include_str!("../src/effect/async_io/pool.rs");
const ORDERED_UNIQUE_SET_SOURCE: &str =
    include_str!("../src/persistent/ordered_unique_set.rs");
const CODEOWNERS: &str = include_str!("../.github/CODEOWNERS");
const GOVERNANCE: &str = include_str!("../GOVERNANCE.md");
const MAINTAINERS: &str = include_str!("../MAINTAINERS.md");

#[test]
fn sc007_security_policy_describes_the_real_unsafe_boundary() {
    assert!(SECURITY_POLICY.contains("#![deny(unsafe_code)]"));
    assert!(SECURITY_POLICY.contains("lazy/concurrent-lazy"));
    assert!(!SECURITY_POLICY.contains("All `unsafe` code is forbidden"));
}

#[test]
fn sc016_root_toolchain_is_pinned_to_declared_msrv() {
    assert!(TOOLCHAIN.contains("channel = \"1.92.0\""));
    assert!(ROOT_MANIFEST.contains("rust-version = \"1.92.0\""));
}

#[test]
fn sc017_sc018_repository_metadata_points_to_audited_repository() {
    let expected = "repository = \"https://github.com/QuasarRay/lambars\"";
    assert!(ROOT_MANIFEST.contains(expected));
    assert!(DERIVE_MANIFEST.contains(expected));
}

#[test]
fn sc019_manifest_license_matches_repository_license_material() {
    assert!(ROOT_MANIFEST.contains("license = \"MIT\""));
    assert!(DERIVE_MANIFEST.contains("license = \"MIT\""));
    assert!(!ROOT_MANIFEST.contains("MIT OR Apache-2.0"));
    assert!(!DERIVE_MANIFEST.contains("MIT OR Apache-2.0"));
}

#[test]
fn sc020_security_policy_has_a_private_reporting_route() {
    assert!(SECURITY_POLICY.contains("Report a vulnerability"));
    assert!(SECURITY_POLICY.contains("private"));
    assert!(SECURITY_POLICY.contains("QuasarRay"));
}

#[test]
fn sc025_deprecation_version_does_not_claim_an_unreleased_version() {
    assert!(ASYNC_IO.contains("since = \"0.1.0\""));
    assert!(!ASYNC_IO.contains("deprecated since version 0.2.0"));
}

#[test]
fn sc024_generated_docs_do_not_emit_literal_quote_placeholders() {
    assert!(!DERIVED_LENSES_SOURCE.contains("`#field_name`"));
    assert!(!DERIVED_PRISMS_SOURCE.contains("`#variant_name`"));
    assert!(DERIVED_LENSES_SOURCE.contains("field_doc"));
    assert!(DERIVED_PRISMS_SOURCE.contains("variant_doc"));
}

#[test]
fn sc006_sc034_concurrency_suite_uses_real_loom_models() {
    assert!(LOOM_SUITE.contains("loom::model"));
    assert!(LOOM_SUITE.contains("Ordering::Acquire"));
    assert!(LOOM_SUITE.contains("Ordering::Release"));
    assert!(CI_WORKFLOW.contains("Loom concurrency model"));
}

#[test]
fn sc009_msrv_job_runs_exact_declared_version() {
    assert!(CI_WORKFLOW.contains("rustc 1.92.0"));
    assert!(CI_WORKFLOW.contains("rust-version = \"1.92.0\""));
    assert!(!CI_WORKFLOW.contains("toolchain: nightly-2025-12-15\n\n      - name: Check MSRV"));
}

#[test]
fn sc015_coverage_is_a_gating_threshold() {
    assert!(CI_WORKFLOW.contains("--fail-under-lines 80"));
    assert!(CI_WORKFLOW.contains("fail_ci_if_error: true"));
}

#[test]
fn sc031_formal_workflows_run_on_main_pushes() {
    assert!(KANI_WORKFLOW.contains("push:"));
    assert!(KANI_WORKFLOW.contains("branches: [main]"));
    assert!(VERUS_WORKFLOW.contains("push:"));
    assert!(VERUS_WORKFLOW.contains("branches: [main]"));
}

#[test]
fn sc036_dependency_policy_enforces_advisories_licenses_and_sources() {
    assert!(CI_WORKFLOW.contains("cargo audit --deny warnings"));
    assert!(CI_WORKFLOW.contains("cargo deny check"));
    assert!(DENY_POLICY.contains("unknown-registry = \"deny\""));
    assert!(DENY_POLICY.contains("unknown-git = \"deny\""));
    assert!(DENY_POLICY.contains("yanked = \"deny\""));
}

#[test]
fn sc039_excluded_verifier_crates_are_explicitly_smoke_checked() {
    assert!(CI_WORKFLOW.contains("verification/kani/Cargo.toml"));
    assert!(CI_WORKFLOW.contains("working-directory: verification/verus"));
}

#[test]
fn sc011_release_requires_source_tests_dependency_policy_and_both_formal_backends() {
    assert!(
        RELEASE_WORKFLOW.contains("needs: [qualification, verify, kani, verus, build-evidence]")
    );
    assert!(RELEASE_WORKFLOW.contains("cargo test --all-features --locked"));
    assert!(RELEASE_WORKFLOW.contains("cargo audit --deny warnings"));
    assert!(RELEASE_WORKFLOW.contains("Release Kani proofs"));
    assert!(RELEASE_WORKFLOW.contains("Release Verus proofs"));
}

#[test]
fn sc022_release_checks_explicit_package_surface() {
    assert!(ROOT_MANIFEST.contains("include = ["));
    assert!(DERIVE_MANIFEST.contains("include = ["));
    assert!(RELEASE_WORKFLOW.contains("check_package_contents.py"));
}

#[test]
fn sc038_release_emits_reproducibility_hash_sbom_and_attestation_evidence() {
    assert!(RELEASE_WORKFLOW.contains("package-pass-1.sha256"));
    assert!(RELEASE_WORKFLOW.contains("package-pass-2.sha256"));
    assert!(RELEASE_WORKFLOW.contains("diff -u"));
    assert!(RELEASE_WORKFLOW.contains("cargo-cyclonedx --version 0.5.9"));
    assert!(RELEASE_WORKFLOW.contains("sha256sum"));
    assert!(
        RELEASE_WORKFLOW
            .contains("uses: actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6 # v4")
    );
    assert!(RELEASE_WORKFLOW.contains("sbom-path: release-evidence/sbom.cdx.json"));
}

#[test]
fn sc040_release_is_fail_closed_on_complete_machine_readable_inventory() {
    for index in 1..=40 {
        let id = format!("SC-{index:03}");
        assert!(QUALIFICATION.contains(&id), "missing {id}");
    }
    assert!(RELEASE_WORKFLOW.contains("check_qualification.py"));
}

#[test]
fn sc012_sc037_ci_supply_chain_is_immutable_and_fail_closed() {
    assert!(CI_WORKFLOW.contains("check_supply_chain.py"));
    let workflows = [
        include_str!("../.github/workflows/benchmark-api.yml"),
        include_str!("../.github/workflows/benchmark-pr.yml"),
        include_str!("../.github/workflows/benchmark.yml"),
        include_str!("../.github/workflows/changelog.yml"),
        include_str!("../.github/workflows/ci.yml"),
        include_str!("../.github/workflows/kani-verification.yml"),
        include_str!("../.github/workflows/labeler.yml"),
        include_str!("../.github/workflows/profiling.yml"),
        include_str!("../.github/workflows/release.yml"),
        include_str!("../.github/workflows/security-assurance.yml"),
        include_str!("../.github/workflows/verus-verification.yml"),
    ];
    for workflow in workflows {
        assert!(!workflow.contains("releases/latest"));
        assert!(!workflow.contains("git clone --depth"));
        assert!(!workflow.contains("apt-get update || true"));
    }
}

#[test]
fn sc021_repository_has_explicit_safety_governance_and_ownership() {
    assert!(CODEOWNERS.contains("* @QuasarRay"));
    assert!(CODEOWNERS.contains("/verification/ @QuasarRay"));
    assert!(CODEOWNERS.contains("/.github/workflows/ @QuasarRay"));
    assert!(MAINTAINERS.contains("Safety-critical responsibilities"));
    assert!(GOVERNANCE.contains("fail-closed"));
    assert!(GOVERNANCE.contains("SC-008"));
    assert!(
        GOVERNANCE
            .contains("must not represent single-maintainer approval as independent assurance")
    );
}

#[test]
fn sc036_root_qualification_graph_excludes_audited_advisories() {
    let forbidden = [
        ("crossbeam-epoch", "0.9.18"),
        ("rand", "0.9.2"),
        ("anyhow", "1.0.100"),
        ("paste", "1.0.15"),
        ("bincode", "1.3.3"),
        ("proc-macro-error2", "2.0.1"),
    ];

    for (name, version) in forbidden {
        let package = format!("name = \"{name}\"\nversion = \"{version}\"");
        assert!(
            !ROOT_LOCK.contains(&package),
            "audited advisory package returned to root lock: {name} {version}"
        );
    }

    assert!(ROOT_LOCK.contains("name = \"crossbeam-epoch\"\nversion = \"0.9.21\""));
    assert!(ROOT_LOCK.contains("name = \"rand\"\nversion = \"0.9.5\""));
    assert!(ROOT_LOCK.contains("name = \"anyhow\"\nversion = \"1.0.104\""));
    assert!(ROOT_LOCK.contains("name = \"pastey\"\nversion = \"0.2.3\""));
}

#[test]
fn sc037_profiler_dependencies_are_isolated_from_release_workspace() {
    assert!(ROOT_MANIFEST.contains("\"benches/iai\""));
    assert!(!ROOT_MANIFEST.contains("iai-callgrind ="));
    assert!(IAI_MANIFEST.contains("iai-callgrind = \"=0.16.1\""));
}

#[test]
fn sc026_panic_contract_is_explicit_and_hidden_pool_panics_do_not_return() {
    for required in [
        "AsyncPool::spawn",
        "ConcurrentLazy::force",
        "Lazy::force",
        "Freer::interpret",
        "panic=abort",
        "SC-026 remains release-blocking",
    ] {
        assert!(PANIC_POLICY.contains(required), "panic policy missing {required}");
    }

    assert!(
        !ASYNC_POOL_SOURCE.contains("expect(\"semaphore should not be closed\")")
    );
    assert!(
        !ASYNC_POOL_SOURCE.contains("expect(\"channel should not be closed\")")
    );
    assert!(ASYNC_POOL_SOURCE.contains("PoolError::PoolClosed"));
}


#[test]
fn sc013_security_assurance_has_sanitizer_fuzz_and_codeql() {
    assert!(SECURITY_ASSURANCE_WORKFLOW.contains("AddressSanitizer"));
    assert!(SECURITY_ASSURANCE_WORKFLOW.contains("cargo-fuzz --version 0.13.2 --locked"));
    assert!(SECURITY_ASSURANCE_WORKFLOW.contains("fuzz run persistent_collections"));
    assert!(SECURITY_ASSURANCE_WORKFLOW.contains("fuzz run lazy_totality"));
    assert!(SECURITY_ASSURANCE_WORKFLOW.contains("fuzz run freer_totality"));
    assert!(
        SECURITY_ASSURANCE_WORKFLOW
            .contains("github/codeql-action/analyze@2892aa5e19bbd11bc0cff5427e3b750a04d9e3c2")
    );
    assert!(ROOT_MANIFEST.contains("\"fuzz\""));
}

#[test]
fn sc014_architecture_matrix_covers_x86_64_and_aarch64() {
    assert!(
        SECURITY_ASSURANCE_WORKFLOW.contains("x86_64-unknown-linux-gnu")
    );
    assert!(
        SECURITY_ASSURANCE_WORKFLOW.contains("aarch64-unknown-linux-gnu")
    );
    assert!(
        SECURITY_ASSURANCE_WORKFLOW
            .contains("cargo +1.92.0 check --all-features --lib --target")
    );
}


#[test]
fn sc004_ordered_set_invariant_predicate_exists_in_release_profiles() {
    assert!(
        ORDERED_UNIQUE_SET_SOURCE.contains("fn is_strictly_sorted<T: Ord>(slice: &[T]) -> bool")
    );
    assert!(
        !ORDERED_UNIQUE_SET_SOURCE.contains(
            "#[cfg(debug_assertions)]\n#[inline]\nfn is_strictly_sorted"
        )
    );
}


#[test]
fn sc035_runtime_forbids_unsafe_without_local_escape_hatches() {
    assert!(LIB_SOURCE.contains("#![forbid(unsafe_code)]"));
    assert!(ROOT_MANIFEST.contains("unsafe_code = \"forbid\""));
    assert!(UNSAFE_BOUNDARY_CHECKER.contains("zero unsafe syntax"));
    assert!(!UNSAFE_BOUNDARY_CHECKER.contains("ALLOWED ="));
}
