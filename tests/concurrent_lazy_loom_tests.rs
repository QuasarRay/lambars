//! Loom models for the ConcurrentLazy synchronization protocol.
//!
//! These tests exhaustively explore bounded interleavings of the same state-transition
//! and publication rules used by `ConcurrentLazy`: EMPTY -> COMPUTING -> READY/POISONED,
//! compare-exchange ownership, and Release/Acquire publication of the initialized value.
//!
//! The production type still uses `std` atomics + `parking_lot`; these models validate
//! the synchronization protocol rather than substituting randomized stress tests for Loom.

#![cfg(feature = "control")]

use loom::sync::Arc;
use loom::sync::atomic::{AtomicUsize, Ordering};
use loom::thread;

const EMPTY: usize = 0;
const COMPUTING: usize = 1;
const READY: usize = 2;
const POISONED: usize = 3;

#[test]
fn loom_exactly_one_initializer_and_release_acquire_publication() {
    loom::model(|| {
        let state = Arc::new(AtomicUsize::new(EMPTY));
        let value = Arc::new(AtomicUsize::new(0));
        let init_count = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..2 {
            let state = Arc::clone(&state);
            let value = Arc::clone(&value);
            let init_count = Arc::clone(&init_count);
            handles.push(thread::spawn(move || {
                loop {
                    match state.load(Ordering::Acquire) {
                        READY => {
                            assert_eq!(value.load(Ordering::Relaxed), 42);
                            break;
                        }
                        EMPTY => {
                            if state
                                .compare_exchange(
                                    EMPTY,
                                    COMPUTING,
                                    Ordering::AcqRel,
                                    Ordering::Acquire,
                                )
                                .is_ok()
                            {
                                init_count.fetch_add(1, Ordering::Relaxed);
                                value.store(42, Ordering::Relaxed);
                                state.store(READY, Ordering::Release);
                                break;
                            }
                        }
                        COMPUTING => thread::yield_now(),
                        POISONED => panic!("successful publication model unexpectedly poisoned"),
                        _ => unreachable!(),
                    }
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(state.load(Ordering::Acquire), READY);
        assert_eq!(value.load(Ordering::Relaxed), 42);
        assert_eq!(init_count.load(Ordering::Relaxed), 1);
    });
}

#[test]
fn loom_poison_release_is_visible_to_waiters() {
    loom::model(|| {
        let state = Arc::new(AtomicUsize::new(EMPTY));

        let initializer_state = Arc::clone(&state);
        let initializer = thread::spawn(move || {
            if initializer_state
                .compare_exchange(
                    EMPTY,
                    COMPUTING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                initializer_state.store(POISONED, Ordering::Release);
            }
        });

        let waiter_state = Arc::clone(&state);
        let waiter = thread::spawn(move || loop {
            match waiter_state.load(Ordering::Acquire) {
                EMPTY | COMPUTING => thread::yield_now(),
                POISONED => break true,
                READY => break false,
                _ => unreachable!(),
            }
        });

        initializer.join().unwrap();
        assert!(waiter.join().unwrap());
        assert_eq!(state.load(Ordering::Acquire), POISONED);
    });
}

#[test]
fn loom_ready_state_is_never_observed_before_value_publication() {
    loom::model(|| {
        let state = Arc::new(AtomicUsize::new(EMPTY));
        let value = Arc::new(AtomicUsize::new(0));

        let writer_state = Arc::clone(&state);
        let writer_value = Arc::clone(&value);
        let writer = thread::spawn(move || {
            if writer_state
                .compare_exchange(
                    EMPTY,
                    COMPUTING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                writer_value.store(7, Ordering::Relaxed);
                writer_state.store(READY, Ordering::Release);
            }
        });

        let reader_state = Arc::clone(&state);
        let reader_value = Arc::clone(&value);
        let reader = thread::spawn(move || loop {
            if reader_state.load(Ordering::Acquire) == READY {
                assert_eq!(reader_value.load(Ordering::Relaxed), 7);
                break;
            }
            thread::yield_now();
        });

        writer.join().unwrap();
        reader.join().unwrap();
    });
}
