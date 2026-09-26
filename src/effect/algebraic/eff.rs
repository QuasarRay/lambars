//! Effectful computation type.
//!
//! This module provides the `Eff<E, A>` type that represents a computation
//! that may use effect `E` and produces a value of type `A`.
//!
//! # Stack Safety
//!
//! The implementation uses a `FlatMap` variant internally
//! to ensure deep `flat_map` chains do not overflow the stack.

use super::effect::Effect;
use std::any::Any;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// A tag identifying different operations within an effect.
///
/// Each operation of an effect has a unique tag used by handlers
/// to determine which operation is being requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperationTag(pub(crate) u32);

impl OperationTag {
    /// Creates a new operation tag with the given value.
    #[must_use]
    #[inline]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

/// Typed failure produced by the algebraic-effect interpreter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgebraicError {
    /// A type-erased operation argument/result did not match its declared type.
    TypeMismatch {
        /// Static description of the failed boundary.
        context: &'static str,
    },
    /// A handler received an operation tag that it does not implement.
    UnknownOperation {
        /// Name of the effect being interpreted.
        effect: &'static str,
        /// Unexpected operation tag.
        operation_tag: OperationTag,
    },
    /// A normalized computation still contained a deferred FlatMap node.
    NormalizationInvariant,
    /// A user continuation or mapping closure unwound.
    ContinuationPanicked,
}

impl fmt::Display for AlgebraicError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeMismatch { context } => {
                write!(formatter, "algebraic effect type mismatch at {context}")
            }
            Self::UnknownOperation { effect, operation_tag } => {
                write!(formatter, "unknown {effect} operation: {operation_tag:?}")
            }
            Self::NormalizationInvariant => {
                write!(formatter, "algebraic effect normalization invariant violated")
            }
            Self::ContinuationPanicked => {
                write!(formatter, "algebraic effect continuation unwound")
            }
        }
    }
}

impl Error for AlgebraicError {}

/// Pure decision used by production checks and Kani/Verus regressions.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlgebraicExecutionDecision {
    /// Execution can proceed.
    Proceed = 0,
    /// The computation has completed with a pure value.
    Complete = 1,
    /// A type-erased boundary failed.
    TypeMismatch = 2,
    /// A handler received an unsupported operation.
    UnknownOperation = 3,
    /// A normalized-shape invariant failed.
    InvalidShape = 4,
    /// A continuation unwound.
    ContinuationPanicked = 5,
    /// A previously captured failure must propagate.
    PropagateFailure = 6,
}

/// Classifies one algebraic interpreter step without panicking.
/// node_kind uses 0=Pure, 1=Impure, 2=FlatMap, 3=Failed.
#[doc(hidden)]
#[must_use]
pub const fn algebraic_execution_decision(
    node_kind: u8,
    operation_known: bool,
    type_matches: bool,
    continuation_panicked: bool,
) -> AlgebraicExecutionDecision {
    if continuation_panicked {
        return AlgebraicExecutionDecision::ContinuationPanicked;
    }

    match node_kind {
        0 => AlgebraicExecutionDecision::Complete,
        1 if !operation_known => AlgebraicExecutionDecision::UnknownOperation,
        1 if !type_matches => AlgebraicExecutionDecision::TypeMismatch,
        1 => AlgebraicExecutionDecision::Proceed,
        2 => AlgebraicExecutionDecision::InvalidShape,
        3 => AlgebraicExecutionDecision::PropagateFailure,
        _ => AlgebraicExecutionDecision::InvalidShape,
    }
}

/// Type alias for type-erased continuation functions.
type Continuation<E, A> = Box<dyn FnOnce(Box<dyn Any>) -> Eff<E, A> + 'static>;

/// Internal structure representing an effect operation.
pub struct EffOperation<E: Effect, A: 'static> {
    pub effect_marker: PhantomData<E>,
    pub operation_tag: OperationTag,
    pub arguments: Box<dyn Any + Send + Sync>,
    pub continuation: Continuation<E, A>,
}

/// Internal structure for deferred `flat_map` (stack safety).
pub struct EffFlatMap<E: Effect, A: 'static> {
    pub source: Box<dyn Any + 'static>,
    pub transform: Continuation<E, A>,
}

/// Internal representation of an effectful computation.
pub enum EffInner<E: Effect, A: 'static> {
    Pure(A),
    Impure(EffOperation<E, A>),
    FlatMap(Box<EffFlatMap<E, A>>),
    Failed(AlgebraicError),
}

