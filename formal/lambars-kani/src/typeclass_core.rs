use lambars::control::Either;
use lambars::typeclass::{
    Alternative, Bifunctor, Bounded, Identity, Max, Min, Monoid, Product, Semigroup, Sum,
    TypeConstructor,
};

use crate::kani_proof;

fn symbolic_option_bool() -> Option<bool> {
    if kani::any::<bool>() {
        Some(kani::any::<bool>())
    } else {
        None
    }
}

fn symbolic_either_bool() -> Either<bool, bool> {
    if kani::any::<bool>() {
        Either::Left(kani::any::<bool>())
    } else {
        Either::Right(kani::any::<bool>())
    }
}

fn symbolic_result_bool() -> Result<bool, bool> {
    if kani::any::<bool>() {
        Ok(kani::any::<bool>())
    } else {
        Err(kani::any::<bool>())
    }
}

macro_rules! associative_ord_wrapper_proofs {
    (
        $assoc_name:ident,
        $order_name:ident,
        $wrapper:ident,
        $combine:expr
    ) => {
        kani_proof!($assoc_name, {
            let first: i64 = kani::any();
            let second: i64 = kani::any();
            let third: i64 = kani::any();
            let left = $wrapper::new(first)
                .combine($wrapper::new(second))
                .combine($wrapper::new(third));
            let right = $wrapper::new(first)
                .combine($wrapper::new(second).combine($wrapper::new(third)));
            assert_eq!(left, right);
        });

        kani_proof!($order_name, {
            let first: i64 = kani::any();
            let second: i64 = kani::any();
            let combined = $wrapper::new(first).combine($wrapper::new(second));
            assert_eq!(combined.into_inner(), $combine(first, second));
        });
    };
}

associative_ord_wrapper_proofs!(
    semigroup_max_i64_associativity_law_holds,
    semigroup_max_i64_combine_preserves_documented_operand_order,
    Max,
    std::cmp::max
);
associative_ord_wrapper_proofs!(
    semigroup_min_i64_associativity_law_holds,
    semigroup_min_i64_combine_preserves_documented_operand_order,
    Min,
    std::cmp::min
);

macro_rules! monoid_identity_and_empty_proofs {
    (
        $left_name:ident,
        $right_name:ident,
        $empty_name:ident,
        $ty:ty
    ) => {
        kani_proof!($left_name, {
            let value: i64 = kani::any();
            let wrapped: $ty = value.into();
            assert_eq!(<$ty>::empty().combine(wrapped), wrapped);
        });

        kani_proof!($right_name, {
            let value: i64 = kani::any();
            let wrapped: $ty = value.into();
            assert_eq!(wrapped.combine(<$ty>::empty()), wrapped);
        });

        kani_proof!($empty_name, {
            let combined = <$ty>::combine_all(std::iter::empty::<$ty>());
            assert_eq!(combined, <$ty>::empty());
        });
    };
}

monoid_identity_and_empty_proofs!(
    monoid_sum_i64_left_identity_law_holds,
    monoid_sum_i64_right_identity_law_holds,
    monoid_sum_i64_combine_all_empty_returns_identity,
    Sum<i64>
);
monoid_identity_and_empty_proofs!(
    monoid_product_i64_left_identity_law_holds,
    monoid_product_i64_right_identity_law_holds,
    monoid_product_i64_combine_all_empty_returns_identity,
    Product<i64>
);
monoid_identity_and_empty_proofs!(
    monoid_max_i64_left_identity_law_holds,
    monoid_max_i64_right_identity_law_holds,
    monoid_max_i64_combine_all_empty_returns_identity,
    Max<i64>
);
monoid_identity_and_empty_proofs!(
    monoid_min_i64_left_identity_law_holds,
    monoid_min_i64_right_identity_law_holds,
    monoid_min_i64_combine_all_empty_returns_identity,
    Min<i64>
);

