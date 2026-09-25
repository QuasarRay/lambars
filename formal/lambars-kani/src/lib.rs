//! Kani backend for Lambars formal verification.

#![forbid(unsafe_code)]

/// Canonical specifications currently discharged semantically by Kani.
pub const KANI_SEMANTIC_COVERAGE: &[&str] = &[
    "applicative_identity_homomorphism_law_holds",
    "applicative_identity_identity_law_holds",
    "applicative_identity_map2_matches_pure_function_application",
    "applicative_identity_product_preserves_left_then_right_value_order",
    "applicative_identity_pure_does_not_introduce_extra_effects",
    "applicative_option_homomorphism_law_holds",
    "applicative_option_identity_law_holds",
    "applicative_option_map2_matches_pure_function_application",
    "applicative_option_product_preserves_left_then_right_value_order",
    "applicative_option_pure_does_not_introduce_extra_effects",
    "applicative_result_homomorphism_law_holds",
    "applicative_result_identity_law_holds",
    "applicative_result_map2_matches_pure_function_application",
    "applicative_result_product_preserves_left_then_right_value_order",
    "applicative_result_pure_does_not_introduce_extra_effects",
    "functor_box_composition_law_holds",
    "functor_box_fmap_invokes_mapping_function_exactly_once_per_present_element",
    "functor_box_fmap_preserves_structure_shape",
    "functor_box_identity_law_holds",
    "functor_identity_composition_law_holds",
    "functor_identity_fmap_invokes_mapping_function_exactly_once_per_present_element",
    "functor_identity_fmap_preserves_structure_shape",
    "functor_identity_identity_law_holds",
    "functor_option_composition_law_holds",
    "functor_option_fmap_handles_empty_structure",
    "functor_option_fmap_invokes_mapping_function_exactly_once_per_present_element",
    "functor_option_fmap_preserves_structure_shape",
    "functor_option_identity_law_holds",
    "functor_result_composition_law_holds",
    "functor_result_fmap_handles_empty_structure",
    "functor_result_fmap_invokes_mapping_function_exactly_once_per_present_element",
    "functor_result_fmap_preserves_structure_shape",
    "functor_result_identity_law_holds",
    "monad_box_and_then_matches_flat_map",
    "monad_box_associativity_law_holds",
    "monad_box_flatten_matches_flat_map_identity",
    "monad_box_left_identity_law_holds",
    "monad_box_right_identity_law_holds",
    "monad_box_then_discards_first_value_but_preserves_first_effects",
    "monad_identity_and_then_matches_flat_map",
    "monad_identity_associativity_law_holds",
    "monad_identity_flatten_matches_flat_map_identity",
    "monad_identity_left_identity_law_holds",
    "monad_identity_right_identity_law_holds",
    "monad_identity_then_discards_first_value_but_preserves_first_effects",
    "monad_option_and_then_matches_flat_map",
    "monad_option_associativity_law_holds",
    "monad_option_flatten_matches_flat_map_identity",
    "monad_option_left_identity_law_holds",
    "monad_option_right_identity_law_holds",
    "monad_option_then_discards_first_value_but_preserves_first_effects",
    "monad_result_and_then_matches_flat_map",
    "monad_result_associativity_law_holds",
    "monad_result_flatten_matches_flat_map_identity",
    "monad_result_left_identity_law_holds",
    "monad_result_right_identity_law_holds",
    "monad_result_then_discards_first_value_but_preserves_first_effects",
    "semigroup_max_i64_associativity_law_holds",
    "semigroup_max_i64_combine_preserves_documented_operand_order",
    "semigroup_min_i64_associativity_law_holds",
    "semigroup_min_i64_combine_preserves_documented_operand_order",
    "monoid_sum_i64_left_identity_law_holds",
    "monoid_sum_i64_right_identity_law_holds",
    "monoid_sum_i64_combine_all_empty_returns_identity",
    "monoid_product_i64_left_identity_law_holds",
    "monoid_product_i64_right_identity_law_holds",
    "monoid_product_i64_combine_all_empty_returns_identity",
    "monoid_max_i64_left_identity_law_holds",
    "monoid_max_i64_right_identity_law_holds",
    "monoid_max_i64_combine_all_empty_returns_identity",
    "monoid_min_i64_left_identity_law_holds",
    "monoid_min_i64_right_identity_law_holds",
    "monoid_min_i64_combine_all_empty_returns_identity",
    "sum_new_into_inner_as_inner_and_as_inner_mut_are_consistent",
    "product_new_into_inner_as_inner_and_as_inner_mut_are_consistent",
    "max_new_into_inner_as_inner_and_as_inner_mut_are_consistent",
    "min_new_into_inner_as_inner_and_as_inner_mut_are_consistent",
    "monoid_option_sum_i64_left_identity_law_holds",
    "monoid_option_sum_i64_right_identity_law_holds",
    "monoid_option_sum_i64_combine_all_empty_returns_identity",
    "alternative_option_left_identity_law_holds",
    "alternative_option_right_identity_law_holds",
    "alternative_option_associativity_law_holds",
    "alternative_option_left_distributivity_over_fmap_holds",
    "alternative_option_guard_true_produces_success_unit",
    "alternative_option_guard_false_produces_empty",
    "bifunctor_either_identity_law_holds",
    "bifunctor_either_composition_law_holds",
    "bifunctor_either_bimap_matches_map_left_then_map_right",
    "bifunctor_either_first_changes_only_first_type_parameter",
    "bifunctor_either_second_changes_only_second_type_parameter",
    "bifunctor_result_identity_law_holds",
    "bifunctor_result_composition_law_holds",
    "bifunctor_result_bimap_matches_map_left_then_map_right",
    "bifunctor_result_first_changes_only_first_type_parameter",
    "bifunctor_result_second_changes_only_second_type_parameter",
    "bifunctor_tuple_identity_law_holds",
    "bifunctor_tuple_composition_law_holds",
    "bifunctor_tuple_bimap_matches_map_left_then_map_right",
    "bifunctor_tuple_first_changes_only_first_type_parameter",
    "bifunctor_tuple_second_changes_only_second_type_parameter",
    "option_type_constructor_with_type_replaces_only_inner_type",
    "result_type_constructor_with_type_preserves_error_type_and_replaces_success_type",
    "vec_type_constructor_with_type_replaces_only_element_type",
    "box_type_constructor_with_type_replaces_only_inner_type",
    "identity_type_constructor_with_type_replaces_only_inner_type",
    "identity_new_into_inner_as_inner_and_as_inner_mut_are_consistent",
    "bounded_integer_min_value_matches_primitive_min",
    "bounded_integer_max_value_matches_primitive_max",
    "bounded_float_min_value_is_negative_infinity",
    "bounded_float_max_value_is_positive_infinity",
    "bounded_char_min_value_is_null_character",
    "bounded_char_max_value_matches_char_max",
    "bounded_bool_min_value_is_false_and_max_value_is_true",
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
    "compose_many_functions_preserves_right_to_left_order",
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
mod functor_monad_fixed;
#[cfg(kani)]
mod metaverification;
#[cfg(kani)]
mod symbolic;
#[cfg(kani)]
mod typeclass_core;

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
            KANI_SEMANTIC_COVERAGE
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
        );
    }
}
