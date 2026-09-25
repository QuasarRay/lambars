use vstd::prelude::*;

verus! {

pub type EitherModel = (bool, bool);

pub open spec fn left(value: bool) -> EitherModel {
    (true, value)
}

pub open spec fn right(value: bool) -> EitherModel {
    (false, value)
}

pub open spec fn is_left(value: EitherModel) -> bool {
    value.0
}

pub open spec fn is_right(value: EitherModel) -> bool {
    !value.0
}

pub open spec fn left_option(value: EitherModel) -> (bool, bool) {
    (value.0, value.1)
}

pub open spec fn right_option(value: EitherModel) -> (bool, bool) {
    (!value.0, value.1)
}

pub open spec fn map_left_not(value: EitherModel) -> EitherModel {
    if value.0 { left(!value.1) } else { value }
}

pub open spec fn map_right_not(value: EitherModel) -> EitherModel {
    if value.0 { value } else { right(!value.1) }
}

pub open spec fn bimap_not(value: EitherModel) -> EitherModel {
    if value.0 { left(!value.1) } else { right(!value.1) }
}

pub open spec fn fold(value: EitherModel) -> bool {
    if value.0 { value.1 } else { !value.1 }
}

pub open spec fn swap(value: EitherModel) -> EitherModel {
    (!value.0, value.1)
}

pub open spec fn iterator_len(value: EitherModel) -> nat {
    if value.0 { 0 } else { 1 }
}

pub open spec fn left_or_default(value: EitherModel) -> bool {
    if value.0 { value.1 } else { false }
}

pub open spec fn right_or_default(value: EitherModel) -> bool {
    if value.0 { false } else { value.1 }
}

pub proof fn either_is_left_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(is_left(left(value)));
}

pub proof fn either_is_left_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(!is_left(right(value)));
}

pub proof fn either_is_right_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(!is_right(left(value)));
}

pub proof fn either_is_right_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(is_right(right(value)));
}

pub proof fn either_left_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(left_option(left(value)) == (true, value));
}

pub proof fn either_left_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(left_option(right(value)).0 == false);
}

pub proof fn either_right_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(right_option(left(value)).0 == false);
}

pub proof fn either_right_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(right_option(right(value)) == (true, value));
}

pub proof fn either_left_ref_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(left_option(left(value)) == (true, value));
}

pub proof fn either_left_ref_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(left_option(right(value)).0 == false);
}

pub proof fn either_right_ref_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(right_option(left(value)).0 == false);
}

pub proof fn either_right_ref_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(right_option(right(value)) == (true, value));
}

pub proof fn either_map_left_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(map_left_not(left(value)) == left(!value));
}

pub proof fn either_map_left_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(map_left_not(right(value)) == right(value));
}

pub proof fn either_map_right_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(map_right_not(left(value)) == left(value));
}

pub proof fn either_map_right_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(map_right_not(right(value)) == right(!value));
}

pub proof fn either_bimap_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(bimap_not(left(value)) == left(!value));
}

pub proof fn either_bimap_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(bimap_not(right(value)) == right(!value));
}

pub proof fn either_fold_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(fold(left(value)) == value);
}

pub proof fn either_fold_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(fold(right(value)) == !value);
}

pub proof fn either_swap_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(swap(left(value)) == right(value));
}

pub proof fn either_swap_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(swap(right(value)) == left(value));
}

pub proof fn either_unwrap_left_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(left_option(left(value)) == (true, value));
}

pub proof fn either_unwrap_right_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(right_option(right(value)) == (true, value));
}

pub proof fn either_into_options_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(left_option(left(value)) == (true, value));
    assert(right_option(left(value)).0 == false);
}

pub proof fn either_into_options_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(left_option(right(value)).0 == false);
    assert(right_option(right(value)) == (true, value));
}

pub proof fn either_iter_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(iterator_len(left(value)) == 0);
}

pub proof fn either_iter_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(iterator_len(right(value)) == 1);
}

pub proof fn either_left_or_default_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(left_or_default(left(value)) == value);
}

pub proof fn either_left_or_default_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(left_or_default(right(value)) == false);
}

pub proof fn either_right_or_default_on_left_variant_has_documented_left_behavior(value: bool) {
    assert(right_or_default(left(value)) == false);
}

pub proof fn either_right_or_default_on_right_variant_has_documented_right_behavior(value: bool) {
    assert(right_or_default(right(value)) == value);
}

pub proof fn either_swap_twice_returns_original_value(side: bool, payload: bool) {
    let value = (side, payload);
    assert(swap(swap(value)) == value);
}

pub proof fn either_bimap_matches_map_left_followed_by_map_right(side: bool, payload: bool) {
    let value = (side, payload);
    assert(bimap_not(value) == map_right_not(map_left_not(value)));
}

pub proof fn either_into_iterator_left_variant_yields_zero_items(value: bool) {
    assert(iterator_len(left(value)) == 0);
}

pub proof fn either_into_iterator_right_variant_yields_exactly_one_item(value: bool) {
    assert(iterator_len(right(value)) == 1);
}

pub proof fn either_shared_iterator_left_variant_yields_zero_items(value: bool) {
    assert(iterator_len(left(value)) == 0);
}

pub proof fn either_shared_iterator_right_variant_yields_reference_to_exact_right_value(value: bool) {
    assert(iterator_len(right(value)) == 1);
    assert(right_option(right(value)) == (true, value));
}

}
