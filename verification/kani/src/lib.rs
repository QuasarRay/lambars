#![deny(unsafe_code)]
#![allow(dead_code)]

#[cfg(any(test, kani))]
mod boundary_families;

#[cfg(any(test, kani))]
mod safety_regressions;

use lambars::control::Either;
use lambars::typeclass::{Functor, Semigroup};

fn option_functor_identity(value: Option<u8>) -> bool {
    value.fmap(|x| x) == value
}

fn result_functor_identity(value: Result<u8, u8>) -> bool {
    value.fmap(|x| x) == value
}

fn either_swap_involution(is_left: bool, left: u8, right: u8) -> bool {
    let value = if is_left {
        Either::Left(left)
    } else {
        Either::Right(right)
    };
    value.clone().swap().swap() == value
}

fn string_semigroup_associativity(a: String, b: String, c: String) -> bool {
    let left = a.clone().combine(b.clone()).combine(c.clone());
    let right = a.combine(b.combine(c));
    left == right
}

#[cfg(test)]
mod runtime_regressions {
    use super::*;

    #[test]
    fn functor_option_identity_law_holds() {
        for value in [None, Some(0), Some(1), Some(u8::MAX)] {
            assert!(option_functor_identity(value));
        }
    }

    #[test]
    fn functor_result_identity_law_holds() {
        for value in [Ok(0), Ok(u8::MAX), Err(0), Err(u8::MAX)] {
            assert!(result_functor_identity(value));
        }
    }

    #[test]
    fn either_swap_twice_returns_original_variant_and_value() {
        assert!(either_swap_involution(true, 7, 9));
        assert!(either_swap_involution(false, 7, 9));
    }

    #[test]
    fn semigroup_string_associativity_law_holds() {
        assert!(string_semigroup_associativity(
            "a".into(),
            "b".into(),
            "c".into()
        ));
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn functor_option_identity_law_holds() {
        let value: Option<u8> = kani::any();
        assert!(option_functor_identity(value));
    }

    #[kani::proof]
    fn functor_result_identity_law_holds() {
        let value: Result<u8, u8> = kani::any();
        assert!(result_functor_identity(value));
    }

    #[kani::proof]
    fn either_swap_twice_returns_original_variant_and_value() {
        let is_left: bool = kani::any();
        let left: u8 = kani::any();
        let right: u8 = kani::any();
        assert!(either_swap_involution(is_left, left, right));
    }

    // String is intentionally kept as a runtime regression here. Kani's
    // unbounded String domain needs an explicit bounded generator before this
    // canonical property can be claimed as model-checked.
}
