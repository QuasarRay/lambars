//! Lazy evaluation with memoization.
//!
//! This module provides the `Lazy<T, F>` type for lazy evaluation.
//! Values are computed only when needed and cached for subsequent accesses.
//!
//! # Storage model
//!
//! The memoized value is stored in `std::cell::OnceCell` and the one-shot
//! initializer in `RefCell<Option<F>>`. No unsafe code or manual initialization/
//! destruction is required. The state byte is retained for poisoning and re-entry
//! semantics and for compatibility with the formal state model.
//!
//! # Referential Transparency Note
//!
//! While `Lazy` provides memoization that appears functionally pure on success,
//! it is **not referentially transparent** in the presence of panics:
//!
//! - If the initialization function panics, the `Lazy` becomes **poisoned**
//! - Once poisoned, all subsequent calls to `force()` will panic
//! - This means the behavior of `force()` depends on the history of previous calls
//!
//! This is a deliberate design decision to prevent returning potentially inconsistent
//! partial state after a panic. Users who need strict referential transparency
//! should ensure their initialization functions do not panic, or handle the
//! poisoned state explicitly via `is_poisoned()` before calling `force()`.
//!
//! # Examples
//!
//! ```rust
//! use lambars::control::Lazy;
//!
//! let lazy = Lazy::new(|| {
//!     println!("Computing...");
//!     42
//! });
//!
//! // No output yet - computation is deferred
//! println!("Created lazy value");
//!
//! // Now "Computing..." is printed
//! let value = lazy.force();
//! assert_eq!(*value, 42);
//!
//! // No recomputation - result is memoized
//! let value2 = lazy.force();
//! assert_eq!(*value2, 42);
//! ```

use std::cell::{OnceCell, RefCell};
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU8, Ordering};

/// State: not yet initialized
const STATE_EMPTY: u8 = 0;
/// State: initialization in progress
const STATE_COMPUTING: u8 = 1;
/// State: initialization complete
const STATE_READY: u8 = 2;
/// State: initialization panicked
const STATE_POISONED: u8 = 3;

/// Error returned when attempting to access a poisoned `Lazy` value.
///
/// A `Lazy` value becomes poisoned when its initialization function panics.
/// After poisoning, the value cannot be accessed and any attempt to do so
/// will return this error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LazyPoisonedError;

impl fmt::Display for LazyPoisonedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Lazy value is poisoned")
    }
}

impl std::error::Error for LazyPoisonedError {}

/// Pure state decision shared by the total Lazy access path and formal regressions.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LazyForceDecision {
    Ready = 0,
    Initialize = 1,
    Error = 2,
}

/// Classifies Lazy state without panicking.
#[doc(hidden)]
#[must_use]
pub const fn lazy_force_decision(state: u8) -> LazyForceDecision {
    match state {
        STATE_READY => LazyForceDecision::Ready,
        STATE_EMPTY => LazyForceDecision::Initialize,
        STATE_COMPUTING | STATE_POISONED => LazyForceDecision::Error,
        _ => LazyForceDecision::Error,
    }
}