/// An effectful computation.
///
/// `Eff<E, A>` represents a computation that may use effect `E` and
/// produces a value of type `A`. Effects are not executed until
/// a handler is applied.
///
/// # Type Parameters
///
/// - `E`: The effect type this computation uses
/// - `A`: The result type of the computation
///
/// # Monad Laws
///
/// `Eff` satisfies the monad laws:
///
/// 1. **Left Identity**: `Eff::pure(a).flat_map(f) == f(a)`
/// 2. **Right Identity**: `m.flat_map(Eff::pure) == m`
/// 3. **Associativity**: `m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))`
///
/// # Stack Safety
///
/// Deep `flat_map` chains are handled using a deferred evaluation strategy
/// combined with iterative execution, preventing stack overflow.
///
/// # Examples
///
/// ```rust
/// use lambars::effect::algebraic::{Eff, NoEffect, PureHandler, Handler};
///
/// let computation = Eff::<NoEffect, i32>::pure(21)
///     .fmap(|x| x * 2);
///
/// let result = PureHandler.run(computation).unwrap();
/// assert_eq!(result, 42);
/// ```
pub struct Eff<E: Effect, A: 'static> {
    pub(super) inner: EffInner<E, A>,
}

impl<E: Effect, A: 'static> Eff<E, A> {
    /// Creates a pure computation that immediately returns the given value.
    ///
    /// This is the `return` / `pure` operation for the Eff monad.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::effect::algebraic::{Eff, NoEffect, PureHandler, Handler};
    ///
    /// let computation = Eff::<NoEffect, i32>::pure(42);
    /// assert!(computation.is_pure());
    /// ```
    #[must_use]
    #[inline]
    pub const fn pure(value: A) -> Self {
        Self {
            inner: EffInner::Pure(value),
        }
    }

    /// Creates a failed computation for internal typed-error propagation.
    #[inline]
    pub(crate) const fn failed(error: AlgebraicError) -> Self {
        Self {
            inner: EffInner::Failed(error),
        }
    }

    /// Invokes a type-erased continuation while converting unwind to a typed failure.
    #[inline]
    pub(crate) fn invoke_continuation(
        continuation: Continuation<E, A>,
        input: Box<dyn Any>,
    ) -> Self {
        match catch_unwind(AssertUnwindSafe(|| continuation(input))) {
            Ok(next) => next,
            Err(_) => Self::failed(AlgebraicError::ContinuationPanicked),
        }
    }

    /// Checks if this computation is a pure value (no pending effects).
    ///
    /// Note: This only checks the immediate structure. A computation
    /// may appear pure but have effects in nested `flat_map` chains.
    #[must_use]
    #[inline]
    pub const fn is_pure(&self) -> bool {
        matches!(&self.inner, EffInner::Pure(_))
    }

    /// Creates an effect operation.
    ///
    /// This function is used by effect definitions to create operations.
    /// Typically accessed through `define_effect!` macro.
    ///
    /// # Note
    ///
    /// This is a low-level API. Prefer using the `define_effect!` macro
    /// to create effects with proper operation tags.
    ///
    /// # Panics
    ///
    /// The returned computation will panic during handler execution if
    /// the handler provides a result of an incorrect type. This indicates
    /// a bug in the handler implementation.
    pub fn perform_raw<R: 'static>(
        operation_tag: OperationTag,
        arguments: impl Any + Send + Sync + 'static,
    ) -> Eff<E, R> {
        Eff {
            inner: EffInner::Impure(EffOperation {
                effect_marker: PhantomData,
                operation_tag,
                arguments: Box::new(arguments),
                continuation: Box::new(|result| match result.downcast::<R>() {
                    Ok(value) => Eff::pure(*value),
                    Err(_) => Eff::failed(AlgebraicError::TypeMismatch {
                        context: "Eff::perform_raw result",
                    }),
                }),
            }),
        }
    }

    /// Normalizes the computation by evaluating `FlatMap` chains.
    ///
    /// This method converts `FlatMap` variants to `Pure` or `Impure`,
    /// using an iterative approach for stack safety.
    #[inline]
    pub(crate) fn normalize(self) -> Self {
        match self.inner {
            EffInner::Pure(_) | EffInner::Impure(_) | EffInner::Failed(_) => self,
            EffInner::FlatMap(flat_map) => Self::normalize_iteratively(*flat_map),
        }
    }

    #[inline]
    fn normalize_iteratively(initial_flat_map: EffFlatMap<E, A>) -> Self {
        let mut current_result = Self::invoke_continuation(
            initial_flat_map.transform,
            initial_flat_map.source,
        );

        loop {
            match current_result.inner {
                EffInner::Pure(_) | EffInner::Impure(_) | EffInner::Failed(_) => {
                    return current_result;
                }
                EffInner::FlatMap(next_flat_map) => {
                    current_result = Self::invoke_continuation(
                        next_flat_map.transform,
                        next_flat_map.source,
                    );
                }
            }
        }
    }

    /// Applies a function to the result of this computation.
    ///
    /// This is the `fmap` / `map` operation (Functor).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::effect::algebraic::{Eff, NoEffect, PureHandler, Handler};
    ///
    /// let computation = Eff::<NoEffect, i32>::pure(21)
    ///     .fmap(|x| x * 2);
    ///
    /// let result = PureHandler.run(computation).unwrap();
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn fmap<B: 'static, F>(self, function: F) -> Eff<E, B>
    where
        F: FnOnce(A) -> B + 'static,
    {
        self.flat_map(|value| Eff::pure(function(value)))
    }

    /// Chains this computation with another that depends on its result.
    ///
    /// This is the `bind` / `>>=` operation (Monad).
    ///
    /// Uses deferred evaluation for stack safety.
    ///
    /// Type-erasure mismatches and continuation unwinds are captured in the
    /// computation and returned by the handler as AlgebraicError.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::effect::algebraic::{Eff, NoEffect, PureHandler, Handler};
    ///
    /// let computation = Eff::<NoEffect, i32>::pure(10)
    ///     .flat_map(|x| Eff::pure(x + 5));
    ///
    /// let result = PureHandler.run(computation).unwrap();
    /// assert_eq!(result, 15);
    /// ```
    #[inline]
    pub fn flat_map<B: 'static, F>(self, function: F) -> Eff<E, B>
    where
        F: FnOnce(A) -> Eff<E, B> + 'static,
    {
        match self.inner {
            EffInner::Pure(value) => {
                match catch_unwind(AssertUnwindSafe(|| function(value))) {
                    Ok(next) => next,
                    Err(_) => Eff::failed(AlgebraicError::ContinuationPanicked),
                }
            }
            EffInner::Impure(operation) => Eff {
                inner: EffInner::Impure(EffOperation {
                    effect_marker: operation.effect_marker,
                    operation_tag: operation.operation_tag,
                    arguments: operation.arguments,
                    continuation: Box::new(move |result| {
                        let next = Eff::<E, A>::invoke_continuation(
                            operation.continuation,
                            result,
                        );
                        Eff {
                            inner: EffInner::FlatMap(Box::new(EffFlatMap {
                                source: Box::new(next),
                                transform: Box::new(move |source| {
                                    match source.downcast::<Self>() {
                                        Ok(eff) => eff.flat_map(function),
                                        Err(_) => Eff::failed(AlgebraicError::TypeMismatch {
                                            context: "Eff::flat_map source",
                                        }),
                                    }
                                }),
                            })),
                        }
                    }),
                }),
            },
            EffInner::FlatMap(flat_map) => {
                let EffFlatMap { source, transform } = *flat_map;
                Eff {
                    inner: EffInner::FlatMap(Box::new(EffFlatMap {
                        source,
                        transform: Box::new(move |src| {
                            let next = Eff::<E, A>::invoke_continuation(transform, src);
                            next.flat_map(function)
                        }),
                    })),
                }
            }
            EffInner::Failed(error) => Eff::failed(error),
        }
    }

    /// Alias for `flat_map`.
    #[inline]
    pub fn and_then<B: 'static, F>(self, function: F) -> Eff<E, B>
    where
        F: FnOnce(A) -> Eff<E, B> + 'static,
    {
        self.flat_map(function)
    }

    /// Sequences two computations, discarding the first result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::effect::algebraic::{Eff, NoEffect, PureHandler, Handler};
    ///
    /// let computation = Eff::<NoEffect, i32>::pure(10)
    ///     .then(Eff::pure(42));
    ///
    /// let result = PureHandler.run(computation).unwrap();
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn then<B: 'static>(self, next: Eff<E, B>) -> Eff<E, B> {
        self.flat_map(|_| next)
    }

    /// Combines two computations using a binary function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::effect::algebraic::{Eff, NoEffect, PureHandler, Handler};
    ///
    /// let computation = Eff::<NoEffect, i32>::pure(10)
    ///     .map2(Eff::pure(20), |a, b| a + b);
    ///
    /// let result = PureHandler.run(computation).unwrap();
    /// assert_eq!(result, 30);
    /// ```
    pub fn map2<B: 'static, C: 'static, F>(self, other: Eff<E, B>, function: F) -> Eff<E, C>
    where
        F: FnOnce(A, B) -> C + 'static,
    {
        self.flat_map(|value_a| other.fmap(|value_b| function(value_a, value_b)))
    }

    /// Combines two computations into a tuple.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::effect::algebraic::{Eff, NoEffect, PureHandler, Handler};
    ///
    /// let computation = Eff::<NoEffect, i32>::pure(1)
    ///     .product(Eff::pure(2));
    ///
    /// let result = PureHandler.run(computation).unwrap();
    /// assert_eq!(result, (1, 2));
    /// ```
    #[inline]
    pub fn product<B: 'static>(self, other: Eff<E, B>) -> Eff<E, (A, B)> {
        self.map2(other, |value_a, value_b| (value_a, value_b))
    }
}