kani_proof!(monoid_option_sum_i64_left_identity_law_holds, {
    let value: i64 = kani::any();
    let wrapped = Some(Sum::new(value));
    assert_eq!(<Option<Sum<i64>>>::empty().combine(wrapped), wrapped);
});

kani_proof!(monoid_option_sum_i64_right_identity_law_holds, {
    let value: i64 = kani::any();
    let wrapped = Some(Sum::new(value));
    assert_eq!(wrapped.combine(<Option<Sum<i64>>>::empty()), wrapped);
});

kani_proof!(monoid_option_sum_i64_combine_all_empty_returns_identity, {
    let combined = <Option<Sum<i64>>>::combine_all(std::iter::empty());
    assert_eq!(combined, None);
});

kani_proof!(alternative_option_left_identity_law_holds, {
    let value = symbolic_option_bool();
    assert_eq!(<Option<()>>::empty::<bool>().alt(value), value);
});

kani_proof!(alternative_option_right_identity_law_holds, {
    let value = symbolic_option_bool();
    assert_eq!(value.alt(<Option<()>>::empty::<bool>()), value);
});

kani_proof!(alternative_option_associativity_law_holds, {
    let first = symbolic_option_bool();
    let second = symbolic_option_bool();
    let third = symbolic_option_bool();
    assert_eq!(
        first.alt(second).alt(third),
        first.alt(second.alt(third))
    );
});

kani_proof!(alternative_option_left_distributivity_over_fmap_holds, {
    let first = symbolic_option_bool();
    let second = symbolic_option_bool();
    let left = first.alt(second).map(|value| !value);
    let right = first.map(|value| !value).alt(second.map(|value| !value));
    assert_eq!(left, right);
});

kani_proof!(alternative_option_guard_true_produces_success_unit, {
    assert_eq!(<Option<()>>::guard(true), Some(()));
});

kani_proof!(alternative_option_guard_false_produces_empty, {
    assert_eq!(<Option<()>>::guard(false), None);
});

kani_proof!(bifunctor_either_identity_law_holds, {
    let value = symbolic_either_bool();
    assert_eq!(value.clone().bimap(|x| x, |x| x), value);
});

kani_proof!(bifunctor_either_composition_law_holds, {
    let value = symbolic_either_bool();
    let sequential = value.clone().bimap(|x| !x, |x| !x).bimap(|x| !x, |x| !x);
    let composed = value.bimap(|x| !!x, |x| !!x);
    assert_eq!(sequential, composed);
});

kani_proof!(bifunctor_either_bimap_matches_map_left_then_map_right, {
    let value = symbolic_either_bool();
    let bimap = value.clone().bimap(|x| !x, |x| !x);
    let mapped = value.map_left(|x| !x).map_right(|x| !x);
    assert_eq!(bimap, mapped);
});

kani_proof!(bifunctor_either_first_changes_only_first_type_parameter, {
    let value = symbolic_either_bool();
    let expected = match value.clone() {
        Either::Left(x) => Either::Left(!x),
        Either::Right(x) => Either::Right(x),
    };
    assert_eq!(value.first(|x| !x), expected);
});

kani_proof!(bifunctor_either_second_changes_only_second_type_parameter, {
    let value = symbolic_either_bool();
    let expected = match value.clone() {
        Either::Left(x) => Either::Left(x),
        Either::Right(x) => Either::Right(!x),
    };
    assert_eq!(value.second(|x| !x), expected);
});

kani_proof!(bifunctor_result_identity_law_holds, {
    let value = symbolic_result_bool();
    assert_eq!(value.bimap(|x| x, |x| x), value);
});

kani_proof!(bifunctor_result_composition_law_holds, {
    let value = symbolic_result_bool();
    let sequential = value.bimap(|x| !x, |x| !x).bimap(|x| !x, |x| !x);
    let composed = value.bimap(|x| !!x, |x| !!x);
    assert_eq!(sequential, composed);
});

kani_proof!(bifunctor_result_bimap_matches_map_left_then_map_right, {
    let value = symbolic_result_bool();
    let bimap = value.bimap(|x| !x, |x| !x);
    let mapped = value.map_err(|x| !x).map(|x| !x);
    assert_eq!(bimap, mapped);
});

