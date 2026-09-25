use lambars::typeclass::{Applicative, Flatten, Functor, Identity, Monad};

use crate::kani_proof;
use crate::symbolic::{option_bool, option_option_bool, result_bool, result_result_bool};

fn bool_not(value: bool) -> bool {
    !value
}

fn bool_identity(value: bool) -> bool {
    value
}

fn option_f(value: bool) -> Option<bool> {
    if value { Some(false) } else { None }
}

fn option_g(value: bool) -> Option<bool> {
    if value { None } else { Some(true) }
}

fn result_f(value: bool) -> Result<bool, bool> {
    if value { Ok(false) } else { Err(true) }
}

fn result_g(value: bool) -> Result<bool, bool> {
    if value { Err(false) } else { Ok(true) }
}

macro_rules! fixed_singleton_functor_proofs {
    (
        $identity_name:ident,
        $composition_name:ident,
        $shape_name:ident,
        $once_name:ident,
        $constructor:expr,
        $extract:expr
    ) => {
        kani_proof!($identity_name, {
            let value: bool = kani::any();
            let input = $constructor(value);
            let output = input.fmap(bool_identity);
            assert_eq!($extract(output), value);
        });

        kani_proof!($composition_name, {
            let value: bool = kani::any();
            let input = $constructor(value);
            let sequential = input.fmap(bool_not).fmap(bool_not);
            let composed = $constructor(value).fmap(|x| bool_not(bool_not(x)));
            assert_eq!($extract(sequential), $extract(composed));
        });

        kani_proof!($shape_name, {
            let value: bool = kani::any();
            let output = $constructor(value).fmap(bool_not);
            assert_eq!($extract(output), !value);
        });

        kani_proof!($once_name, {
            let value: bool = kani::any();
            let mut calls = 0_u8;
            let output = $constructor(value).fmap(|x| {
                calls += 1;
                !x
            });
            assert_eq!(calls, 1);
            assert_eq!($extract(output), !value);
        });
    };
}

kani_proof!(functor_option_identity_law_holds, {
    let input = option_bool();
    assert_eq!(input.fmap(bool_identity), input);
});

kani_proof!(functor_option_composition_law_holds, {
    let input = option_bool();
    let sequential = input.fmap(bool_not).fmap(bool_not);
    let composed = input.fmap(|x| bool_not(bool_not(x)));
    assert_eq!(sequential, composed);
});

kani_proof!(functor_option_fmap_preserves_structure_shape, {
    let input = option_bool();
    let expected_present = input.is_some();
    assert_eq!(input.fmap(bool_not).is_some(), expected_present);
});

kani_proof!(functor_option_fmap_invokes_mapping_function_exactly_once_per_present_element, {
    let input = option_bool();
    let expected = u8::from(input.is_some());
    let mut calls = 0_u8;
    let _ = input.fmap(|x| {
        calls += 1;
        !x
    });
    assert_eq!(calls, expected);
});

kani_proof!(functor_option_fmap_handles_empty_structure, {
    let input: Option<bool> = None;
    assert_eq!(input.fmap(bool_not), None);
});

kani_proof!(functor_result_identity_law_holds, {
    let input = result_bool();
    assert_eq!(input.fmap(bool_identity), input);
});

kani_proof!(functor_result_composition_law_holds, {
    let input = result_bool();
    let sequential = input.fmap(bool_not).fmap(bool_not);
    let composed = input.fmap(|x| bool_not(bool_not(x)));
    assert_eq!(sequential, composed);
});

kani_proof!(functor_result_fmap_preserves_structure_shape, {
    let input = result_bool();
    let expected_ok = input.is_ok();
    assert_eq!(input.fmap(bool_not).is_ok(), expected_ok);
});

kani_proof!(functor_result_fmap_invokes_mapping_function_exactly_once_per_present_element, {
    let input = result_bool();
    let expected = u8::from(input.is_ok());
    let mut calls = 0_u8;
    let _ = input.fmap(|x| {
        calls += 1;
        !x
    });
    assert_eq!(calls, expected);
});