// =============================================================================
// Note on TypeClass implementations
// =============================================================================
//
// Eff requires 'static bounds for its type parameters due to the use of
// type-erased continuations (Box<dyn FnOnce(...)>). This makes it incompatible
// with the current typeclass trait definitions which don't have 'static bounds.
//
// Instead of implementing TypeConstructor, Functor, Applicative, and Monad traits,
// Eff provides equivalent functionality through its own methods:
//
// - `pure`: Equivalent to Applicative::pure
// - `fmap`: Equivalent to Functor::fmap
// - `flat_map` / `and_then`: Equivalent to Monad::flat_map
// - `map2`, `product`: Equivalent to Applicative operations
//
// Future work: Consider creating separate traits with 'static bounds for
// effect-based computations, or using GATs more extensively.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::algebraic::NoEffect;
    use rstest::rstest;

    #[rstest]
    fn eff_pure_creates_pure_value() {
        let eff: Eff<NoEffect, i32> = Eff::pure(42);
        assert!(eff.is_pure());
    }

    #[rstest]
    fn eff_pure_with_string() {
        let eff: Eff<NoEffect, String> = Eff::pure("hello".to_string());
        assert!(eff.is_pure());
    }

    #[rstest]
    fn eff_pure_with_complex_type() {
        let eff: Eff<NoEffect, Vec<i32>> = Eff::pure(vec![1, 2, 3]);
        assert!(eff.is_pure());
    }

    #[rstest]
    fn eff_fmap_on_pure_produces_pure() {
        let eff: Eff<NoEffect, i32> = Eff::pure(21);
        let mapped = eff.fmap(|x| x * 2);
        assert!(mapped.is_pure());
    }

    #[rstest]
    fn eff_flat_map_on_pure_produces_result_directly() {
        let eff: Eff<NoEffect, i32> = Eff::pure(10);
        let result = eff.flat_map(|x| Eff::pure(x + 5));
        assert!(result.is_pure());
    }

    #[rstest]
    fn eff_and_then_is_alias_for_flat_map() {
        let eff: Eff<NoEffect, i32> = Eff::pure(10);
        let result = eff.and_then(|x| Eff::pure(x + 5));
        assert!(result.is_pure());
    }

    #[rstest]
    fn eff_then_discards_first_result() {
        let first: Eff<NoEffect, i32> = Eff::pure(10);
        let second: Eff<NoEffect, &str> = Eff::pure("result");
        let result = first.then(second);
        assert!(result.is_pure());
    }

    #[rstest]
    fn eff_map2_combines_two_pure_values() {
        let first: Eff<NoEffect, i32> = Eff::pure(10);
        let second: Eff<NoEffect, i32> = Eff::pure(20);
        let result = first.map2(second, |a, b| a + b);
        assert!(result.is_pure());
    }

    #[rstest]
    fn eff_product_creates_tuple() {
        let first: Eff<NoEffect, i32> = Eff::pure(1);
        let second: Eff<NoEffect, &str> = Eff::pure("hello");
        let result = first.product(second);
        assert!(result.is_pure());
    }

    #[rstest]
    fn operation_tag_new_creates_tag() {
        let tag = OperationTag::new(42);
        assert_eq!(tag.0, 42);
    }

    #[rstest]
    fn operation_tag_equality() {
        let tag1 = OperationTag::new(1);
        let tag2 = OperationTag::new(1);
        let tag3 = OperationTag::new(2);
        assert_eq!(tag1, tag2);
        assert_ne!(tag1, tag3);
    }

    #[rstest]
    fn operation_tag_is_debug() {
        let tag = OperationTag::new(42);
        let debug_string = format!("{tag:?}");
        assert!(debug_string.contains("42"));
    }

    #[rstest]
    fn operation_tag_is_clone() {
        let tag = OperationTag::new(42);
        let cloned = tag;
        assert_eq!(tag, cloned);
    }

    #[rstest]
    fn operation_tag_is_copy() {
        let tag = OperationTag::new(42);
        let copied = tag;
        assert_eq!(tag, copied);
    }
}