/// A lazily evaluated value with memoization.
///
/// `Lazy<T, F>` defers computation until the value is first accessed via `force()`.
/// Once computed, the value is cached and subsequent calls to `force()` return
/// the cached value without recomputation.
///
/// # Type Parameters
///
/// * `T` - The type of the computed value
/// * `F` - The type of the initialization function (defaults to `fn() -> T`)
///
/// # Thread Safety
///
/// This type is NOT thread-safe (`!Sync`). For concurrent access, use
/// [`ConcurrentLazy`](super::ConcurrentLazy).
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use lambars::control::Lazy;
///
/// let lazy = Lazy::new(|| expensive_computation());
///
/// // Computation happens here
/// let value = lazy.force();
///
/// fn expensive_computation() -> i32 {
///     // Simulating expensive work
///     42
/// }
/// ```
///
/// ## Memoization
///
/// ```rust
/// use lambars::control::Lazy;
/// use std::cell::Cell;
///
/// let call_count = Cell::new(0);
/// let lazy = Lazy::new(|| {
///     call_count.set(call_count.get() + 1);
///     42
/// });
///
/// assert_eq!(call_count.get(), 0); // Not called yet
///
/// let _ = lazy.force();
/// assert_eq!(call_count.get(), 1); // Called once
///
/// let _ = lazy.force();
/// assert_eq!(call_count.get(), 1); // Still only once - memoized
/// ```
pub struct Lazy<T, F = fn() -> T> {
    state: AtomicU8,
    value: OnceCell<T>,
    initializer: RefCell<Option<F>>,
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    /// Creates a new lazy value with the given initialization function.
    ///
    /// The function will not be called until `force()` is invoked.
    ///
    /// # Arguments
    ///
    /// * `initializer` - A function that produces the value when called
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy = Lazy::new(|| {
    ///     println!("Initializing...");
    ///     42
    /// });
    /// // Nothing printed yet
    /// ```
    #[inline]
    pub const fn new(initializer: F) -> Self {
        Self {
            state: AtomicU8::new(STATE_EMPTY),
            value: OnceCell::new(),
            initializer: RefCell::new(Some(initializer)),
        }
    }

    /// Forces evaluation of the lazy value and returns a reference to it.
    ///
    /// If the value has not been computed yet, the initialization function
    /// is called and the result is cached. Subsequent calls return the
    /// cached value.
    ///
    /// # Returns
    ///
    /// A reference to the computed value.
    ///
    /// # Panics
    ///
    /// - If the initialization function panics, the lazy value becomes
    ///   poisoned and all future calls to `force()` will panic.
    /// - If the value is already poisoned from a previous panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy = Lazy::new(|| 42);
    /// let value = lazy.force();
    /// assert_eq!(*value, 42);
    /// ```
    pub fn force(&self) -> &T {
        let state = self.state.load(Ordering::Acquire);

        match state {
            STATE_READY => self
                .value
                .get()
                .expect("Lazy invariant violated: READY without a value"),
            STATE_POISONED => {
                panic!("Lazy instance has been poisoned")
            }
            STATE_EMPTY => self.initialize(),
            STATE_COMPUTING => {
                panic!("Lazy::force called recursively during initialization")
            }
            _ => unreachable!("Invalid state"),
        }
    }

    /// Forces evaluation and returns a mutable reference to the value.
    ///
    /// If the value has not been computed yet, the initialization function
    /// is called and the result is cached. Subsequent calls return the
    /// cached value.
    ///
    /// # Returns
    ///
    /// A mutable reference to the computed value.
    ///
    /// # Panics
    ///
    /// - If the initialization function panics, the lazy value becomes
    ///   poisoned and all future calls will panic.
    /// - If the value is already poisoned from a previous panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let mut lazy = Lazy::new(|| vec![1, 2, 3]);
    /// lazy.force_mut().push(4);
    /// assert_eq!(lazy.force().as_slice(), &[1, 2, 3, 4]);
    /// ```
    pub fn force_mut(&mut self) -> &mut T {
        let state = *self.state.get_mut();

        match state {
            STATE_READY => self
                .value
                .get_mut()
                .expect("Lazy invariant violated: READY without a value"),
            STATE_POISONED => {
                panic!("Lazy instance has been poisoned")
            }
            STATE_EMPTY => {
                // We have &mut self, so we can safely initialize
                self.initialize_mut()
            }
            STATE_COMPUTING => {
                // With &mut self, STATE_COMPUTING should not be observable
                // as it would require concurrent access which is prevented by borrowing rules
                panic!("Lazy::force_mut called recursively during initialization")
            }
            _ => unreachable!("Invalid state"),
        }
    }

    /// Performs initialization for the explicitly partial `force()` wrapper.
    fn initialize(&self) -> &T {
        self.initialize_result()
            .unwrap_or_else(|_| panic!("Lazy: initialization failed or re-entered"))
    }

    /// Performs initialization without unwinding.
    fn initialize_result(&self) -> Result<&T, LazyPoisonedError> {
        match self.state.compare_exchange(
            STATE_EMPTY,
            STATE_COMPUTING,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => {
                let Some(initializer) = self.initializer.borrow_mut().take() else {
                    self.state.store(STATE_POISONED, Ordering::Release);
                    return Err(LazyPoisonedError);
                };

                match catch_unwind(AssertUnwindSafe(initializer)) {
                    Ok(value) => {
                        if self.value.set(value).is_err() {
                            self.state.store(STATE_POISONED, Ordering::Release);
                            return Err(LazyPoisonedError);
                        }
                        self.state.store(STATE_READY, Ordering::Release);
                        self.value.get().ok_or(LazyPoisonedError)
                    }
                    Err(_) => {
                        self.state.store(STATE_POISONED, Ordering::Release);
                        Err(LazyPoisonedError)
                    }
                }
            }
            Err(current) => match lazy_force_decision(current) {
                LazyForceDecision::Ready => self.value.get().ok_or(LazyPoisonedError),
                LazyForceDecision::Initialize | LazyForceDecision::Error => {
                    Err(LazyPoisonedError)
                }
            },
        }
    }

    /// Performs the initialization (mutable self version).
    fn initialize_mut(&mut self) -> &mut T {
        *self.state.get_mut() = STATE_COMPUTING;

        let initializer = self
            .initializer
            .get_mut()
            .take()
            .expect("initializer already consumed");

        match catch_unwind(AssertUnwindSafe(initializer)) {
            Ok(value) => {
                if self.value.set(value).is_err() {
                    *self.state.get_mut() = STATE_POISONED;
                    panic!("Lazy invariant violated: value initialized twice");
                }

                *self.state.get_mut() = STATE_READY;
                self.value
                    .get_mut()
                    .expect("Lazy invariant violated: READY without a value")
            }
            Err(_) => {
                *self.state.get_mut() = STATE_POISONED;
                panic!("Lazy: initialization function panicked");
            }
        }
    }

}

impl<T> Lazy<T, fn() -> T> {
    /// Creates a new lazy value that is already initialized.
    ///
    /// This is useful when you have a value that should be treated as lazy
    /// for API consistency, but the value is already available.
    ///
    /// # Arguments
    ///
    /// * `value` - The already-computed value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy = Lazy::new_with_value(42);
    /// assert!(lazy.is_initialized());
    /// ```
    #[inline]
    pub fn new_with_value(value: T) -> Self {
        let cell = OnceCell::new();
        if cell.set(value).is_err() {
            unreachable!("new OnceCell unexpectedly contained a value");
        }
        Self {
            state: AtomicU8::new(STATE_READY),
            value: cell,
            initializer: RefCell::new(None),
        }
    }

    /// Creates a pure lazy value (Applicative pure).
    ///
    /// This is equivalent to `new_with_value` and lifts a value into
    /// the Lazy context.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy = Lazy::pure(42);
    /// assert_eq!(*lazy.force(), 42);
    /// ```
    #[inline]
    pub fn pure(value: T) -> Self {
        Self::new_with_value(value)
    }
}

impl<T, F> Lazy<T, F> {
    /// Returns a reference to the value if it has been initialized.
    ///
    /// Unlike `force()`, this method does not trigger initialization.
    ///
    /// # Returns
    ///
    /// - `Some(&T)` if the value is initialized
    /// - `None` if the value has not been initialized or is poisoned
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy = Lazy::new(|| 42);
    ///
    /// assert!(lazy.get().is_none()); // Not initialized yet
    ///
    /// let _ = lazy.force();
    /// assert!(lazy.get().is_some()); // Now initialized
    /// ```
    pub fn get(&self) -> Option<&T> {
        if self.state.load(Ordering::Acquire) == STATE_READY {
            self.value.get()
        } else {
            None
        }
    }

    /// Returns a mutable reference to the value if it has been initialized.
    ///
    /// Unlike `force_mut()`, this method does not trigger initialization.
    ///
    /// # Returns
    ///
    /// - `Some(&mut T)` if the value is initialized
    /// - `None` if the value has not been initialized yet
    ///
    /// # Panics
    ///
    /// Panics if the Lazy is in a poisoned state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let mut lazy = Lazy::new(|| 42);
    /// assert!(lazy.get_mut().is_none());
    /// lazy.force();
    /// assert!(lazy.get_mut().is_some());
    /// assert_eq!(*lazy.get_mut().unwrap(), 42);
    /// ```
    pub fn get_mut(&mut self) -> Option<&mut T> {
        let state = *self.state.get_mut();
        match state {
            STATE_READY => self.value.get_mut(),
            STATE_POISONED => panic!("Lazy instance has been poisoned"),
            _ => None,
        }
    }

    /// Returns whether the value has been initialized.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy = Lazy::new(|| 42);
    /// assert!(!lazy.is_initialized());
    ///
    /// let _ = lazy.force();
    /// assert!(lazy.is_initialized());
    /// ```
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.state.load(Ordering::Acquire) == STATE_READY
    }

    /// Returns whether the lazy value has been poisoned.
    ///
    /// A lazy value becomes poisoned if the initialization function panics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    /// use std::panic::catch_unwind;
    ///
    /// let lazy = Lazy::new(|| panic!("initialization failed"));
    ///
    /// let _ = catch_unwind(std::panic::AssertUnwindSafe(|| lazy.force()));
    ///
    /// assert!(lazy.is_poisoned());
    /// ```
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.state.load(Ordering::Acquire) == STATE_POISONED
    }
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    /// Tries to force evaluation without panicking on poisoned state.
    ///
    /// This is a pure functional alternative to `force()` that returns a `Result`
    /// instead of panicking when the lazy value is poisoned.
    ///
    /// # Returns
    ///
    /// - `Ok(&T)` if the value is successfully computed or already cached
    /// - `Err(LazyPoisonedError)` if the lazy value is poisoned
    ///
    /// # Errors
    ///
    /// Returns `Err(LazyPoisonedError)` if:
    /// - The lazy value is poisoned (initialization previously panicked)
    /// - Re-entry is detected (calling `try_force` during initialization)
    ///
    /// Initializer panics are caught, the Lazy is poisoned, and this method
    /// returns `Err(LazyPoisonedError)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    /// use std::panic::catch_unwind;
    ///
    /// // Normal usage
    /// let lazy = Lazy::new(|| 42);
    /// assert_eq!(*lazy.try_force().unwrap(), 42);
    ///
    /// // Poisoned lazy
    /// let poisoned = Lazy::new(|| panic!("init failed"));
    /// let _ = catch_unwind(std::panic::AssertUnwindSafe(|| poisoned.force()));
    /// assert!(poisoned.try_force().is_err());
    /// ```
    pub fn try_force(&self) -> Result<&T, LazyPoisonedError> {
        let state = self.state.load(Ordering::Acquire);

        match lazy_force_decision(state) {
            LazyForceDecision::Ready => self.value.get().ok_or(LazyPoisonedError),
            LazyForceDecision::Initialize => self.initialize_result(),
            LazyForceDecision::Error => Err(LazyPoisonedError),
        }
    }

