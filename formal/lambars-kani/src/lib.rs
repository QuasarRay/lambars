//! Kani backend for Lambars formal verification.

#![forbid(unsafe_code)]

/// Canonical specifications currently discharged semantically by Kani.
pub const KANI_SEMANTIC_COVERAGE: &[&str] = &[
    "either_is_left_on_left_variant_has_documented_left_behavior",
    "either_is_left_on_right_variant_has_documented_right_behavior",
    "either_is_right_on_left_variant_has_documented_left_behavior",
    "either_is_right_on_right_variant_has_documented_right_behavior",
    "either_left_on_left_variant_has_documented_left_behavior",
    "either_left_on_right_variant_has_documented_right_behavior",
    "either_right_on_left_variant_has_documented_left_behavior",
    "either_right_on_right_variant_has_documented_right_behavior",
    "either_left_ref_on_left_variant_has_documented_left_behavior",
    "either_left_ref_on_right_variant_has_documented_right_behavior",
    "either_right_ref_on_left_variant_has_documented_left_behavior",
    "either_right_ref_on_right_variant_has_documented_right_behavior",
    "either_map_left_on_left_variant_has_documented_left_behavior",
    "either_map_left_on_right_variant_has_documented_right_behavior",
    "either_map_right_on_left_variant_has_documented_left_behavior",
    "either_map_right_on_right_variant_has_documented_right_behavior",
    "either_bimap_on_left_variant_has_documented_left_behavior",
    "either_bimap_on_right_variant_has_documented_right_behavior",
    "either_fold_on_left_variant_has_documented_left_behavior",
    "either_fold_on_right_variant_has_documented_right_behavior",
    "either_swap_on_left_variant_has_documented_left_behavior",
    "either_swap_on_right_variant_has_documented_right_behavior",
    "either_unwrap_left_on_left_variant_has_documented_left_behavior",
    "either_unwrap_right_on_right_variant_has_documented_right_behavior",
    "either_into_options_on_left_variant_has_documented_left_behavior",
    "either_into_options_on_right_variant_has_documented_right_behavior",
    "either_iter_on_left_variant_has_documented_left_behavior",
    "either_iter_on_right_variant_has_documented_right_behavior",
    "either_left_or_default_on_left_variant_has_documented_left_behavior",
    "either_left_or_default_on_right_variant_has_documented_right_behavior",
    "either_right_or_default_on_left_variant_has_documented_left_behavior",
    "either_right_or_default_on_right_variant_has_documented_right_behavior",
    "either_swap_twice_returns_original_value",
    "either_bimap_matches_map_left_followed_by_map_right",
    "either_into_iterator_left_variant_yields_zero_items",
    "either_into_iterator_right_variant_yields_exactly_one_item",
    "either_shared_iterator_left_variant_yields_zero_items",
    "either_shared_iterator_right_variant_yields_reference_to_exact_right_value",
    "identity_returns_exact_input_value",
    "identity_preserves_reference_identity_for_references",
    "constant_returns_cloned_constant_for_every_input",
    "flip_swaps_binary_function_argument_order",
    "flip_applied_twice_matches_original_binary_function",
    "placeholder_constant_has_expected_zero_sized_marker_semantics",
    "compose_two_functions_applies_functions_right_to_left",
    "compose_three_functions_applies_functions_right_to_left",
    "compose_with_identity_on_left_matches_original_function",
    "compose_with_identity_on_right_matches_original_function",
    "compose_associativity_holds_for_pure_functions",
];

#[cfg(kani)]
macro_rules! kani_proof {
    ($name:ident, $body:block) => {
        #[kani::proof]
        fn $name() $body
    };
}

#[cfg(kani)]
pub(crate) use kani_proof;

#[cfg(kani)]
mod compose;
#[cfg(kani)]
mod either;
#[cfg(kani)]
mod metaverification;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::KANI_SEMANTIC_COVERAGE;

    #[test]
    fn every_declared_kani_semantic_coverage_name_exists_in_the_canonical_catalog() {
        let catalog = lambars_spec::all_specs()
            .iter()
            .map(|spec| spec.name)
            .collect::<BTreeSet<_>>();
        for name in KANI_SEMANTIC_COVERAGE {
            assert!(catalog.contains(name), "unknown Kani coverage entry {name}");
        }
    }

    #[test]
    fn kani_semantic_coverage_manifest_contains_no_duplicate_names() {
        assert_eq!(
            KANI_SEMANTIC_COVERAGE.len(),
            KANI_SEMANTIC_COVERAGE.iter().copied().collect::<BTreeSet<_>>().len()
        );
    }
}
