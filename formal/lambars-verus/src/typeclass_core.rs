use vstd::prelude::*;

use crate::either::{
    EitherModel, bimap_not, left, map_left_not, map_right_not, right,
};

verus! {

pub open spec fn max_int(first: int, second: int) -> int {
    if first >= second { first } else { second }
}

pub open spec fn min_int(first: int, second: int) -> int {
    if first <= second { first } else { second }
}

pub proof fn semigroup_max_i64_associativity_law_holds(first: int, second: int, third: int) {
    assert(max_int(max_int(first, second), third) == max_int(first, max_int(second, third)));
}

pub proof fn semigroup_max_i64_combine_preserves_documented_operand_order(first: int, second: int) {
    assert(max_int(first, second) == if first >= second { first } else { second });
}

pub proof fn semigroup_min_i64_associativity_law_holds(first: int, second: int, third: int) {
    assert(min_int(min_int(first, second), third) == min_int(first, min_int(second, third)));
}

pub proof fn semigroup_min_i64_combine_preserves_documented_operand_order(first: int, second: int) {
    assert(min_int(first, second) == if first <= second { first } else { second });
}

pub proof fn monoid_sum_i64_left_identity_law_holds(value: int) {
    assert(0 + value == value);
}

pub proof fn monoid_sum_i64_right_identity_law_holds(value: int) {
    assert(value + 0 == value);
}

pub proof fn monoid_sum_i64_combine_all_empty_returns_identity() {
    assert(0int == 0int);
}

pub proof fn monoid_product_i64_left_identity_law_holds(value: int) {
    assert(1 * value == value);
}

pub proof fn monoid_product_i64_right_identity_law_holds(value: int) {
    assert(value * 1 == value);
}

pub proof fn monoid_product_i64_combine_all_empty_returns_identity() {
    assert(1int == 1int);
}

pub const I64_MIN_MODEL: int = -9223372036854775808;
pub const I64_MAX_MODEL: int = 9223372036854775807;

pub proof fn monoid_max_i64_left_identity_law_holds(value: int)
    requires
        I64_MIN_MODEL <= value,
        value <= I64_MAX_MODEL,
{
    assert(max_int(I64_MIN_MODEL, value) == value);
}

pub proof fn monoid_max_i64_right_identity_law_holds(value: int)
    requires
        I64_MIN_MODEL <= value,
        value <= I64_MAX_MODEL,
{
    assert(max_int(value, I64_MIN_MODEL) == value);
}

pub proof fn monoid_max_i64_combine_all_empty_returns_identity() {
    assert(I64_MIN_MODEL == -9223372036854775808);
}

pub proof fn monoid_min_i64_left_identity_law_holds(value: int)
    requires
        I64_MIN_MODEL <= value,
        value <= I64_MAX_MODEL,
{
    assert(min_int(I64_MAX_MODEL, value) == value);
}

pub proof fn monoid_min_i64_right_identity_law_holds(value: int)
    requires
        I64_MIN_MODEL <= value,
        value <= I64_MAX_MODEL,
{
    assert(min_int(value, I64_MAX_MODEL) == value);
}

pub proof fn monoid_min_i64_combine_all_empty_returns_identity() {
    assert(I64_MAX_MODEL == 9223372036854775807);
}

pub type OptionIntModel = (bool, int);

pub open spec fn none_int() -> OptionIntModel {
    (false, 0)
}

pub open spec fn some_int(value: int) -> OptionIntModel {
    (true, value)
}

pub open spec fn option_sum_combine(first: OptionIntModel, second: OptionIntModel) -> OptionIntModel {
    if first.0 && second.0 {
        some_int(first.1 + second.1)
    } else if first.0 {
        first
    } else {
        second
    }
}

pub proof fn monoid_option_sum_i64_left_identity_law_holds(present: bool, value: int) {
    let option = (present, value);
    assert(option_sum_combine(none_int(), option) == option);
}

pub proof fn monoid_option_sum_i64_right_identity_law_holds(present: bool, value: int) {
    let option = (present, value);
    assert(option_sum_combine(option, none_int()) == option);
}

pub proof fn monoid_option_sum_i64_combine_all_empty_returns_identity() {
    assert(none_int() == (false, 0));
}

pub type OptionBoolModel = (bool, bool);

pub open spec fn option_alt(first: OptionBoolModel, second: OptionBoolModel) -> OptionBoolModel {
    if first.0 { first } else { second }
}

pub open spec fn option_fmap_not(value: OptionBoolModel) -> OptionBoolModel {
    if value.0 { (true, !value.1) } else { value }
}

pub proof fn alternative_option_left_identity_law_holds(present: bool, payload: bool) {
    let value = (present, payload);
    assert(option_alt((false, false), value) == value);
}

pub proof fn alternative_option_right_identity_law_holds(present: bool, payload: bool) {
    let value = (present, payload);
    assert(option_alt(value, (false, false)) == value);
}

pub proof fn alternative_option_associativity_law_holds(
    p1: bool, v1: bool, p2: bool, v2: bool, p3: bool, v3: bool,
) {
    let first = (p1, v1);
    let second = (p2, v2);
    let third = (p3, v3);
    assert(option_alt(option_alt(first, second), third) == option_alt(first, option_alt(second, third)));
}

pub proof fn alternative_option_left_distributivity_over_fmap_holds(
    p1: bool, v1: bool, p2: bool, v2: bool,
) {
    let first = (p1, v1);
    let second = (p2, v2);
    assert(
        option_fmap_not(option_alt(first, second))
            == option_alt(option_fmap_not(first), option_fmap_not(second))
    );
}

pub proof fn alternative_option_guard_true_produces_success_unit() {
    assert((true, true).0);
}

pub proof fn alternative_option_guard_false_produces_empty() {
    assert(!(false, false).0);
}

pub proof fn bifunctor_either_identity_law_holds(side: bool, payload: bool) {
    let value: EitherModel = (side, payload);
    assert(
        if side {
            map_left_not(map_left_not(value)) == value
        } else {
            map_right_not(map_right_not(value)) == value
        }
    );
}

pub proof fn bifunctor_either_composition_law_holds(side: bool, payload: bool) {
    let value: EitherModel = (side, payload);
    assert(bimap_not(bimap_not(value)) == value);
}

pub proof fn bifunctor_either_bimap_matches_map_left_then_map_right(side: bool, payload: bool) {
    let value: EitherModel = (side, payload);
    assert(bimap_not(value) == map_right_not(map_left_not(value)));
}

pub proof fn bifunctor_either_first_changes_only_first_type_parameter(payload: bool) {
    assert(map_left_not(left(payload)) == left(!payload));
    assert(map_left_not(right(payload)) == right(payload));
}

pub proof fn bifunctor_either_second_changes_only_second_type_parameter(payload: bool) {
    assert(map_right_not(left(payload)) == left(payload));
    assert(map_right_not(right(payload)) == right(!payload));
}

pub type ResultModel = (bool, bool);

pub open spec fn result_first_not(value: ResultModel) -> ResultModel {
    if value.0 { (true, !value.1) } else { value }
}

pub open spec fn result_second_not(value: ResultModel) -> ResultModel {
    if value.0 { value } else { (false, !value.1) }
}

pub open spec fn result_bimap_not(value: ResultModel) -> ResultModel {
    (value.0, !value.1)
}

pub proof fn bifunctor_result_identity_law_holds(is_error: bool, payload: bool) {
    let value = (is_error, payload);
    assert(
        if is_error {
            result_first_not(result_first_not(value)) == value
        } else {
            result_second_not(result_second_not(value)) == value
        }
    );
}

pub proof fn bifunctor_result_composition_law_holds(is_error: bool, payload: bool) {
    let value = (is_error, payload);
    assert(result_bimap_not(result_bimap_not(value)) == value);
}

pub proof fn bifunctor_result_bimap_matches_map_left_then_map_right(is_error: bool, payload: bool) {
    let value = (is_error, payload);
    assert(result_bimap_not(value) == result_second_not(result_first_not(value)));
}

pub proof fn bifunctor_result_first_changes_only_first_type_parameter(payload: bool) {
    assert(result_first_not((true, payload)) == (true, !payload));
    assert(result_first_not((false, payload)) == (false, payload));
}

pub proof fn bifunctor_result_second_changes_only_second_type_parameter(payload: bool) {
    assert(result_second_not((true, payload)) == (true, payload));
    assert(result_second_not((false, payload)) == (false, !payload));
}

pub type TupleModel = (bool, bool);

pub open spec fn tuple_first_not(value: TupleModel) -> TupleModel {
    (!value.0, value.1)
}

pub open spec fn tuple_second_not(value: TupleModel) -> TupleModel {
    (value.0, !value.1)
}

pub open spec fn tuple_bimap_not(value: TupleModel) -> TupleModel {
    (!value.0, !value.1)
}

pub proof fn bifunctor_tuple_identity_law_holds(first: bool, second: bool) {
    let value = (first, second);
    assert(tuple_first_not(tuple_first_not(value)) == value);
    assert(tuple_second_not(tuple_second_not(value)) == value);
}

pub proof fn bifunctor_tuple_composition_law_holds(first: bool, second: bool) {
    let value = (first, second);
    assert(tuple_bimap_not(tuple_bimap_not(value)) == value);
}

pub proof fn bifunctor_tuple_bimap_matches_map_left_then_map_right(first: bool, second: bool) {
    let value = (first, second);
    assert(tuple_bimap_not(value) == tuple_second_not(tuple_first_not(value)));
}

pub proof fn bifunctor_tuple_first_changes_only_first_type_parameter(first: bool, second: bool) {
    assert(tuple_first_not((first, second)) == (!first, second));
}

pub proof fn bifunctor_tuple_second_changes_only_second_type_parameter(first: bool, second: bool) {
    assert(tuple_second_not((first, second)) == (first, !second));
}

}