    /// Consumes the Lazy and returns the inner value.
    ///
    /// If the Lazy has been initialized, returns `Ok(value)`.
    /// If it has not been initialized, forces evaluation and returns `Ok(value)`.
    /// If it is poisoned, returns `Err(LazyPoisonedError)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy = Lazy::new(|| 42);
    /// assert_eq!(lazy.into_inner(), Ok(42));
    /// ```
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy = Lazy::new_with_value(42);
    /// assert_eq!(lazy.into_inner(), Ok(42));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err(LazyPoisonedError)` if the `Lazy` instance has been poisoned.
    ///
    /// Initializer panics are caught and returned as `Err(LazyPoisonedError)`.
    pub fn into_inner(mut self) -> Result<T, LazyPoisonedError> {
        match self.state.load(Ordering::Acquire) {
            STATE_READY => self.value.take().ok_or(LazyPoisonedError),
            STATE_POISONED | STATE_COMPUTING => Err(LazyPoisonedError),
            STATE_EMPTY => {
                let Some(initializer) = self.initializer.get_mut().take() else {
                    return Err(LazyPoisonedError);
                };

                catch_unwind(AssertUnwindSafe(initializer)).map_err(|_| LazyPoisonedError)
            }
            _ => Err(LazyPoisonedError),
        }
    }
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    /// Applies a function to the lazy value, producing a new lazy value.
    ///
    /// The resulting lazy value will compute the original value and then
    /// apply the function when forced.
    ///
    /// # Arguments
    ///
    /// * `function` - A function to apply to the computed value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy = Lazy::new(|| 21);
    /// let doubled = lazy.map(|x| x * 2);
    ///
    /// assert_eq!(*doubled.force(), 42);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when forced if the `Lazy` instance has been poisoned.
    pub fn map<U, G>(self, function: G) -> Lazy<U, impl FnOnce() -> U>
    where
        G: FnOnce(T) -> U,
    {
        Lazy::new(move || {
            let value = self.into_inner().expect("Lazy instance has been poisoned");
            function(value)
        })
    }