kani_proof!(bifunctor_result_first_changes_only_first_type_parameter, {
    let value = symbolic_result_bool();
    let expected = value.map_err(|x| !x);
    assert_eq!(value.first(|x| !x), expected);
});

kani_proof!(bifunctor_result_second_changes_only_second_type_parameter, {
    let value = symbolic_result_bool();
    let expected = value.map(|x| !x);
    assert_eq!(value.second(|x| !x), expected);
});

kani_proof!(bifunctor_tuple_identity_law_holds, {
    let value = (kani::any::<bool>(), kani::any::<bool>());
    assert_eq!(value.bimap(|x| x, |x| x), value);
});

kani_proof!(bifunctor_tuple_composition_law_holds, {
    let value = (kani::any::<bool>(), kani::any::<bool>());
    let sequential = value.bimap(|x| !x, |x| !x).bimap(|x| !x, |x| !x);
    let composed = value.bimap(|x| !!x, |x| !!x);
    assert_eq!(sequential, composed);
});

kani_proof!(bifunctor_tuple_bimap_matches_map_left_then_map_right, {
    let value = (kani::any::<bool>(), kani::any::<bool>());
    let bimap = value.bimap(|x| !x, |x| !x);
    let mapped = (value.0, value.1).first(|x| !x).second(|x| !x);
    assert_eq!(bimap, mapped);
});

kani_proof!(bifunctor_tuple_first_changes_only_first_type_parameter, {
    let first = kani::any::<bool>();
    let second = kani::any::<bool>();
    assert_eq!((first, second).first(|x| !x), (!first, second));
});

kani_proof!(bifunctor_tuple_second_changes_only_second_type_parameter, {
    let first = kani::any::<bool>();
    let second = kani::any::<bool>();
    assert_eq!((first, second).second(|x| !x), (first, !second));
});

kani_proof!(option_type_constructor_with_type_replaces_only_inner_type, {
    let value: <Option<i32> as TypeConstructor>::WithType<bool> = Some(true);
    assert_eq!(value, Some(true));
});

kani_proof!(result_type_constructor_with_type_preserves_error_type_and_replaces_success_type, {
    let value: <Result<i32, u16> as TypeConstructor>::WithType<bool> = Ok(true);
    let error: <Result<i32, u16> as TypeConstructor>::WithType<bool> = Err(7_u16);
    assert_eq!(value, Ok(true));
    assert_eq!(error, Err(7_u16));
});

kani_proof!(vec_type_constructor_with_type_replaces_only_element_type, {
    let value: <Vec<i32> as TypeConstructor>::WithType<bool> = Vec::new();
    assert!(value.is_empty());
});

kani_proof!(box_type_constructor_with_type_replaces_only_inner_type, {
    let value: <Box<i32> as TypeConstructor>::WithType<bool> = Box::new(true);
    assert!(*value);
});

kani_proof!(identity_type_constructor_with_type_replaces_only_inner_type, {
    let value: <Identity<i32> as TypeConstructor>::WithType<bool> = Identity::new(true);
    assert!(*value.as_inner());
});

macro_rules! wrapper_roundtrip_proof {
    ($name:ident, $wrapper:ident) => {
        kani_proof!($name, {
            let value: i64 = kani::any();
            let mut wrapped = $wrapper::new(value);
            assert_eq!(*wrapped.as_inner(), value);
            *wrapped.as_inner_mut() = value.wrapping_add(1);
            assert_eq!(wrapped.into_inner(), value.wrapping_add(1));
        });
    };
}

kani_proof!(identity_new_into_inner_as_inner_and_as_inner_mut_are_consistent, {
    let value: i64 = kani::any();
    let mut wrapped = Identity::new(value);
    assert_eq!(*wrapped.as_inner(), value);
    *wrapped.as_inner_mut() = value.wrapping_add(1);
    assert_eq!(wrapped.into_inner(), value.wrapping_add(1));
});