kani_proof!(functor_result_fmap_handles_empty_structure, {
    let input: Result<bool, bool> = Err(kani::any());
    assert_eq!(input.fmap(bool_not), input);
});

fixed_singleton_functor_proofs!(
    functor_box_identity_law_holds,
    functor_box_composition_law_holds,
    functor_box_fmap_preserves_structure_shape,
    functor_box_fmap_invokes_mapping_function_exactly_once_per_present_element,
    Box::new,
    |value: Box<bool>| *value
);

fixed_singleton_functor_proofs!(
    functor_identity_identity_law_holds,
    functor_identity_composition_law_holds,
    functor_identity_fmap_preserves_structure_shape,
    functor_identity_fmap_invokes_mapping_function_exactly_once_per_present_element,
    Identity::new,
    |value: Identity<bool>| value.into_inner()
);

macro_rules! applicative_option_like_proofs {
    (
        $identity_name:ident,
        $homomorphism_name:ident,
        $map2_name:ident,
        $product_name:ident,
        $pure_name:ident,
        $context:ty,
        $pure_owner:ty,
        $pure_expected:expr
    ) => {
        kani_proof!($identity_name, {
            let value: bool = kani::any();
            let lifted_function: <$context as lambars::typeclass::TypeConstructor>::WithType<fn(bool) -> bool> =
                <$pure_owner>::pure(bool_identity as fn(bool) -> bool);
            let lifted_value: $context = <$pure_owner>::pure(value);
            assert_eq!(lifted_function.apply(lifted_value), $pure_expected(value));
        });

        kani_proof!($homomorphism_name, {
            let value: bool = kani::any();
            let lifted_function: <$context as lambars::typeclass::TypeConstructor>::WithType<fn(bool) -> bool> =
                <$pure_owner>::pure(bool_not as fn(bool) -> bool);
            let lifted_value: $context = <$pure_owner>::pure(value);
            assert_eq!(lifted_function.apply(lifted_value), $pure_expected(!value));
        });

        kani_proof!($map2_name, {
            let first: bool = kani::any();
            let second: bool = kani::any();
            let left: $context = <$pure_owner>::pure(first);
            let right: $context = <$pure_owner>::pure(second);
            assert_eq!(left.map2(right, |a, b| a ^ b), $pure_expected(first ^ second));
        });

        kani_proof!($product_name, {
            let first: bool = kani::any();
            let second: bool = kani::any();
            let left: $context = <$pure_owner>::pure(first);
            let right: $context = <$pure_owner>::pure(second);
            assert_eq!(left.product(right), <$pure_owner>::pure((first, second)));
        });

        kani_proof!($pure_name, {
            let value: bool = kani::any();
            let lifted: $context = <$pure_owner>::pure(value);
            assert_eq!(lifted, $pure_expected(value));
        });
    };
}

applicative_option_like_proofs!(
    applicative_option_identity_law_holds,
    applicative_option_homomorphism_law_holds,
    applicative_option_map2_matches_pure_function_application,
    applicative_option_product_preserves_left_then_right_value_order,
    applicative_option_pure_does_not_introduce_extra_effects,
    Option<bool>,
    Option<()>,
    Some
);

applicative_option_like_proofs!(
    applicative_result_identity_law_holds,
    applicative_result_homomorphism_law_holds,
    applicative_result_map2_matches_pure_function_application,
    applicative_result_product_preserves_left_then_right_value_order,
    applicative_result_pure_does_not_introduce_extra_effects,
    Result<bool, bool>,
    Result<(), bool>,
    Ok
);

kani_proof!(applicative_identity_identity_law_holds, {
    let value: bool = kani::any();
    let function: Identity<fn(bool) -> bool> =
        <Identity<()> as Applicative>::pure(bool_identity as fn(bool) -> bool);
    let input: Identity<bool> = <Identity<()> as Applicative>::pure(value);
    assert_eq!(function.apply(input), Identity::new(value));
});