    /// Applies a function to the lazy value, returning a Result.
    ///
    /// This is a pure functional alternative to `map()` that returns `Result`
    /// instead of panicking when the lazy value is poisoned. The returned lazy
    /// value will compute to `Ok(f(value))` if successful, or `Err(LazyPoisonedError)`
    /// if the original lazy value is poisoned.
    ///
    /// # Arguments
    ///
    /// * `function` - A function to apply to the computed value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy = Lazy::new(|| 21);
    /// let doubled = lazy.try_map(|x| x * 2);
    ///
    /// assert_eq!(doubled.force().unwrap(), 42);
    /// ```
    pub fn try_map<U, G>(
        self,
        function: G,
    ) -> Lazy<Result<U, LazyPoisonedError>, impl FnOnce() -> Result<U, LazyPoisonedError>>
    where
        G: FnOnce(T) -> U,
    {
        Lazy::new(move || self.into_inner().map(function))
    }

    /// Applies a function that returns a Lazy, then flattens the result.
    ///
    /// This is the monadic bind operation for Lazy.
    ///
    /// # Arguments
    ///
    /// * `function` - A function that takes the computed value and returns a new Lazy
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy = Lazy::new(|| 21);
    /// let result = lazy.flat_map(|x| Lazy::new(move || x * 2));
    ///
    /// assert_eq!(*result.force(), 42);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when forced if the `Lazy` instance has been poisoned.
    pub fn flat_map<U, FunctionResult, G>(self, function: G) -> Lazy<U, impl FnOnce() -> U>
    where
        FunctionResult: FnOnce() -> U,
        G: FnOnce(T) -> Lazy<U, FunctionResult>,
    {
        Lazy::new(move || {
            let value = self.into_inner().expect("Lazy instance has been poisoned");
            function(value)
                .into_inner()
                .expect("Lazy instance has been poisoned")
        })
    }

