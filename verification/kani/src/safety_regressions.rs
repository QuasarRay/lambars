use lambars::control::{ConcurrentLazy, concurrent_lazy_reentry_matches};
use lambars::persistent::persistent_hashmap_generation_successor;

fn distinct_identity_is_not_reentrant(active: usize, candidate: usize) -> bool {
    active != candidate && !concurrent_lazy_reentry_matches(active, candidate)
}

fn same_identity_is_reentrant(identity: usize) -> bool {
    concurrent_lazy_reentry_matches(identity, identity)
}

#[cfg(test)]
mod runtime_regressions {
    use super::*;

    #[test]
    fn sc002_distinct_instance_identity_is_not_reentrant() {
        assert!(distinct_identity_is_not_reentrant(1, 2));
        assert!(same_identity_is_reentrant(1));
    }

    #[test]
    fn sc002_nested_distinct_concurrent_lazy_executes_without_poisoning() {
        let inner = ConcurrentLazy::new(|| 41u8);
        let outer = ConcurrentLazy::new(|| *inner.force() + 1);

        assert_eq!(*outer.force(), 42);
        assert_eq!(*inner.force(), 41);
        assert!(!outer.is_poisoned());
        assert!(!inner.is_poisoned());
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// SC-002 regression: the production predicate cannot classify two distinct
    /// ConcurrentLazy instance identities as re-entry.
    #[kani::proof]
    fn sc002_distinct_instances_are_never_reentry() {
        let active: usize = kani::any();
        let candidate: usize = kani::any();
        kani::assume(active != candidate);
        assert!(distinct_identity_is_not_reentrant(active, candidate));
    }

    /// SC-002 regression: revisiting the same instance is always classified as
    /// re-entry, which is the cycle that must be rejected before waiting.
    #[kani::proof]
    fn sc002_same_instance_is_always_reentry() {
        let identity: usize = kani::any();
        assert!(same_identity_is_reentrant(identity));
    }
}


fn generation_successor_is_nonzero_and_monotonic(current: u64) -> bool {
    match persistent_hashmap_generation_successor(current) {
        Some(next) => next > current && next != 0,
        None => current == u64::MAX,
    }
}

#[cfg(test)]
mod hash_generation_runtime_regressions {
    use super::*;

    #[test]
    fn sc030_generation_successor_never_wraps_to_zero() {
        assert!(generation_successor_is_nonzero_and_monotonic(1));
        assert!(generation_successor_is_nonzero_and_monotonic(u64::MAX - 1));
        assert!(generation_successor_is_nonzero_and_monotonic(u64::MAX));
    }
}

#[cfg(kani)]
mod hash_generation_kani_proofs {
    use super::*;

    /// SC-030 regression: every successful successor is strictly larger and non-zero;
    /// exhaustion is represented as None rather than wrapping to the shared sentinel.
    #[kani::proof]
    fn sc030_generation_token_never_wraps_to_shared_zero() {
        let current: u64 = kani::any();
        assert!(generation_successor_is_nonzero_and_monotonic(current));
    }
}


#[cfg(any(test, kani))]
mod prism_regressions {
    use lambars::optics::OwnedPrism;
    use lambars_derive::Prisms;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Prisms)]
    enum PairVariant {
        Pair(u8, u8),
        Other,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Prisms)]
    enum StructVariant {
        Pair { left: u8, right: u8 },
        Other,
    }

    fn tuple_owned_prism_roundtrip(left: u8, right: u8) -> bool {
        let prism = PairVariant::pair_prism();
        let value = (left, right);
        prism.preview_owned(prism.review(value)) == Some(value)
    }

    fn struct_owned_prism_roundtrip(left: u8, right: u8) -> bool {
        let prism = StructVariant::pair_prism();
        let value = (left, right);
        prism.preview_owned(prism.review(value)) == Some(value)
    }

    #[cfg(test)]
    #[test]
    fn sc003_derived_owned_prisms_satisfy_roundtrip_law() {
        assert!(tuple_owned_prism_roundtrip(1, 2));
        assert!(struct_owned_prism_roundtrip(3, 4));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc003_tuple_variant_derive_satisfies_owned_preview_review() {
        let left: u8 = kani::any();
        let right: u8 = kani::any();
        assert!(tuple_owned_prism_roundtrip(left, right));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc003_struct_variant_derive_satisfies_owned_preview_review() {
        let left: u8 = kani::any();
        let right: u8 = kani::any();
        assert!(struct_owned_prism_roundtrip(left, right));
    }
}


#[cfg(any(test, kani))]
mod ordered_unique_set_regressions {
    use lambars::persistent::OrderedUniqueSet;

    fn normalized_three(a: u8, b: u8, c: u8) -> bool {
        let set = OrderedUniqueSet::from_sorted_vec(vec![a, b, c, a]);
        let values = set.to_sorted_vec();

        values.windows(2).all(|window| window[0] < window[1])
            && values.contains(&a)
            && values.contains(&b)
            && values.contains(&c)
            && values.len() <= 3
    }

    #[cfg(test)]
    #[test]
    fn sc004_safe_constructor_preserves_order_and_uniqueness() {
        assert!(normalized_three(3, 1, 2));
        assert!(normalized_three(1, 1, 1));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[kani::unwind(16)]
    fn sc004_safe_constructor_cannot_create_unsorted_or_duplicate_state() {
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        let c: u8 = kani::any();
        assert!(normalized_three(a, b, c));
    }
}
