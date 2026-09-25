use lambars::control::{ConcurrentLazy, concurrent_lazy_reentry_matches};

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