    /// Combines two lazy values into a lazy tuple.
    ///
    /// # Arguments
    ///
    /// * `other` - Another lazy value to combine with
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy1 = Lazy::new(|| 1);
    /// let lazy2 = Lazy::new(|| "hello");
    /// let combined = lazy1.zip(lazy2);
    ///
    /// assert_eq!(*combined.force(), (1, "hello"));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when forced if either `Lazy` instance has been poisoned.
    pub fn zip<U, OtherFunction>(
        self,
        other: Lazy<U, OtherFunction>,
    ) -> Lazy<(T, U), impl FnOnce() -> (T, U)>
    where
        OtherFunction: FnOnce() -> U,
    {
        Lazy::new(move || {
            let value1 = self.into_inner().expect("Lazy instance has been poisoned");
            let value2 = other.into_inner().expect("Lazy instance has been poisoned");
            (value1, value2)
        })
    }

    /// Combines two lazy values using a function.
    ///
    /// # Arguments
    ///
    /// * `other` - Another lazy value to combine with
    /// * `function` - A function that combines the two values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy1 = Lazy::new(|| 20);
    /// let lazy2 = Lazy::new(|| 22);
    /// let sum = lazy1.zip_with(lazy2, |a, b| a + b);
    ///
    /// assert_eq!(*sum.force(), 42);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when forced if either `Lazy` instance has been poisoned.
    pub fn zip_with<U, V, OtherFunction, CombineFunction>(
        self,
        other: Lazy<U, OtherFunction>,
        function: CombineFunction,
    ) -> Lazy<V, impl FnOnce() -> V>
    where
        OtherFunction: FnOnce() -> U,
        CombineFunction: FnOnce(T, U) -> V,
    {
        Lazy::new(move || {
            let value1 = self.into_inner().expect("Lazy instance has been poisoned");
            let value2 = other.into_inner().expect("Lazy instance has been poisoned");
            function(value1, value2)
        })
    }
}

