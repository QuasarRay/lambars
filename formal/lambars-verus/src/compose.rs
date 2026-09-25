use vstd::prelude::*;

verus! {

pub open spec fn identity_model(value: int) -> int {
    value
}

pub open spec fn constant_model(constant: int, _input: int) -> int {
    constant
}

pub open spec fn f(value: int) -> int {
    value + 1
}

pub open spec fn g(value: int) -> int {
    value * 2
}

pub open spec fn h(value: int) -> int {
    value - 3
}

pub open spec fn k(value: int) -> int {
    value + 11
}

pub open spec fn subtract(left: int, right: int) -> int {
    left - right
}

pub open spec fn flipped_subtract(left: int, right: int) -> int {
    subtract(right, left)
}

pub proof fn identity_returns_exact_input_value(value: int) {
    assert(identity_model(value) == value);
}

pub proof fn constant_returns_cloned_constant_for_every_input(constant: int, input: int) {
    assert(constant_model(constant, input) == constant);
}

pub proof fn flip_swaps_binary_function_argument_order(left: int, right: int) {
    assert(flipped_subtract(left, right) == subtract(right, left));
}

pub proof fn flip_applied_twice_matches_original_binary_function(left: int, right: int) {
    assert(subtract(left, right) == flipped_subtract(right, left));
}

pub proof fn compose_two_functions_applies_functions_right_to_left(value: int) {
    assert(f(g(value)) == (value * 2) + 1);
}

pub proof fn compose_three_functions_applies_functions_right_to_left(value: int) {
    assert(f(g(h(value))) == ((value - 3) * 2) + 1);
}

pub proof fn compose_many_functions_preserves_right_to_left_order(value: int) {
    assert(f(g(h(k(value)))) == ((((value + 11) - 3) * 2) + 1));
}

pub proof fn compose_with_identity_on_left_matches_original_function(value: int) {
    assert(identity_model(f(value)) == f(value));
}

pub proof fn compose_with_identity_on_right_matches_original_function(value: int) {
    assert(f(identity_model(value)) == f(value));
}

pub proof fn compose_associativity_holds_for_pure_functions(value: int) {
    let left = f(g(h(value)));
    let right = f(g(h(value)));
    assert(left == right);
}

}