kani_proof!(applicative_identity_homomorphism_law_holds, {
    let value: bool = kani::any();
    let function: Identity<fn(bool) -> bool> =
        <Identity<()> as Applicative>::pure(bool_not as fn(bool) -> bool);
    let input: Identity<bool> = <Identity<()> as Applicative>::pure(value);
    assert_eq!(function.apply(input), Identity::new(!value));
});

kani_proof!(applicative_identity_map2_matches_pure_function_application, {
    let first: bool = kani::any();
    let second: bool = kani::any();
    let left = Identity::new(first);
    let right = Identity::new(second);
    assert_eq!(left.map2(right, |a, b| a ^ b), Identity::new(first ^ second));
});

kani_proof!(applicative_identity_product_preserves_left_then_right_value_order, {
    let first: bool = kani::any();
    let second: bool = kani::any();
    assert_eq!(
        Identity::new(first).product(Identity::new(second)),
        Identity::new((first, second))
    );
});

kani_proof!(applicative_identity_pure_does_not_introduce_extra_effects, {
    let value: bool = kani::any();
    let lifted: Identity<bool> = <Identity<()> as Applicative>::pure(value);
    assert_eq!(lifted, Identity::new(value));
});

kani_proof!(monad_option_left_identity_law_holds, {
    let value: bool = kani::any();
    let left = <Option<()> as Applicative>::pure(value).flat_map(option_f);
    assert_eq!(left, option_f(value));
});

kani_proof!(monad_option_right_identity_law_holds, {
    let input = option_bool();
    let output = input.flat_map(|value| <Option<()> as Applicative>::pure(value));
    assert_eq!(output, input);
});

kani_proof!(monad_option_associativity_law_holds, {
    let input = option_bool();
    let left = input.flat_map(option_f).flat_map(option_g);
    let right = input.flat_map(|value| option_f(value).flat_map(option_g));
    assert_eq!(left, right);
});

kani_proof!(monad_option_flatten_matches_flat_map_identity, {
    let input = option_option_bool();
    let flattened = <Option<Option<bool>> as Flatten>::flatten(input);
    let flat_mapped = input.flat_map(bool_identity);
    assert_eq!(flattened, flat_mapped);
});

kani_proof!(monad_option_and_then_matches_flat_map, {
    let input = option_bool();
    assert_eq!(
        <Option<bool> as Monad>::and_then(input, option_f),
        input.flat_map(option_f)
    );
});

kani_proof!(monad_option_then_discards_first_value_but_preserves_first_effects, {
    let first = option_bool();
    let second = option_bool();
    let expected = if first.is_some() { second } else { None };
    assert_eq!(<Option<bool> as Monad>::then(first, second), expected);
});

kani_proof!(monad_result_left_identity_law_holds, {
    let value: bool = kani::any();
    let left: Result<bool, bool> = <Result<(), bool> as Applicative>::pure(value).flat_map(result_f);
    assert_eq!(left, result_f(value));
});

kani_proof!(monad_result_right_identity_law_holds, {
    let input = result_bool();
    let output = input.flat_map(|value| <Result<(), bool> as Applicative>::pure(value));
    assert_eq!(output, input);
});

kani_proof!(monad_result_associativity_law_holds, {
    let input = result_bool();
    let left = input.flat_map(result_f).flat_map(result_g);
    let right = input.flat_map(|value| result_f(value).flat_map(result_g));
    assert_eq!(left, right);
});

kani_proof!(monad_result_flatten_matches_flat_map_identity, {
    let input = result_result_bool();
    let flattened = <Result<Result<bool, bool>, bool> as Flatten>::flatten(input);
    let flat_mapped = input.flat_map(bool_identity);
    assert_eq!(flattened, flat_mapped);
});

kani_proof!(monad_result_and_then_matches_flat_map, {
    let input = result_bool();
    assert_eq!(
        <Result<bool, bool> as Monad>::and_then(input, result_f),
        input.flat_map(result_f)
    );
});