impl<T: Default> Default for Lazy<T> {
    /// Creates a lazy value that computes the default value of `T`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambars::control::Lazy;
    ///
    /// let lazy: Lazy<i32> = Lazy::default();
    /// assert_eq!(*lazy.force(), 0);
    /// ```
    fn default() -> Self {
        Self::new(T::default)
    }
}

impl<T: fmt::Debug, F> fmt::Debug for Lazy<T, F> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.state.load(Ordering::Acquire) {
            STATE_READY => match self.value.get() {
                Some(value) => fmt::Debug::fmt(value, formatter),
                None => formatter.write_str("<invalid>"),
            }
            STATE_EMPTY | STATE_COMPUTING => formatter.write_str("<uninit>"),
            STATE_POISONED => formatter.write_str("<poisoned>"),
            _ => unreachable!(),
        }
    }
}

impl<T: fmt::Display, F> fmt::Display for Lazy<T, F> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.state.load(Ordering::Acquire) {
            STATE_READY => match self.value.get() {
                Some(value) => fmt::Display::fmt(value, formatter),
                None => formatter.write_str("<invalid>"),
            }
            STATE_EMPTY | STATE_COMPUTING => formatter.write_str("<uninit>"),
            STATE_POISONED => formatter.write_str("<poisoned>"),
            _ => unreachable!(),
        }
    }
}


