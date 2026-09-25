use lambars::control::Either;

use crate::kani_proof;

kani_proof!(either_is_left_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    assert!(Either::<i32, i32>::Left(value).is_left());
});
kani_proof!(either_is_left_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    assert!(!Either::<i32, i32>::Right(value).is_left());
});
kani_proof!(either_is_right_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    assert!(!Either::<i32, i32>::Left(value).is_right());
});
kani_proof!(either_is_right_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    assert!(Either::<i32, i32>::Right(value).is_right());
});
kani_proof!(either_left_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i32>::Left(value).left(), Some(value));
});
kani_proof!(either_left_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i32>::Right(value).left(), None);
});
kani_proof!(either_right_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i32>::Left(value).right(), None);
});
kani_proof!(either_right_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i32>::Right(value).right(), Some(value));
});
kani_proof!(either_left_ref_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    let either = Either::<i32, i32>::Left(value);
    assert_eq!(either.left_ref(), Some(&value));
});
kani_proof!(either_left_ref_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    let either = Either::<i32, i32>::Right(value);
    assert_eq!(either.left_ref(), None);
});
kani_proof!(either_right_ref_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    let either = Either::<i32, i32>::Left(value);
    assert_eq!(either.right_ref(), None);
});
kani_proof!(either_right_ref_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    let either = Either::<i32, i32>::Right(value);
    assert_eq!(either.right_ref(), Some(&value));
});
kani_proof!(either_map_left_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    let mapped = Either::<i32, i32>::Left(value).map_left(|x| x.wrapping_add(1));
    assert_eq!(mapped, Either::Left(value.wrapping_add(1)));
});
kani_proof!(either_map_left_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    let mapped = Either::<i32, i32>::Right(value).map_left(|x| x.wrapping_add(1));
    assert_eq!(mapped, Either::Right(value));
});
kani_proof!(either_map_right_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    let mapped = Either::<i32, i32>::Left(value).map_right(|x| x.wrapping_add(1));
    assert_eq!(mapped, Either::Left(value));
});
kani_proof!(either_map_right_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    let mapped = Either::<i32, i32>::Right(value).map_right(|x| x.wrapping_add(1));
    assert_eq!(mapped, Either::Right(value.wrapping_add(1)));
});
kani_proof!(either_bimap_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    let mapped = Either::<i32, i32>::Left(value)
        .bimap(|x| x.wrapping_add(1), |x| x.wrapping_sub(1));
    assert_eq!(mapped, Either::Left(value.wrapping_add(1)));
});
kani_proof!(either_bimap_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    let mapped = Either::<i32, i32>::Right(value)
        .bimap(|x| x.wrapping_add(1), |x| x.wrapping_sub(1));
    assert_eq!(mapped, Either::Right(value.wrapping_sub(1)));
});
kani_proof!(either_fold_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    let folded = Either::<i32, i32>::Left(value).fold(i64::from, |x| i64::from(x) + 1);
    assert_eq!(folded, i64::from(value));
});
kani_proof!(either_fold_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    let folded = Either::<i32, i32>::Right(value).fold(i64::from, |x| i64::from(x) + 1);
    assert_eq!(folded, i64::from(value) + 1);
});
kani_proof!(either_swap_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i64>::Left(value).swap(), Either::Right(value));
});
kani_proof!(either_swap_on_right_variant_has_documented_right_behavior, {
    let value: i64 = kani::any();
    assert_eq!(Either::<i32, i64>::Right(value).swap(), Either::Left(value));
});
kani_proof!(either_unwrap_left_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i32>::Left(value).unwrap_left(), value);
});
kani_proof!(either_unwrap_right_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i32>::Right(value).unwrap_right(), value);
});
kani_proof!(either_into_options_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i32>::Left(value).into_options(), (Some(value), None));
});
kani_proof!(either_into_options_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i32>::Right(value).into_options(), (None, Some(value)));
});
kani_proof!(either_iter_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    let either = Either::<i32, i32>::Left(value);
    let mut iter = either.iter();
    assert_eq!(iter.next(), None);
});
kani_proof!(either_iter_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    let either = Either::<i32, i32>::Right(value);
    let mut iter = either.iter();
    assert_eq!(iter.next(), Some(&value));
    assert_eq!(iter.next(), None);
});
kani_proof!(either_left_or_default_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i32>::Left(value).left_or_default(), value);
});
kani_proof!(either_left_or_default_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i32>::Right(value).left_or_default(), 0);
});
kani_proof!(either_right_or_default_on_left_variant_has_documented_left_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i32>::Left(value).right_or_default(), 0);
});
kani_proof!(either_right_or_default_on_right_variant_has_documented_right_behavior, {
    let value: i32 = kani::any();
    assert_eq!(Either::<i32, i32>::Right(value).right_or_default(), value);
});
kani_proof!(either_swap_twice_returns_original_value, {
    let choose_left: bool = kani::any();
    let value: i32 = kani::any();
    let either = if choose_left { Either::Left(value) } else { Either::Right(value) };
    assert_eq!(either.clone().swap().swap(), either);
});
kani_proof!(either_bimap_matches_map_left_followed_by_map_right, {
    let choose_left: bool = kani::any();
    let value: i32 = kani::any();
    let either = if choose_left { Either::Left(value) } else { Either::Right(value) };
    let expected = either.clone().map_left(|x| x.wrapping_add(1)).map_right(|x| x.wrapping_sub(1));
    let actual = either.bimap(|x| x.wrapping_add(1), |x| x.wrapping_sub(1));
    assert_eq!(actual, expected);
});
kani_proof!(either_into_iterator_left_variant_yields_zero_items, {
    let value: i32 = kani::any();
    let mut iter = Either::<i32, i32>::Left(value).into_iter();
    assert_eq!(iter.next(), None);
});
kani_proof!(either_into_iterator_right_variant_yields_exactly_one_item, {
    let value: i32 = kani::any();
    let mut iter = Either::<i32, i32>::Right(value).into_iter();
    assert_eq!(iter.next(), Some(value));
    assert_eq!(iter.next(), None);
});
kani_proof!(either_shared_iterator_left_variant_yields_zero_items, {
    let value: i32 = kani::any();
    let either = Either::<i32, i32>::Left(value);
    let mut iter = (&either).into_iter();
    assert_eq!(iter.next(), None);
});
kani_proof!(either_shared_iterator_right_variant_yields_reference_to_exact_right_value, {
    let value: i32 = kani::any();
    let either = Either::<i32, i32>::Right(value);
    let mut iter = (&either).into_iter();
    assert_eq!(iter.next(), Some(&value));
    assert_eq!(iter.next(), None);
});
