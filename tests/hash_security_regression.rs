#![cfg(feature = "persistent")]

/// SC-001 source-level regression.
///
/// Kani and Verus can prove collision-handling and state invariants, but they do
/// not prove operating-system entropy. This guard prevents reintroduction of the
/// concrete audited defect: direct fixed `DefaultHasher::new()` on the default
/// untrusted-input path.
#[test]
fn sc001_default_hash_path_uses_random_state_not_fixed_default_hasher() {
    let source = include_str!("../src/persistent/hashmap.rs");
    assert!(
        source.contains("static DEFAULT_HASH_STATE: LazyLock<RandomState>"),
        "default hash path must retain a process-keyed RandomState"
    );
    assert!(
        !source.contains("let mut hasher = DefaultHasher::new();"),
        "fixed DefaultHasher::new() reintroduces SC-001"
    );
}