wrapper_roundtrip_proof!(
    sum_new_into_inner_as_inner_and_as_inner_mut_are_consistent,
    Sum
);
wrapper_roundtrip_proof!(
    product_new_into_inner_as_inner_and_as_inner_mut_are_consistent,
    Product
);
wrapper_roundtrip_proof!(
    max_new_into_inner_as_inner_and_as_inner_mut_are_consistent,
    Max
);
wrapper_roundtrip_proof!(
    min_new_into_inner_as_inner_and_as_inner_mut_are_consistent,
    Min
);

kani_proof!(bounded_integer_min_value_matches_primitive_min, {
    assert_eq!(<i8 as Bounded>::MIN_VALUE, i8::MIN);
    assert_eq!(<i16 as Bounded>::MIN_VALUE, i16::MIN);
    assert_eq!(<i32 as Bounded>::MIN_VALUE, i32::MIN);
    assert_eq!(<i64 as Bounded>::MIN_VALUE, i64::MIN);
    assert_eq!(<i128 as Bounded>::MIN_VALUE, i128::MIN);
    assert_eq!(<isize as Bounded>::MIN_VALUE, isize::MIN);
    assert_eq!(<u8 as Bounded>::MIN_VALUE, u8::MIN);
    assert_eq!(<u16 as Bounded>::MIN_VALUE, u16::MIN);
    assert_eq!(<u32 as Bounded>::MIN_VALUE, u32::MIN);
    assert_eq!(<u64 as Bounded>::MIN_VALUE, u64::MIN);
    assert_eq!(<u128 as Bounded>::MIN_VALUE, u128::MIN);
    assert_eq!(<usize as Bounded>::MIN_VALUE, usize::MIN);
});

kani_proof!(bounded_integer_max_value_matches_primitive_max, {
    assert_eq!(<i8 as Bounded>::MAX_VALUE, i8::MAX);
    assert_eq!(<i16 as Bounded>::MAX_VALUE, i16::MAX);
    assert_eq!(<i32 as Bounded>::MAX_VALUE, i32::MAX);
    assert_eq!(<i64 as Bounded>::MAX_VALUE, i64::MAX);
    assert_eq!(<i128 as Bounded>::MAX_VALUE, i128::MAX);
    assert_eq!(<isize as Bounded>::MAX_VALUE, isize::MAX);
    assert_eq!(<u8 as Bounded>::MAX_VALUE, u8::MAX);
    assert_eq!(<u16 as Bounded>::MAX_VALUE, u16::MAX);
    assert_eq!(<u32 as Bounded>::MAX_VALUE, u32::MAX);
    assert_eq!(<u64 as Bounded>::MAX_VALUE, u64::MAX);
    assert_eq!(<u128 as Bounded>::MAX_VALUE, u128::MAX);
    assert_eq!(<usize as Bounded>::MAX_VALUE, usize::MAX);
});

kani_proof!(bounded_float_min_value_is_negative_infinity, {
    assert_eq!(<f32 as Bounded>::MIN_VALUE, f32::NEG_INFINITY);
    assert_eq!(<f64 as Bounded>::MIN_VALUE, f64::NEG_INFINITY);
});

kani_proof!(bounded_float_max_value_is_positive_infinity, {
    assert_eq!(<f32 as Bounded>::MAX_VALUE, f32::INFINITY);
    assert_eq!(<f64 as Bounded>::MAX_VALUE, f64::INFINITY);
});

kani_proof!(bounded_char_min_value_is_null_character, {
    assert_eq!(<char as Bounded>::MIN_VALUE, '\0');
});

kani_proof!(bounded_char_max_value_matches_char_max, {
    assert_eq!(<char as Bounded>::MAX_VALUE, char::MAX);
});

kani_proof!(bounded_bool_min_value_is_false_and_max_value_is_true, {
    assert!(!<bool as Bounded>::MIN_VALUE);
    assert!(<bool as Bounded>::MAX_VALUE);
});
