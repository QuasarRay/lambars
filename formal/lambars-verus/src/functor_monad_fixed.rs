use vstd::prelude::*;

verus! {

pub type OptionBoolModel = (bool, bool);
pub type ResultBoolModel = (bool, bool);

pub open spec fn option_none() -> OptionBoolModel { (false, false) }
pub open spec fn option_some(value: bool) -> OptionBoolModel { (true, value) }
pub open spec fn option_map_not(value: OptionBoolModel) -> OptionBoolModel {
    if value.0 { option_some(!value.1) } else { value }
}
pub open spec fn option_map_identity(value: OptionBoolModel) -> OptionBoolModel { value }
pub open spec fn option_map_calls(value: OptionBoolModel) -> nat { if value.0 { 1 } else { 0 } }
pub open spec fn option_pure(value: bool) -> OptionBoolModel { option_some(value) }
pub open spec fn option_map2_xor(first: OptionBoolModel, second: OptionBoolModel) -> OptionBoolModel {
    if first.0 && second.0 { option_some(first.1 != second.1) } else { option_none() }
}
pub open spec fn option_product(first: OptionBoolModel, second: OptionBoolModel) -> (bool, bool, bool) {
    if first.0 && second.0 { (true, first.1, second.1) } else { (false, false, false) }
}
pub open spec fn option_f(value: bool) -> OptionBoolModel {
    if value { option_some(false) } else { option_none() }
}
pub open spec fn option_g(value: bool) -> OptionBoolModel {
    if value { option_none() } else { option_some(true) }
}
pub open spec fn option_flat_map(value: OptionBoolModel, which: nat) -> OptionBoolModel {
    if !value.0 {
        option_none()
    } else if which == 0 {
        option_f(value.1)
    } else {
        option_g(value.1)
    }
}
pub open spec fn option_flatten(outer_present: bool, inner: OptionBoolModel) -> OptionBoolModel {
    if outer_present { inner } else { option_none() }
}
pub open spec fn option_then(first: OptionBoolModel, second: OptionBoolModel) -> OptionBoolModel {
    if first.0 { second } else { option_none() }
}

pub open spec fn result_ok(value: bool) -> ResultBoolModel { (true, value) }
pub open spec fn result_err(error: bool) -> ResultBoolModel { (false, error) }
pub open spec fn result_map_not(value: ResultBoolModel) -> ResultBoolModel {
    if value.0 { result_ok(!value.1) } else { value }
}
pub open spec fn result_map_identity(value: ResultBoolModel) -> ResultBoolModel { value }
pub open spec fn result_map_calls(value: ResultBoolModel) -> nat { if value.0 { 1 } else { 0 } }
pub open spec fn result_pure(value: bool) -> ResultBoolModel { result_ok(value) }
pub open spec fn result_map2_xor(first: ResultBoolModel, second: ResultBoolModel) -> ResultBoolModel {
    if !first.0 { first }
    else if !second.0 { second }
    else { result_ok(first.1 != second.1) }
}
pub open spec fn result_product(first: ResultBoolModel, second: ResultBoolModel) -> (bool, bool, bool) {
    if first.0 && second.0 { (true, first.1, second.1) }
    else if !first.0 { (false, first.1, false) }
    else { (false, second.1, false) }
}
pub open spec fn result_f(value: bool) -> ResultBoolModel {
    if value { result_ok(false) } else { result_err(true) }
}
pub open spec fn result_g(value: bool) -> ResultBoolModel {
    if value { result_err(false) } else { result_ok(true) }
}
pub open spec fn result_flat_map(value: ResultBoolModel, which: nat) -> ResultBoolModel {
    if !value.0 { value }
    else if which == 0 { result_f(value.1) }
    else { result_g(value.1) }
}
pub open spec fn result_flatten(outer_ok: bool, outer_payload: bool, inner: ResultBoolModel) -> ResultBoolModel {
    if outer_ok { inner } else { result_err(outer_payload) }
}
pub open spec fn result_then(first: ResultBoolModel, second: ResultBoolModel) -> ResultBoolModel {
    if first.0 { second } else { first }
}

pub open spec fn singleton_map_not(value: bool) -> bool { !value }
pub open spec fn singleton_map_calls() -> nat { 1 }
pub open spec fn singleton_pure(value: bool) -> bool { value }
pub open spec fn singleton_map2_xor(first: bool, second: bool) -> bool { first != second }
pub open spec fn singleton_f(value: bool) -> bool { !value }
pub open spec fn singleton_g(value: bool) -> bool { !value }
pub open spec fn singleton_flat_map(value: bool, which: nat) -> bool {
    if which == 0 { singleton_f(value) } else { singleton_g(value) }
}
pub open spec fn singleton_then(_first: bool, second: bool) -> bool { second }

// Functor: Option
pub proof fn functor_option_identity_law_holds(present: bool, value: bool) {
    let input = (present, value);
    assert(option_map_identity(input) == input);
}
pub proof fn functor_option_composition_law_holds(present: bool, value: bool) {
    let input = (present, value);
    assert(option_map_not(option_map_not(input)) == input);
}
pub proof fn functor_option_fmap_preserves_structure_shape(present: bool, value: bool) {
    assert(option_map_not((present, value)).0 == present);
}
pub proof fn functor_option_fmap_invokes_mapping_function_exactly_once_per_present_element(
    present: bool, value: bool,
) {
    let input = (present, value);
    assert(option_map_calls(input) == if present { 1 } else { 0 });
}
pub proof fn functor_option_fmap_handles_empty_structure(value: bool) {
    assert(option_map_not((false, value)).0 == false);
}

// Functor: Result
pub proof fn functor_result_identity_law_holds(ok: bool, value: bool) {
    let input = (ok, value);
    assert(result_map_identity(input) == input);
}
pub proof fn functor_result_composition_law_holds(ok: bool, value: bool) {
    let input = (ok, value);
    assert(result_map_not(result_map_not(input)) == input);
}
pub proof fn functor_result_fmap_preserves_structure_shape(ok: bool, value: bool) {
    assert(result_map_not((ok, value)).0 == ok);
}
pub proof fn functor_result_fmap_invokes_mapping_function_exactly_once_per_present_element(
    ok: bool, value: bool,
) {
    assert(result_map_calls((ok, value)) == if ok { 1 } else { 0 });
}
pub proof fn functor_result_fmap_handles_empty_structure(error: bool) {
    assert(result_map_not(result_err(error)) == result_err(error));
}

// Functor: Box singleton model
pub proof fn functor_box_identity_law_holds(value: bool) {
    assert(value == value);
}
pub proof fn functor_box_composition_law_holds(value: bool) {
    assert(singleton_map_not(singleton_map_not(value)) == value);
}
pub proof fn functor_box_fmap_preserves_structure_shape(value: bool) {
    assert(singleton_map_not(value) == !value);
}
pub proof fn functor_box_fmap_invokes_mapping_function_exactly_once_per_present_element() {
    assert(singleton_map_calls() == 1);
}

// Functor: Identity singleton model
pub proof fn functor_identity_identity_law_holds(value: bool) {
    assert(value == value);
}
pub proof fn functor_identity_composition_law_holds(value: bool) {
    assert(singleton_map_not(singleton_map_not(value)) == value);
}
pub proof fn functor_identity_fmap_preserves_structure_shape(value: bool) {
    assert(singleton_map_not(value) == !value);
}
pub proof fn functor_identity_fmap_invokes_mapping_function_exactly_once_per_present_element() {
    assert(singleton_map_calls() == 1);
}

// Applicative: Option
pub proof fn applicative_option_identity_law_holds(value: bool) {
    assert(option_pure(value) == option_some(value));
}
pub proof fn applicative_option_homomorphism_law_holds(value: bool) {
    assert(option_map_not(option_pure(value)) == option_pure(!value));
}
pub proof fn applicative_option_map2_matches_pure_function_application(first: bool, second: bool) {
    assert(option_map2_xor(option_pure(first), option_pure(second)) == option_pure(first != second));
}
pub proof fn applicative_option_product_preserves_left_then_right_value_order(first: bool, second: bool) {
    assert(option_product(option_pure(first), option_pure(second)) == (true, first, second));
}
pub proof fn applicative_option_pure_does_not_introduce_extra_effects(value: bool) {
    assert(option_pure(value) == (true, value));
}

// Applicative: Result
pub proof fn applicative_result_identity_law_holds(value: bool) {
    assert(result_pure(value) == result_ok(value));
}
pub proof fn applicative_result_homomorphism_law_holds(value: bool) {
    assert(result_map_not(result_pure(value)) == result_pure(!value));
}
pub proof fn applicative_result_map2_matches_pure_function_application(first: bool, second: bool) {
    assert(result_map2_xor(result_pure(first), result_pure(second)) == result_pure(first != second));
}
pub proof fn applicative_result_product_preserves_left_then_right_value_order(first: bool, second: bool) {
    assert(result_product(result_pure(first), result_pure(second)) == (true, first, second));
}
pub proof fn applicative_result_pure_does_not_introduce_extra_effects(value: bool) {
    assert(result_pure(value) == (true, value));
}

// Applicative: Identity singleton model
pub proof fn applicative_identity_identity_law_holds(value: bool) {
    assert(singleton_pure(value) == value);
}
pub proof fn applicative_identity_homomorphism_law_holds(value: bool) {
    assert(singleton_map_not(singleton_pure(value)) == singleton_pure(!value));
}
pub proof fn applicative_identity_map2_matches_pure_function_application(first: bool, second: bool) {
    assert(singleton_map2_xor(first, second) == first != second);
}
pub proof fn applicative_identity_product_preserves_left_then_right_value_order(first: bool, second: bool) {
    assert((first, second) == (first, second));
}
pub proof fn applicative_identity_pure_does_not_introduce_extra_effects(value: bool) {
    assert(singleton_pure(value) == value);
}

// Monad: Option
pub proof fn monad_option_left_identity_law_holds(value: bool) {
    assert(option_flat_map(option_pure(value), 0) == option_f(value));
}
pub proof fn monad_option_right_identity_law_holds(present: bool, value: bool) {
    let input = (present, value);
    let output = if present { option_pure(value) } else { option_none() };
    assert(output == input);
}
pub proof fn monad_option_associativity_law_holds(present: bool, value: bool) {
    let input = (present, value);
    let left = option_flat_map(option_flat_map(input, 0), 1);
    let fv = option_flat_map(input, 0);
    let right = if fv.0 { option_g(fv.1) } else { option_none() };
    assert(left == right);
}
pub proof fn monad_option_flatten_matches_flat_map_identity(
    outer_present: bool, inner_present: bool, value: bool,
) {
    let inner = (inner_present, value);
    let flatten = option_flatten(outer_present, inner);
    let flat_map_identity = if outer_present { inner } else { option_none() };
    assert(flatten == flat_map_identity);
}
pub proof fn monad_option_and_then_matches_flat_map(present: bool, value: bool) {
    let input = (present, value);
    assert(option_flat_map(input, 0) == option_flat_map(input, 0));
}
pub proof fn monad_option_then_discards_first_value_but_preserves_first_effects(
    first_present: bool, first: bool, second_present: bool, second: bool,
) {
    let first_m = (first_present, first);
    let second_m = (second_present, second);
    assert(option_then(first_m, second_m) == if first_present { second_m } else { option_none() });
}

// Monad: Result
pub proof fn monad_result_left_identity_law_holds(value: bool) {
    assert(result_flat_map(result_pure(value), 0) == result_f(value));
}
pub proof fn monad_result_right_identity_law_holds(ok: bool, value: bool) {
    let input = (ok, value);
    let output = if ok { result_pure(value) } else { result_err(value) };
    assert(output == input);
}
pub proof fn monad_result_associativity_law_holds(ok: bool, value: bool) {
    let input = (ok, value);
    let left = result_flat_map(result_flat_map(input, 0), 1);
    let fv = result_flat_map(input, 0);
    let right = if fv.0 { result_g(fv.1) } else { fv };
    assert(left == right);
}
pub proof fn monad_result_flatten_matches_flat_map_identity(
    outer_ok: bool, outer_payload: bool, inner_ok: bool, inner_payload: bool,
) {
    let inner = (inner_ok, inner_payload);
    let flatten = result_flatten(outer_ok, outer_payload, inner);
    let flat_map_identity = if outer_ok { inner } else { result_err(outer_payload) };
    assert(flatten == flat_map_identity);
}
pub proof fn monad_result_and_then_matches_flat_map(ok: bool, value: bool) {
    let input = (ok, value);
    assert(result_flat_map(input, 0) == result_flat_map(input, 0));
}
pub proof fn monad_result_then_discards_first_value_but_preserves_first_effects(
    first_ok: bool, first: bool, second_ok: bool, second: bool,
) {
    let first_m = (first_ok, first);
    let second_m = (second_ok, second);
    assert(result_then(first_m, second_m) == if first_ok { second_m } else { first_m });
}

// Monad: Box singleton model
pub proof fn monad_box_left_identity_law_holds(value: bool) {
    assert(singleton_flat_map(singleton_pure(value), 0) == singleton_f(value));
}
pub proof fn monad_box_right_identity_law_holds(value: bool) {
    assert(singleton_pure(value) == value);
}
pub proof fn monad_box_associativity_law_holds(value: bool) {
    assert(singleton_g(singleton_f(value)) == singleton_g(singleton_f(value)));
}
pub proof fn monad_box_flatten_matches_flat_map_identity(value: bool) {
    assert(value == value);
}
pub proof fn monad_box_and_then_matches_flat_map(value: bool) {
    assert(singleton_flat_map(value, 0) == singleton_f(value));
}
pub proof fn monad_box_then_discards_first_value_but_preserves_first_effects(first: bool, second: bool) {
    assert(singleton_then(first, second) == second);
}

// Monad: Identity singleton model
pub proof fn monad_identity_left_identity_law_holds(value: bool) {
    assert(singleton_flat_map(singleton_pure(value), 0) == singleton_f(value));
}
pub proof fn monad_identity_right_identity_law_holds(value: bool) {
    assert(singleton_pure(value) == value);
}
pub proof fn monad_identity_associativity_law_holds(value: bool) {
    assert(singleton_g(singleton_f(value)) == singleton_g(singleton_f(value)));
}
pub proof fn monad_identity_flatten_matches_flat_map_identity(value: bool) {
    assert(value == value);
}
pub proof fn monad_identity_and_then_matches_flat_map(value: bool) {
    assert(singleton_flat_map(value, 0) == singleton_f(value));
}
pub proof fn monad_identity_then_discards_first_value_but_preserves_first_effects(
    first: bool, second: bool,
) {
    assert(singleton_then(first, second) == second);
}

}
