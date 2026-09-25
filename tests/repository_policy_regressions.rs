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
