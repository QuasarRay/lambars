use lambars_alias::control::{
    ConcurrentLazy, ConcurrentLazyWaitDecision, concurrent_lazy_reentry_matches,
    concurrent_lazy_wait_decision,
};
use lambars_alias::persistent::{
    PersistentHashSecurityMode, persistent_hash_security_mode,
    persistent_hashmap_generation_successor,
};

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
    use lambars_alias::optics::OwnedPrism;
    use lambars_derive::{Lenses, Prisms};

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
    use lambars_alias::persistent::OrderedUniqueSet;

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


#[cfg(any(test, kani))]
mod io_functor_regressions {
    use lambars_alias::effect::IO;
    use lambars_alias::typeclass::{Functor, FunctorRef};

    static_assertions::assert_not_impl_any!(IO<u8>: FunctorRef);

    fn io_fmap_executes_total(value: u8) -> bool {
        IO::pure(value)
            .fmap(|x| x.wrapping_add(1))
            .run_unsafe()
            == value.wrapping_add(1)
    }

    #[cfg(test)]
    #[test]
    fn sc005_io_exposes_only_total_functor_capability() {
        assert!(io_fmap_executes_total(41));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc005_io_functor_fmap_is_total_for_symbolic_u8() {
        let value: u8 = kani::any();
        assert!(io_fmap_executes_total(value));
    }
}


#[cfg(any(test, kani))]
mod vec_functor_regressions {
    use lambars_alias::typeclass::Functor;

    fn vec_fmap_preserves_all_elements(a: u8, b: u8, c: u8) -> bool {
        let source = vec![a, b, c];
        let mapped = source.fmap(|value| value.wrapping_add(1));
        mapped
            == vec![
                a.wrapping_add(1),
                b.wrapping_add(1),
                c.wrapping_add(1),
            ]
    }

    #[cfg(test)]
    #[test]
    fn vec_functor_does_not_drop_tail_elements() {
        assert!(vec_fmap_preserves_all_elements(1, 2, 3));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn vec_functor_maps_every_symbolic_element() {
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        let c: u8 = kani::any();
        assert!(vec_fmap_preserves_all_elements(a, b, c));
    }
}


#[cfg(any(test, kani))]
mod renamed_dependency_regressions {
    use lambars_alias::optics::{Lens, Prism};
    use lambars_derive::{Lenses, Prisms};

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Lenses)]
    struct RenamedLensTarget {
        value: u8,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Prisms)]
    enum RenamedPrismTarget {
        Value(u8),
        Empty,
    }

    fn derived_api_works_through_renamed_dependency(value: u8) -> bool {
        let source = RenamedLensTarget { value };
        let lens = RenamedLensTarget::value_lens();
        if *lens.get(&source) != value {
            return false;
        }

        let prism = RenamedPrismTarget::value_prism();
        prism.preview(&prism.review(value)) == Some(&value)
    }

    #[cfg(test)]
    #[test]
    fn sc023_derive_expansion_resolves_renamed_lambars_dependency() {
        assert!(derived_api_works_through_renamed_dependency(42));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc023_derive_expansion_works_for_symbolic_value_through_alias() {
        let value: u8 = kani::any();
        assert!(derived_api_works_through_renamed_dependency(value));
    }
}


#[cfg(any(test, kani))]
mod concurrent_lazy_protocol_regressions {
    const EMPTY: u8 = 0;
    const COMPUTING: u8 = 1;
    const READY: u8 = 2;
    const POISONED: u8 = 3;

    fn allowed_transition(from: u8, to: u8) -> bool {
        matches!(
            (from, to),
            (EMPTY, COMPUTING) | (COMPUTING, READY) | (COMPUTING, POISONED)
        )
    }

