#![deny(unsafe_code)]

pub use lambars_verify_macros::{dual_verify, verification_case, VerificationModel};

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
