//! Verus backend for Lambars production-readiness specifications.

#![forbid(unsafe_code)]

use vstd::prelude::*;

pub mod compose;
pub mod either;
pub mod metaverification;

/// Canonical specifications currently represented by Verus proof obligations.
pub const VERUS_FORMAL_MODEL_COVERAGE: &[&str] = &[
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
    "constant_returns_cloned_constant_for_every_input",
    "flip_swaps_binary_function_argument_order",
    "flip_applied_twice_matches_original_binary_function",
    "compose_two_functions_applies_functions_right_to_left",
    "compose_three_functions_applies_functions_right_to_left",
    "compose_many_functions_preserves_right_to_left_order",
    "compose_with_identity_on_left_matches_original_function",
    "compose_with_identity_on_right_matches_original_function",
    "compose_associativity_holds_for_pure_functions",
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::VERUS_FORMAL_MODEL_COVERAGE;

    #[test]
    fn every_declared_verus_model_coverage_name_exists_in_the_canonical_catalog() {
        let catalog = lambars_spec::all_specs()
            .iter()
            .map(|spec| spec.name)
            .collect::<BTreeSet<_>>();
        for name in VERUS_FORMAL_MODEL_COVERAGE {
            assert!(catalog.contains(name), "unknown Verus coverage entry {name}");
        }
    }

    #[test]
    fn verus_model_coverage_manifest_contains_no_duplicate_names() {
        assert_eq!(
            VERUS_FORMAL_MODEL_COVERAGE.len(),
            VERUS_FORMAL_MODEL_COVERAGE.iter().copied().collect::<BTreeSet<_>>().len()
        );
    }
}
