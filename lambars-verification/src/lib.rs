#![deny(unsafe_code)]

pub use lambars_verify_macros::{
    VerificationModel, boundary_cases, dual_verify, verification_case,
};

/// Marker implemented by data models that participate in generated verification.
pub trait VerificationModel {
    const TYPE_NAME: &'static str;
}

/// Reuse one semantic predicate under multiple canonical specification names.
///
/// This is intentionally a declarative delegation layer: repeated proof bodies
/// should delegate to a single predicate rather than copy verification logic.
#[macro_export]
macro_rules! delegate_verification {
    ($predicate:path => $($name:ident : $id:literal),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                assert!($predicate(), "delegated specification {} failed", $id);
            }
        )+
    };
}

/// Declare multiple zero-argument predicates that share one implementation.
///
/// Kani companions are generated for the same predicate so regressions in the
/// generic implementation are explored once per canonical contract.
#[macro_export]
macro_rules! delegate_kani_verification {
    ($predicate:path => $($name:ident),+ $(,)?) => {
        $(
            #[cfg(kani)]
            #[kani::proof]
            fn $name() {
                assert!($predicate());
            }
        )+
    };
}


/// Number of findings in the safety-critical qualification register.
pub const SAFETY_FINDING_COUNT: usize = 40;

/// Bit mask containing every release-blocking SC-001..SC-040 finding.
pub const SAFETY_REQUIRED_MASK: u64 = (1u64 << SAFETY_FINDING_COUNT) - 1;

/// Returns the bit corresponding to a zero-based safety finding index.
#[must_use]
pub const fn safety_finding_bit(index: usize) -> u64 {
    1u64 << index
}

/// Fail-closed release qualification predicate.
///
/// Bit N is set only when SC-(N+1) is closed by the repository qualification
/// checker. Unknown high bits are ignored; every one of the 40 required bits
/// must be present.
#[must_use]
pub const fn safety_release_qualified(closed_mask: u64) -> bool {
    closed_mask & SAFETY_REQUIRED_MASK == SAFETY_REQUIRED_MASK
}