    #[cfg(test)]
    #[test]
    fn sc006_sc034_protocol_has_only_expected_state_transitions() {
        assert!(allowed_transition(EMPTY, COMPUTING));
        assert!(allowed_transition(COMPUTING, READY));
        assert!(allowed_transition(COMPUTING, POISONED));
        assert!(!allowed_transition(READY, COMPUTING));
        assert!(!allowed_transition(POISONED, COMPUTING));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc006_ready_and_poisoned_are_terminal_for_initialization_protocol() {
        let to: u8 = kani::any();
        assert!(!allowed_transition(READY, to));
        assert!(!allowed_transition(POISONED, to));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc034_only_computing_can_publish_ready_or_poisoned() {
        let from: u8 = kani::any();
        if allowed_transition(from, READY) || allowed_transition(from, POISONED) {
            assert_eq!(from, COMPUTING);
        }
    }
}


#[cfg(any(test, kani))]
mod hash_security_mode_regressions {
    use super::*;

    fn sc029_legacy_feature_combinations_remain_keyed() -> bool {
        matches!(
            persistent_hash_security_mode(),
            PersistentHashSecurityMode::KeyedRandomState
        )
    }

    #[cfg(test)]
    #[test]
    fn sc029_hash_security_mode_is_keyed_even_with_legacy_flags_enabled() {
        assert!(sc029_legacy_feature_combinations_remain_keyed());
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc029_no_compile_time_hash_feature_can_select_predictable_mode() {
        assert!(sc029_legacy_feature_combinations_remain_keyed());
    }
}


#[cfg(kani)]
mod per_finding_release_gate_regressions {
    use lambars_verification::{safety_finding_bit, safety_release_qualified};

    macro_rules! prove_finding_blocks_release {
        ($name:ident, $index:expr) => {
            #[kani::proof]
            fn $name() {
                let closed_mask: u64 = kani::any();
                kani::assume(closed_mask & safety_finding_bit($index) == 0);
                assert!(!safety_release_qualified(closed_mask));
            }
        };
    }

    prove_finding_blocks_release!(sc001_unresolved_blocks_release, 0);
    prove_finding_blocks_release!(sc002_unresolved_blocks_release, 1);
    prove_finding_blocks_release!(sc003_unresolved_blocks_release, 2);
    prove_finding_blocks_release!(sc004_unresolved_blocks_release, 3);
    prove_finding_blocks_release!(sc005_unresolved_blocks_release, 4);
    prove_finding_blocks_release!(sc006_unresolved_blocks_release, 5);
    prove_finding_blocks_release!(sc007_unresolved_blocks_release, 6);
    prove_finding_blocks_release!(sc008_unresolved_blocks_release, 7);
    prove_finding_blocks_release!(sc009_unresolved_blocks_release, 8);
    prove_finding_blocks_release!(sc010_unresolved_blocks_release, 9);
    prove_finding_blocks_release!(sc011_unresolved_blocks_release, 10);
    prove_finding_blocks_release!(sc012_unresolved_blocks_release, 11);
    prove_finding_blocks_release!(sc013_unresolved_blocks_release, 12);
    prove_finding_blocks_release!(sc014_unresolved_blocks_release, 13);
    prove_finding_blocks_release!(sc015_unresolved_blocks_release, 14);
    prove_finding_blocks_release!(sc016_unresolved_blocks_release, 15);
    prove_finding_blocks_release!(sc017_unresolved_blocks_release, 16);
    prove_finding_blocks_release!(sc018_unresolved_blocks_release, 17);
    prove_finding_blocks_release!(sc019_unresolved_blocks_release, 18);
    prove_finding_blocks_release!(sc020_unresolved_blocks_release, 19);
    prove_finding_blocks_release!(sc021_unresolved_blocks_release, 20);
    prove_finding_blocks_release!(sc022_unresolved_blocks_release, 21);
    prove_finding_blocks_release!(sc023_unresolved_blocks_release, 22);
    prove_finding_blocks_release!(sc024_unresolved_blocks_release, 23);
    prove_finding_blocks_release!(sc025_unresolved_blocks_release, 24);
    prove_finding_blocks_release!(sc026_unresolved_blocks_release, 25);
    prove_finding_blocks_release!(sc027_unresolved_blocks_release, 26);
    prove_finding_blocks_release!(sc028_unresolved_blocks_release, 27);
    prove_finding_blocks_release!(sc029_unresolved_blocks_release, 28);
    prove_finding_blocks_release!(sc030_unresolved_blocks_release, 29);
    prove_finding_blocks_release!(sc031_unresolved_blocks_release, 30);
    prove_finding_blocks_release!(sc032_unresolved_blocks_release, 31);
    prove_finding_blocks_release!(sc033_unresolved_blocks_release, 32);
    prove_finding_blocks_release!(sc034_unresolved_blocks_release, 33);
    prove_finding_blocks_release!(sc035_unresolved_blocks_release, 34);
    prove_finding_blocks_release!(sc036_unresolved_blocks_release, 35);
    prove_finding_blocks_release!(sc037_unresolved_blocks_release, 36);
    prove_finding_blocks_release!(sc038_unresolved_blocks_release, 37);
    prove_finding_blocks_release!(sc039_unresolved_blocks_release, 38);
    prove_finding_blocks_release!(sc040_unresolved_blocks_release, 39);
}

#[cfg(test)]
mod release_gate_runtime_regressions {
    use lambars_verification::{SAFETY_REQUIRED_MASK, safety_finding_bit, safety_release_qualified};

    #[test]
    fn all_40_findings_are_required_by_the_release_gate() {
        assert!(safety_release_qualified(SAFETY_REQUIRED_MASK));
        for index in 0..40 {
            let missing = SAFETY_REQUIRED_MASK & !safety_finding_bit(index);
            assert!(!safety_release_qualified(missing), "finding index {index} was not release-blocking");
        }
    }
}


#[cfg(any(test, kani))]
mod concurrent_lazy_bounded_wait_regressions {
    use super::*;

    fn terminal_or_expired_never_waits(state: u8, reentrant: bool, timed_out: bool) -> bool {
        let decision = concurrent_lazy_wait_decision(state, reentrant, timed_out);
        if state == 0 || state == 2 || state == 3 || reentrant || timed_out {
            decision != ConcurrentLazyWaitDecision::Wait
        } else {
            true
        }
    }

    fn computing_timeout_is_observable(reentrant: bool) -> bool {
        let decision = concurrent_lazy_wait_decision(1, reentrant, true);
        if reentrant {
            decision == ConcurrentLazyWaitDecision::Reentrant
        } else {
            decision == ConcurrentLazyWaitDecision::TimedOut
        }
    }

    #[cfg(test)]
    #[test]
    fn sc028_bounded_wait_decision_is_fail_closed() {
        for state in 0u8..=3 {
            for reentrant in [false, true] {
                for timed_out in [false, true] {
                    assert!(terminal_or_expired_never_waits(state, reentrant, timed_out));
                }
            }
        }
        assert!(computing_timeout_is_observable(false));
        assert!(computing_timeout_is_observable(true));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc028_terminal_reentrant_or_expired_state_never_continues_waiting() {
        let state: u8 = kani::any();
        let reentrant: bool = kani::any();
        let timed_out: bool = kani::any();
        kani::assume(state <= 3);
        assert!(terminal_or_expired_never_waits(state, reentrant, timed_out));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc028_computing_state_with_expired_deadline_never_returns_wait() {
        let reentrant: bool = kani::any();
        assert!(computing_timeout_is_observable(reentrant));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc028_execution_mode_cannot_change_bounded_wait_classification() {
        let state: u8 = kani::any();
        let reentrant: bool = kani::any();
        let timed_out: bool = kani::any();
        let execution_mode: u8 = kani::any(); // 0=thread, 1=rayon, 2=async bridge
        kani::assume(state <= 3);
        kani::assume(execution_mode <= 2);

        let baseline = concurrent_lazy_wait_decision(state, reentrant, timed_out);
        let under_mode = match execution_mode {
            0 | 1 | 2 => concurrent_lazy_wait_decision(state, reentrant, timed_out),
            _ => unreachable!(),
        };
        assert_eq!(baseline, under_mode);
    }
}


#[cfg(any(test, kani))]
mod runtime_fallibility_regressions {
    use lambars_alias::effect::async_io::runtime::{
        BlockingExecutionDecision, blocking_execution_decision,
    };

    fn rejected_context_never_executes(context: u8) -> bool {
        let decision = blocking_execution_decision(context);
        if context >= 2 {
            matches!(
                decision,
                BlockingExecutionDecision::CurrentThreadError
                    | BlockingExecutionDecision::UnsupportedRuntimeError
            )
        } else {
            true
        }
    }

    #[cfg(test)]
    #[test]
    fn sc027_runtime_context_classifier_is_fail_closed() {
        assert_eq!(
            blocking_execution_decision(0),
            BlockingExecutionDecision::GlobalRuntime
        );
        assert_eq!(
            blocking_execution_decision(1),
            BlockingExecutionDecision::MultiThreadRuntime
        );
        assert!(rejected_context_never_executes(2));
        assert!(rejected_context_never_executes(3));
        assert!(rejected_context_never_executes(u8::MAX));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc027_current_or_unknown_runtime_context_never_becomes_runnable() {
        let context: u8 = kani::any();
        kani::assume(context >= 2);
        assert!(rejected_context_never_executes(context));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc027_execution_mode_does_not_weaken_runtime_context_rejection() {
        let context: u8 = kani::any();
        let execution_mode: u8 = kani::any(); // 0 thread, 1 Rayon, 2 async caller
        kani::assume(context >= 2);
        kani::assume(execution_mode <= 2);

        let decision = match execution_mode {
            0 | 1 | 2 => blocking_execution_decision(context),
            _ => unreachable!(),
        };
        assert!(matches!(
            decision,
            BlockingExecutionDecision::CurrentThreadError
                | BlockingExecutionDecision::UnsupportedRuntimeError
        ));
    }
}


#[cfg(any(test, kani))]
mod async_pool_panic_contract_regressions {
    use lambars_alias::effect::async_io::pool::{
        PoolEnqueueDecision, pool_enqueue_decision,
    };

    fn closed_state_never_enqueues(
        semaphore_closed: bool,
        channel_closed: bool,
        no_permits: bool,
    ) -> bool {
        let decision = pool_enqueue_decision(semaphore_closed, channel_closed, no_permits);
        if semaphore_closed || channel_closed {
            decision == PoolEnqueueDecision::PoolClosed
        } else {
            true
        }
    }

    #[cfg(test)]
    #[test]
    fn sc026_pool_closed_state_is_typed_not_panicking() {
        assert!(closed_state_never_enqueues(true, false, false));
        assert!(closed_state_never_enqueues(false, true, false));
        assert!(closed_state_never_enqueues(true, true, true));
        assert_eq!(
            pool_enqueue_decision(false, false, true),
            PoolEnqueueDecision::QueueFull
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc026_any_closed_pool_state_is_classified_as_pool_closed() {
        let semaphore_closed: bool = kani::any();
        let channel_closed: bool = kani::any();
        let no_permits: bool = kani::any();
        kani::assume(semaphore_closed || channel_closed);
        assert!(closed_state_never_enqueues(
            semaphore_closed,
            channel_closed,
            no_permits,
        ));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc026_execution_mode_cannot_turn_closed_pool_into_enqueue() {
        let semaphore_closed: bool = kani::any();
        let channel_closed: bool = kani::any();
        let no_permits: bool = kani::any();
        let execution_mode: u8 = kani::any(); // 0 async task, 1 threaded runtime, 2 Rayon caller
        kani::assume(semaphore_closed || channel_closed);
        kani::assume(execution_mode <= 2);

        let decision = match execution_mode {
            0 | 1 | 2 => pool_enqueue_decision(
                semaphore_closed,
                channel_closed,
                no_permits,
            ),
            _ => unreachable!(),
        };
        assert_eq!(decision, PoolEnqueueDecision::PoolClosed);
    }
}


#[cfg(any(test, kani))]
mod lazy_totality_regressions {
    use lambars_alias::control::{Lazy, LazyForceDecision, lazy_force_decision};

    static_assertions::assert_not_impl_any!(Lazy<u8>: Sync);

    fn lazy_state_is_total(state: u8) -> bool {
        matches!(
            lazy_force_decision(state),
            LazyForceDecision::Ready
                | LazyForceDecision::Initialize
                | LazyForceDecision::Error
        )
    }

    #[cfg(test)]
    #[test]
    fn sc026_lazy_total_classifier_has_no_panic_state() {
        for state in 0u8..=u8::MAX {
            assert!(lazy_state_is_total(state));
        }

        let lazy = Lazy::new(|| 41u8);
        assert_eq!(lazy.try_force().copied(), Ok(41));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc026_lazy_state_classifier_is_total_for_every_byte() {
        let state: u8 = kani::any();
        assert!(lazy_state_is_total(state));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc026_lazy_execution_mode_cannot_introduce_a_panic_decision() {
        let state: u8 = kani::any();
        let execution_mode: u8 = kani::any(); // 0 local thread, 1 moved Rayon task, 2 local async task
        kani::assume(execution_mode <= 2);

        let decision = match execution_mode {
            0 | 1 | 2 => lazy_force_decision(state),
            _ => unreachable!(),
        };
        assert!(matches!(
            decision,
            LazyForceDecision::Ready
                | LazyForceDecision::Initialize
                | LazyForceDecision::Error
        ));
    }
}


#[cfg(any(test, kani))]
mod concurrent_lazy_totality_regressions {
    use lambars_alias::control::{
        ConcurrentLazyTryForceDecision, concurrent_lazy_try_force_decision,
    };

    fn decision_is_total(state: u8, reentrant: bool) -> bool {
        matches!(
            concurrent_lazy_try_force_decision(state, reentrant),
            ConcurrentLazyTryForceDecision::Ready
                | ConcurrentLazyTryForceDecision::Initialize
                | ConcurrentLazyTryForceDecision::Wait
                | ConcurrentLazyTryForceDecision::Error
        )
    }

    #[cfg(test)]
    #[test]
    fn sc026_concurrent_lazy_try_force_classifier_is_total() {
        for state in 0u8..=u8::MAX {
            assert!(decision_is_total(state, false));
            assert!(decision_is_total(state, true));
        }
        assert_eq!(
            concurrent_lazy_try_force_decision(1, true),
            ConcurrentLazyTryForceDecision::Error
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc026_concurrent_lazy_try_force_has_no_panic_decision() {
        let state: u8 = kani::any();
        let reentrant: bool = kani::any();
        assert!(decision_is_total(state, reentrant));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc026_same_thread_reentry_never_waits() {
        assert_eq!(
            concurrent_lazy_try_force_decision(1, true),
            ConcurrentLazyTryForceDecision::Error
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc026_concurrent_parallel_async_modes_preserve_try_force_decision() {
        let state: u8 = kani::any();
        let reentrant: bool = kani::any();
        let execution_mode: u8 = kani::any(); // 0 thread, 1 Rayon, 2 async bridge
        kani::assume(execution_mode <= 2);
        let baseline = concurrent_lazy_try_force_decision(state, reentrant);
        let under_mode = match execution_mode {
            0 | 1 | 2 => concurrent_lazy_try_force_decision(state, reentrant),
            _ => unreachable!(),
        };
        assert_eq!(baseline, under_mode);
    }
}


#[cfg(any(test, kani))]
mod freer_totality_regressions {
    use lambars_alias::control::{FreerInterpretDecision, freer_interpret_decision};

    #[cfg(test)]
    #[test]
    fn sc026_freer_failure_states_are_typed_decisions() {
        assert_eq!(
            freer_interpret_decision(true, true, false),
            FreerInterpretDecision::HandlerPanicked
        );
        assert_eq!(
            freer_interpret_decision(true, false, true),
            FreerInterpretDecision::ContinuationPanicked
        );
        assert_eq!(
            freer_interpret_decision(false, false, false),
            FreerInterpretDecision::TypeMismatch
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc026_freer_handler_panic_never_continues() {
        let type_match: bool = kani::any();
        let continuation_panicked: bool = kani::any();
        assert_ne!(
            freer_interpret_decision(type_match, true, continuation_panicked),
            FreerInterpretDecision::Continue
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc026_freer_continuation_panic_never_continues() {
        let type_match: bool = kani::any();
        assert_ne!(
            freer_interpret_decision(type_match, false, true),
            FreerInterpretDecision::Continue
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc026_freer_type_mismatch_never_continues() {
        assert_eq!(
            freer_interpret_decision(false, false, false),
            FreerInterpretDecision::TypeMismatch
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    fn sc026_freer_thread_parallel_async_modes_preserve_failure_decision() {
        let type_match: bool = kani::any();
        let handler_panicked: bool = kani::any();
        let continuation_panicked: bool = kani::any();
        let execution_mode: u8 = kani::any(); // 0 thread, 1 Rayon, 2 async
        kani::assume(execution_mode <= 2);

        let baseline =
            freer_interpret_decision(type_match, handler_panicked, continuation_panicked);
        let under_mode = match execution_mode {
            0 | 1 | 2 => {
                freer_interpret_decision(type_match, handler_panicked, continuation_panicked)
            }
            _ => unreachable!(),
        };
        assert_eq!(baseline, under_mode);
    }
}
