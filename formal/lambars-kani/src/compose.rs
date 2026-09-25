use std::mem::size_of;

use lambars::compose::{Placeholder, constant, flip, identity};
use lambars::compose;

use crate::kani_proof;

kani_proof!(identity_returns_exact_input_value, {
    let value: i64 = kani::any();
    assert_eq!(identity(value), value);
});

kani_proof!(identity_preserves_reference_identity_for_references, {
    let value: i64 = kani::any();
    let reference = &value;
    let returned = identity(reference);
    assert!(std::ptr::eq(reference, returned));
});

kani_proof!(constant_returns_cloned_constant_for_every_input, {
    let constant_value: i32 = kani::any();
    let input: u64 = kani::any();
    let constant_function = constant::<i32, u64>(constant_value);
    assert_eq!(constant_function(input), constant_value);
});

kani_proof!(flip_swaps_binary_function_argument_order, {
    let a: i32 = kani::any();
    let b: i32 = kani::any();
    let subtract = |left: i32, right: i32| left.wrapping_sub(right);
    let flipped = flip(subtract);
    assert_eq!(flipped(a, b), subtract(b, a));
});

kani_proof!(flip_applied_twice_matches_original_binary_function, {
    let a: i32 = kani::any();
    let b: i32 = kani::any();
    let combine = |left: i32, right: i32| left.wrapping_mul(31).wrapping_add(right);
    let double_flipped = flip(flip(combine));
    assert_eq!(double_flipped(a, b), combine(a, b));
});

kani_proof!(placeholder_constant_has_expected_zero_sized_marker_semantics, {
    assert_eq!(size_of::<Placeholder>(), 0);
});

kani_proof!(compose_two_functions_applies_functions_right_to_left, {
    let value: i32 = kani::any();
    fn increment(value: i32) -> i32 { value.wrapping_add(1) }
    fn double(value: i32) -> i32 { value.wrapping_mul(2) }
    let composed = compose!(increment, double);
    assert_eq!(composed(value), increment(double(value)));
});

kani_proof!(compose_three_functions_applies_functions_right_to_left, {
    let value: i32 = kani::any();
    fn increment(value: i32) -> i32 { value.wrapping_add(1) }
    fn double(value: i32) -> i32 { value.wrapping_mul(2) }
    fn negate(value: i32) -> i32 { value.wrapping_neg() }
    let composed = compose!(increment, double, negate);
    assert_eq!(composed(value), increment(double(negate(value))));
});

kani_proof!(compose_many_functions_preserves_right_to_left_order, {
    let value: i32 = kani::any();
    fn f1(value: i32) -> i32 { value.wrapping_add(1) }
    fn f2(value: i32) -> i32 { value.wrapping_mul(3) }
    fn f3(value: i32) -> i32 { value.wrapping_sub(7) }
    fn f4(value: i32) -> i32 { value.rotate_left(5) }
    let composed = compose!(f1, f2, f3, f4);
    assert_eq!(composed(value), f1(f2(f3(f4(value)))));
});

kani_proof!(compose_with_identity_on_left_matches_original_function, {
    let value: i32 = kani::any();
    fn f(value: i32) -> i32 { value.rotate_left(3).wrapping_add(9) }
    let composed = compose!(identity, f);
    assert_eq!(composed(value), f(value));
});

kani_proof!(compose_with_identity_on_right_matches_original_function, {
    let value: i32 = kani::any();
    fn f(value: i32) -> i32 { value.rotate_left(3).wrapping_add(9) }
    let composed = compose!(f, identity);
    assert_eq!(composed(value), f(value));
});

kani_proof!(compose_associativity_holds_for_pure_functions, {
    let value: i32 = kani::any();
    fn f(value: i32) -> i32 { value.wrapping_add(5) }
    fn g(value: i32) -> i32 { value.wrapping_mul(3) }
    fn h(value: i32) -> i32 { value.rotate_right(2) }
    let left = compose!(f, compose!(g, h));
    let right = compose!(compose!(f, g), h);
    assert_eq!(left(value), right(value));
});