kani_proof!(monad_result_then_discards_first_value_but_preserves_first_effects, {
    let first = result_bool();
    let second = result_bool();
    let expected = match first {
        Ok(_) => second,
        Err(error) => Err(error),
    };
    assert_eq!(<Result<bool, bool> as Monad>::then(first, second), expected);
});

kani_proof!(monad_box_left_identity_law_holds, {
    let value: bool = kani::any();
    let left = <Box<()> as Applicative>::pure(value).flat_map(|x| Box::new(!x));
    assert_eq!(*left, !value);
});

kani_proof!(monad_box_right_identity_law_holds, {
    let value: bool = kani::any();
    let output = Box::new(value).flat_map(|x| <Box<()> as Applicative>::pure(x));
    assert_eq!(*output, value);
});

kani_proof!(monad_box_associativity_law_holds, {
    let value: bool = kani::any();
    let left = Box::new(value)
        .flat_map(|x| Box::new(!x))
        .flat_map(|x| Box::new(!x));
    let right = Box::new(value)
        .flat_map(|x| Box::new(!x).flat_map(|y| Box::new(!y)));
    assert_eq!(*left, *right);
});

kani_proof!(monad_box_flatten_matches_flat_map_identity, {
    let value: bool = kani::any();
    let nested = Box::new(Box::new(value));
    let flattened = <Box<Box<bool>> as Flatten>::flatten(nested);
    let flat_mapped = Box::new(Box::new(value)).flat_map(bool_identity);
    assert_eq!(*flattened, *flat_mapped);
});

kani_proof!(monad_box_and_then_matches_flat_map, {
    let value: bool = kani::any();
    let left = <Box<bool> as Monad>::and_then(Box::new(value), |x| Box::new(!x));
    let right = Box::new(value).flat_map(|x| Box::new(!x));
    assert_eq!(*left, *right);
});

kani_proof!(monad_box_then_discards_first_value_but_preserves_first_effects, {
    let first: bool = kani::any();
    let second: bool = kani::any();
    assert_eq!(
        *<Box<bool> as Monad>::then(Box::new(first), Box::new(second)),
        second
    );
});

kani_proof!(monad_identity_left_identity_law_holds, {
    let value: bool = kani::any();
    let left = <Identity<()> as Applicative>::pure(value).flat_map(|x| Identity::new(!x));
    assert_eq!(left, Identity::new(!value));
});

kani_proof!(monad_identity_right_identity_law_holds, {
    let value: bool = kani::any();
    let output = Identity::new(value).flat_map(|x| <Identity<()> as Applicative>::pure(x));
    assert_eq!(output, Identity::new(value));
});

kani_proof!(monad_identity_associativity_law_holds, {
    let value: bool = kani::any();
    let left = Identity::new(value)
        .flat_map(|x| Identity::new(!x))
        .flat_map(|x| Identity::new(!x));
    let right = Identity::new(value)
        .flat_map(|x| Identity::new(!x).flat_map(|y| Identity::new(!y)));
    assert_eq!(left, right);
});

kani_proof!(monad_identity_flatten_matches_flat_map_identity, {
    let value: bool = kani::any();
    let flattened = <Identity<Identity<bool>> as Flatten>::flatten(Identity::new(Identity::new(value)));
    let flat_mapped = Identity::new(Identity::new(value)).flat_map(bool_identity);
    assert_eq!(flattened, flat_mapped);
});

kani_proof!(monad_identity_and_then_matches_flat_map, {
    let value: bool = kani::any();
    let left = <Identity<bool> as Monad>::and_then(Identity::new(value), |x| Identity::new(!x));
    let right = Identity::new(value).flat_map(|x| Identity::new(!x));
    assert_eq!(left, right);
});

kani_proof!(monad_identity_then_discards_first_value_but_preserves_first_effects, {
    let first: bool = kani::any();
    let second: bool = kani::any();
    assert_eq!(
        <Identity<bool> as Monad>::then(Identity::new(first), Identity::new(second)),
        Identity::new(second)
    );
});