// Note: We intentionally do NOT implement Deref for Lazy.
//
// Reason: This makes the laziness explicit in the code.
// Users must explicitly call force() to access the value.

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::cell::Cell;
    use std::panic;

    // =========================================================================
    // Display Tests
    // =========================================================================

    #[rstest]
    fn test_display_unevaluated_lazy() {
        let lazy = Lazy::new(|| 42);
        assert_eq!(format!("{lazy}"), "<uninit>");
    }

    #[rstest]
    fn test_display_evaluated_lazy() {
        let lazy = Lazy::new(|| 42);
        let _ = lazy.force();
        assert_eq!(format!("{lazy}"), "42");
    }

    #[rstest]
    fn test_display_poisoned_lazy() {
        let lazy = Lazy::new(|| -> i32 { panic!("initialization failed") });
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| lazy.force()));
        assert_eq!(format!("{lazy}"), "<poisoned>");
    }

    // =========================================================================
    // Original Tests
    // =========================================================================

    #[rstest]
    fn test_lazy_basic_creation() {
        let lazy = Lazy::new(|| 42);
        assert!(!lazy.is_initialized());
    }

    #[rstest]
    fn test_lazy_force_computes_value() {
        let lazy = Lazy::new(|| 42);
        let value = lazy.force();
        assert_eq!(*value, 42);
        assert!(lazy.is_initialized());
    }

    #[rstest]
    fn test_lazy_memoization() {
        let call_count = Cell::new(0);
        let lazy = Lazy::new(|| {
            call_count.set(call_count.get() + 1);
            42
        });

        assert_eq!(call_count.get(), 0);

        let _ = lazy.force();
        assert_eq!(call_count.get(), 1);

        let _ = lazy.force();
        assert_eq!(call_count.get(), 1); // Still 1, not 2
    }

    #[rstest]
    fn test_lazy_new_with_value() {
        let lazy = Lazy::new_with_value(42);
        assert!(lazy.is_initialized());
        assert_eq!(*lazy.force(), 42);
    }

    #[rstest]
    fn test_lazy_map() {
        let lazy = Lazy::new(|| 21);
        let doubled = lazy.map(|x| x * 2);
        assert_eq!(*doubled.force(), 42);
    }

    #[rstest]
    fn test_lazy_flat_map() {
        let lazy = Lazy::new(|| 21);
        let result = lazy.flat_map(|x| Lazy::new(move || x * 2));
        assert_eq!(*result.force(), 42);
    }

    #[rstest]
    fn test_lazy_get_before_init() {
        let lazy = Lazy::new(|| 42);
        assert!(lazy.get().is_none());
    }

    #[rstest]
    fn test_lazy_get_after_init() {
        let lazy = Lazy::new(|| 42);
        let _ = lazy.force();
        assert_eq!(*lazy.get().unwrap(), 42);
    }

    #[rstest]
    fn test_lazy_get_mut_before_init() {
        let mut lazy = Lazy::new(|| 42);
        assert!(lazy.get_mut().is_none());
    }

    #[rstest]
    fn test_lazy_get_mut_after_init() {
        let mut lazy = Lazy::new(|| 42);
        let _ = lazy.force();
        *lazy.get_mut().unwrap() = 100;
        assert_eq!(*lazy.force(), 100);
    }

    #[rstest]
    fn test_lazy_force_mut() {
        let mut lazy = Lazy::new(|| vec![1, 2, 3]);
        lazy.force_mut().push(4);
        assert_eq!(lazy.force().as_slice(), &[1, 2, 3, 4]);
    }

    #[rstest]
    fn test_lazy_into_inner_uninit() {
        let lazy = Lazy::new(|| 42);
        assert_eq!(lazy.into_inner(), Ok(42));
    }

    #[rstest]
    fn test_lazy_into_inner_init() {
        let lazy = Lazy::new(|| 42);
        let _ = lazy.force();
        assert_eq!(lazy.into_inner(), Ok(42));
    }

    #[rstest]
    fn test_lazy_into_inner_poisoned() {
        let lazy = Lazy::new(|| -> i32 { panic!("initialization failed") });
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| lazy.force()));
        assert_eq!(lazy.into_inner(), Err(LazyPoisonedError));
    }

    #[rstest]
    fn test_lazy_zip() {
        let lazy1 = Lazy::new(|| 1);
        let lazy2 = Lazy::new(|| "hello");
        let combined = lazy1.zip(lazy2);
        assert_eq!(*combined.force(), (1, "hello"));
    }

    #[rstest]
    fn test_lazy_zip_with() {
        let lazy1 = Lazy::new(|| 20);
        let lazy2 = Lazy::new(|| 22);
        let sum = lazy1.zip_with(lazy2, |a, b| a + b);
        assert_eq!(*sum.force(), 42);
    }

    #[rstest]
    fn test_lazy_pure() {
        let lazy = Lazy::pure(42);
        assert!(lazy.is_initialized());
        assert_eq!(*lazy.force(), 42);
    }

    #[rstest]
    fn test_lazy_default() {
        let lazy: Lazy<i32> = Lazy::default();
        assert_eq!(*lazy.force(), 0);
    }

    #[rstest]
    fn test_lazy_poison_propagation() {
        let lazy = Lazy::new(|| -> i32 { panic!("test panic") });

        // First force panics
        let result1 = panic::catch_unwind(panic::AssertUnwindSafe(|| lazy.force()));
        assert!(result1.is_err());

        // Second force also panics (poisoned)
        let result2 = panic::catch_unwind(panic::AssertUnwindSafe(|| lazy.force()));
        assert!(result2.is_err());

        assert!(lazy.is_poisoned());
    }

    #[rstest]
    fn test_lazy_debug_uninit() {
        let lazy = Lazy::new(|| 42);
        assert_eq!(format!("{lazy:?}"), "<uninit>");
    }

    #[rstest]
    fn test_lazy_debug_init() {
        let lazy = Lazy::new(|| 42);
        let _ = lazy.force();
        assert_eq!(format!("{lazy:?}"), "42");
    }

    #[rstest]
    fn test_lazy_debug_poisoned() {
        let lazy = Lazy::new(|| -> i32 { panic!("initialization failed") });
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| lazy.force()));
        assert_eq!(format!("{lazy:?}"), "<poisoned>");
    }

    // =========================================================================
    // Drop Tests
    // =========================================================================

    #[rstest]
    fn test_lazy_drop_uninit() {
        // Should not panic when dropping uninitialized Lazy
        let _lazy = Lazy::new(|| 42);
    }

    #[rstest]
    fn test_lazy_drop_init() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        struct DropTracker {
            dropped: Arc<AtomicBool>,
        }
        impl Drop for DropTracker {
            fn drop(&mut self) {
                self.dropped.store(true, Ordering::SeqCst);
            }
        }

        let dropped = Arc::new(AtomicBool::new(false));
        let dropped_clone = dropped.clone();

        let lazy = Lazy::new(move || DropTracker {
            dropped: dropped_clone,
        });
        let _ = lazy.force();
        assert!(!dropped.load(Ordering::SeqCst));

        drop(lazy);
        assert!(dropped.load(Ordering::SeqCst));
    }

    // =========================================================================
    // Re-entry Tests
    // =========================================================================

    // Note: Testing actual recursive force() is difficult because Lazy is !Sync
    // and cannot easily be shared via Rc in the initializer without triggering
    // borrow checker issues. The STATE_COMPUTING case primarily protects against
    // internal implementation errors or edge cases.
    //
    // The panic message is still tested indirectly through the code coverage.

    #[rstest]
    fn test_lazy_into_inner_initializer_panic_is_typed_error() {
        let lazy = Lazy::new(|| -> i32 { panic!("into_inner panic test") });
        assert_eq!(lazy.into_inner(), Err(LazyPoisonedError));
    }

    #[rstest]
    fn test_lazy_try_force_initializer_panic_is_typed_error() {
        let lazy = Lazy::new(|| -> i32 { panic!("try_force panic test") });
        assert_eq!(lazy.try_force(), Err(LazyPoisonedError));
        assert!(lazy.is_poisoned());
        assert_eq!(lazy.try_force(), Err(LazyPoisonedError));
    }

    // =========================================================================
    // Functor/Monad Law Property Tests
    // =========================================================================

    mod law_property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Single evaluation: force() always returns the same value
            #[test]
            fn prop_lazy_memoization(x in any::<i64>()) {
                let lazy = Lazy::new(|| x);
                let v1 = *lazy.force();
                let v2 = *lazy.force();
                prop_assert_eq!(v1, v2);
                prop_assert_eq!(v1, x);
            }

            /// Functor identity law: lazy.map(|x| x) == lazy
            #[test]
            fn prop_lazy_functor_identity(x in any::<i64>()) {
                let lazy = Lazy::new(|| x);
                let mapped = Lazy::new(|| x).map(|v| v);
                prop_assert_eq!(*lazy.force(), *mapped.force());
            }

            /// Functor composition law: lazy.map(f).map(g) == lazy.map(|x| g(f(x)))
            #[test]
            fn prop_lazy_functor_composition(x in any::<i32>()) {
                let f = |v: i32| v.wrapping_add(1);
                let g = |v: i32| v.wrapping_mul(2);
                let lazy1 = Lazy::new(|| x).map(f).map(g);
                let lazy2 = Lazy::new(|| x).map(|v| g(f(v)));
                prop_assert_eq!(*lazy1.force(), *lazy2.force());
            }

            /// Monad left identity: pure(a).flat_map(f) == f(a)
            #[test]
            fn prop_lazy_monad_left_identity(x in any::<i32>()) {
                let f = |v: i32| Lazy::new(move || v.wrapping_mul(2));
                let lazy1 = Lazy::pure(x).flat_map(f);
                let lazy2 = f(x);
                prop_assert_eq!(*lazy1.force(), *lazy2.force());
            }

            /// Monad right identity: m.flat_map(pure) == m
            #[test]
            fn prop_lazy_monad_right_identity(x in any::<i32>()) {
                let lazy1 = Lazy::new(|| x);
                let lazy2 = Lazy::new(|| x).flat_map(Lazy::pure);
                prop_assert_eq!(*lazy1.force(), *lazy2.force());
            }

            /// Monad associativity: (m.flat_map(f)).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))
            #[test]
            fn prop_lazy_monad_associativity(x in any::<i32>()) {
                let f = |v: i32| Lazy::new(move || v.wrapping_add(1));
                let g = |v: i32| Lazy::new(move || v.wrapping_mul(2));
                let lazy1 = Lazy::new(|| x).flat_map(f).flat_map(g);
                let lazy2 = Lazy::new(|| x).flat_map(|v| f(v).flat_map(g));
                prop_assert_eq!(*lazy1.force(), *lazy2.force());
            }
        }
    }
}
