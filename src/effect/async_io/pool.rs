//! Fixed-size async task pool for efficient batch execution.
//!
//! This module provides `AsyncPool`, a fixed-capacity pool for managing
//! asynchronous tasks with backpressure support. The pool enforces a
//! maximum capacity for concurrent execution and a separate queue capacity
//! for pending tasks.
//!
//! # Design Philosophy
//!
//! `AsyncPool` is designed to:
//!
//! 1. **Limit resource usage**: By enforcing fixed capacities for both
//!    concurrent execution and queued tasks, the pool prevents unbounded
//!    memory growth from spawning too many futures.
//!
//! 2. **Provide backpressure**: When the queue is full, `spawn` will wait
//!    until space becomes available, while `try_spawn` returns an error
//!    immediately.
//!
//! 3. **Reduce poll overhead**: By batching futures and limiting concurrent
//!    execution, the number of active futures being polled is bounded.
//!
//! # Implementation
//!
//! Uses `tokio::sync::Semaphore` and `tokio::sync::mpsc` for:
//! - Robust backpressure without manual Waker management
//! - Automatic permit release on cancellation (drop safety)
//! - FIFO ordering guaranteed by bounded mpsc channel
//!
//! # Capacity Model
//!
//! - `capacity`: Maximum number of concurrently executing tasks
//! - `queue_capacity`: Maximum number of tasks waiting in the queue (must be `<= capacity`)
//! - Total tasks: `inflight + queued <= capacity + queue_capacity <= 2 * capacity`
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```rust,ignore
//! use lambars::effect::async_io::pool::AsyncPool;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut pool = AsyncPool::new(128).unwrap();
//!
//!     // Spawn futures into the pool (waits if queue is full)
//!     pool.spawn(async { 1 }).await.unwrap();
//!     pool.spawn(async { 2 }).await.unwrap();
//!     pool.spawn(async { 3 }).await.unwrap();
//!
//!     // Execute all futures with bounded concurrency
//!     let results = pool.run_all().await;
//!     assert_eq!(results.len(), 3);
//! }
//! ```
//!
//! With `try_spawn` for immediate feedback:
//!
//! ```rust,ignore
//! use lambars::effect::async_io::pool::{AsyncPool, PoolError};
//!
//! #[tokio::main]
//! async fn main() {
//!     // capacity=4, queue_capacity=3 (queue_capacity must be <= capacity)
//!     let mut pool = AsyncPool::with_queue_capacity(4, 3).unwrap();
//!
//!     // These will succeed (within queue capacity of 3)
//!     assert!(pool.try_spawn(async { 1 }).is_ok());
//!     assert!(pool.try_spawn(async { 2 }).is_ok());
//!     assert!(pool.try_spawn(async { 3 }).is_ok());
//!
//!     // This will fail immediately (queue full)
//!     assert_eq!(pool.try_spawn(async { 4 }), Err(PoolError::QueueFull));
//!
//!     let results = pool.run_all().await;
//!     assert_eq!(results.len(), 3);
//! }
//! ```

use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::Semaphore;
use tokio::sync::mpsc;

// =============================================================================
// Error Types
// =============================================================================

/// Errors that can occur when working with `AsyncPool`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolError {
    /// The pool capacity was set to zero.
    ///
    /// A pool must have at least one slot for futures.
    InvalidCapacity,

    /// The queue is full and cannot accept more futures.
    ///
    /// Use `spawn` to wait for space, or increase the queue capacity.
    QueueFull,

    /// The concurrency limit for `run_buffered` was set to zero.
    ///
    /// The limit must be at least 1.
    InvalidConcurrencyLimit,

    /// The internal queue or semaphore is closed.
    ///
    /// This is reported as a typed error rather than terminating the caller.
    PoolClosed,
}

impl fmt::Display for PoolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCapacity => {
                write!(formatter, "pool capacity must be greater than 0")
            }
            Self::QueueFull => {
                write!(
                    formatter,
                    "queue is full: cannot spawn more futures without exceeding capacity"
                )
            }
            Self::InvalidConcurrencyLimit => {
                write!(formatter, "concurrency limit must be greater than 0")
            }
            Self::PoolClosed => {
                write!(formatter, "pool queue is closed")
            }
        }
    }
}

impl Error for PoolError {}

/// Pure queue-state decision used by the implementation's error policy and formal regressions.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PoolEnqueueDecision {
    Enqueue = 0,
    QueueFull = 1,
    PoolClosed = 2,
}

/// Classifies the observable enqueue state without panicking.
#[doc(hidden)]
#[must_use]
pub const fn pool_enqueue_decision(
    semaphore_closed: bool,
    channel_closed: bool,
    no_permits: bool,
) -> PoolEnqueueDecision {
    if semaphore_closed || channel_closed {
        PoolEnqueueDecision::PoolClosed
    } else if no_permits {
        PoolEnqueueDecision::QueueFull
    } else {
        PoolEnqueueDecision::Enqueue
    }
}

/// Returns whether an AsyncPool capacity pair is valid.
///
/// This predicate is used directly by the production constructors and by the
/// Kani/Verus regression models.
#[doc(hidden)]
#[must_use]
pub const fn pool_capacity_is_valid(capacity: usize, queue_capacity: usize) -> bool {
    capacity > 0 && queue_capacity > 0 && queue_capacity <= capacity
}

// =============================================================================
// Type Aliases
// =============================================================================

/// Type alias for boxed futures stored in the pool.
type BoxedFuture<A> = Pin<Box<dyn Future<Output = A> + Send>>;

// =============================================================================
// AsyncPool
// =============================================================================

/// A fixed-capacity pool for managing asynchronous tasks.
///
/// `AsyncPool` provides a bounded container for futures with backpressure
/// support. The pool has separate limits for concurrent execution (`capacity`)
/// and queued tasks (`queue_capacity`).
///
/// # Implementation Details
///
/// This implementation uses:
/// - `tokio::sync::Semaphore` for queue permit management (backpressure)
/// - `tokio::sync::mpsc` bounded channel for FIFO task queue
///
/// This design eliminates manual Waker management and provides:
/// - Automatic permit release on cancellation (drop safety)
/// - FIFO ordering guaranteed by the channel
/// - Robust backpressure without polling overhead
///
/// # Type Parameters
///
/// - `A`: The output type of the futures in the pool.
///
/// # Capacity Model
///
/// - `capacity`: Maximum number of tasks executing concurrently during `run_all`
/// - `queue_capacity`: Maximum number of tasks that can be queued
///
/// # Backpressure Behavior
///
/// - `spawn`: Waits until queue space is available (cancellable via Future drop)
/// - `try_spawn`: Returns `Err(PoolError::QueueFull)` immediately if queue is full
///
/// # Concurrency Model
///
/// `run_all` and `run_buffered` require `&mut self`, preventing concurrent
/// execution with `spawn` on the same pool instance. This is intentional for
/// Rust's memory safety guarantees.
///
/// To allow concurrent spawning during execution, consider:
/// - Using separate pool instances for different task groups
/// - Wrapping the pool in `Arc<tokio::sync::Mutex<AsyncPool>>` (note: this may
///   cause deadlock if `spawn` waits while holding the lock)
/// - Using the spawn-drain cycle pattern: fill the queue, drain with `run_all`,
///   then fill again
///
/// # Examples
///
/// ```rust,ignore
/// use lambars::effect::async_io::pool::AsyncPool;
///
/// #[tokio::main]
/// async fn main() {
///     let mut pool = AsyncPool::new(10).unwrap();
///
///     for i in 0..10 {
///         pool.spawn(async move { i * 2 }).await.unwrap();
///     }
///
///     let results = pool.run_all().await;
///     assert_eq!(results.len(), 10);
/// }
/// ```
pub struct AsyncPool<A> {
    /// Maximum number of concurrent executions during `run_all`.
    capacity: usize,

    /// Maximum number of tasks that can be queued.
    queue_capacity: usize,

    /// Semaphore for queue permit management.
    /// Permits represent available queue slots.
    queue_semaphore: Arc<Semaphore>,

    /// Sender for the bounded mpsc channel.
    sender: mpsc::Sender<BoxedFuture<A>>,

    /// Receiver for the bounded mpsc channel (wrapped in Mutex for shared access).
    receiver: Arc<Mutex<mpsc::Receiver<BoxedFuture<A>>>>,
}

impl<A> fmt::Debug for AsyncPool<A> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncPool")
            .field("capacity", &self.capacity)
            .field("queue_capacity", &self.queue_capacity)
            .finish_non_exhaustive()
    }
}

impl<A> AsyncPool<A> {
    /// Creates a new `AsyncPool` with the specified capacity.
    ///
    /// The `queue_capacity` defaults to the same value as `capacity`.
    ///
    /// # Errors
    ///
    /// Returns [`PoolError::InvalidCapacity`] when `capacity == 0`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use lambars::effect::async_io::pool::AsyncPool;
    ///
    /// let pool = AsyncPool::<i32>::new(128)?;
    /// assert_eq!(pool.capacity(), 128);
    /// assert_eq!(pool.queue_capacity(), 128);
    /// # Ok::<(), lambars::effect::async_io::pool::PoolError>(())
    /// ```
    pub fn new(capacity: usize) -> Result<Self, PoolError> {
        Self::try_new(capacity)
    }

    /// Creates a new `AsyncPool` with separate capacity and queue capacity.
    ///
    /// # Errors
    ///
    /// Returns [`PoolError::InvalidCapacity`] when either capacity is zero or
    /// `queue_capacity > capacity`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use lambars::effect::async_io::pool::AsyncPool;
    ///
    /// let pool = AsyncPool::<i32>::with_queue_capacity(10, 5)?;
    /// assert_eq!(pool.capacity(), 10);
    /// assert_eq!(pool.queue_capacity(), 5);
    /// # Ok::<(), lambars::effect::async_io::pool::PoolError>(())
    /// ```
    pub fn with_queue_capacity(
        capacity: usize,
        queue_capacity: usize,
    ) -> Result<Self, PoolError> {
        Self::try_with_queue_capacity(capacity, queue_capacity)
    }

    /// Tries to create a new `AsyncPool` with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The maximum number of concurrent executions.
    ///
    /// # Returns
    ///
    /// `Ok(AsyncPool)` if capacity is valid, `Err(PoolError::InvalidCapacity)` otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`PoolError::InvalidCapacity`] if `capacity` is 0.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use lambars::effect::async_io::pool::{AsyncPool, PoolError};
    ///
    /// let pool = AsyncPool::<i32>::try_new(10);
    /// assert!(pool.is_ok());
    ///
    /// let error = AsyncPool::<i32>::try_new(0);
    /// assert_eq!(error, Err(PoolError::InvalidCapacity));
    /// ```
    pub fn try_new(capacity: usize) -> Result<Self, PoolError> {
        Self::try_with_queue_capacity(capacity, capacity)
    }

    /// Tries to create a new `AsyncPool` with separate capacity and queue capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The maximum number of concurrent executions.
    /// * `queue_capacity` - The maximum number of tasks in the queue.
    ///   Must be `<= capacity` to satisfy the memory bound (inflight + queued <= 2 * capacity).
    ///
    /// # Returns
    ///
    /// `Ok(AsyncPool)` if capacities are valid, `Err(PoolError::InvalidCapacity)` otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`PoolError::InvalidCapacity`] if either capacity is 0 or if
    /// `queue_capacity > capacity`.
    pub fn try_with_queue_capacity(
        capacity: usize,
        queue_capacity: usize,
    ) -> Result<Self, PoolError> {
        if !pool_capacity_is_valid(capacity, queue_capacity) {
            return Err(PoolError::InvalidCapacity);
        }

        let queue_semaphore = Arc::new(Semaphore::new(queue_capacity));
        let (sender, receiver) = mpsc::channel(queue_capacity);

        Ok(Self {
            capacity,
            queue_capacity,
            queue_semaphore,
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        })
    }

    /// Returns the maximum concurrent execution capacity of the pool.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pool = AsyncPool::<i32>::new(100)?;
    /// assert_eq!(pool.capacity(), 100);
    /// ```
    #[must_use]
    #[inline]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the maximum queue capacity of the pool.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pool = AsyncPool::<i32>::with_queue_capacity(10, 50).unwrap();
    /// assert_eq!(pool.queue_capacity(), 50);
    /// ```
    #[must_use]
    #[inline]
    pub const fn queue_capacity(&self) -> usize {
        self.queue_capacity
    }

    /// Returns the current number of queued tasks.
    #[must_use]
    pub fn queue_len(&self) -> usize {
        self.queue_capacity
            .saturating_sub(self.queue_semaphore.available_permits())
    }

    /// Returns `true` if the queue is empty.
    #[must_use]
    pub fn is_queue_empty(&self) -> bool {
        self.queue_semaphore.available_permits() == self.queue_capacity
    }

    /// Returns `true` if the queue is at capacity.
    #[must_use]
    pub fn is_queue_full(&self) -> bool {
        self.queue_semaphore.available_permits() == 0
    }
}

impl<A: Send + 'static> AsyncPool<A> {
    /// Spawns a future into the pool, waiting if the queue is full.
    ///
    /// This method acquires a queue permit (waiting if necessary), then
    /// sends the future to the channel. The permit is "forgotten" and will
    /// be returned when the task is dequeued during `run_all`.
    ///
    /// # Cancellation
    ///
    /// The returned future can be cancelled by dropping it before completion.
    /// If cancelled during permit acquisition, the permit is automatically
    /// released (no leak). If cancelled after sending, the task remains
    /// in the queue.
    ///
    /// # Arguments
    ///
    /// * `future` - The future to add to the pool.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the future is successfully queued.
    ///
    /// # Errors
    ///
    /// This method does not return errors for queue-full conditions (it waits).
    /// It returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns [`PoolError::PoolClosed`] if the internal semaphore or queue is
    /// unexpectedly closed. This path is typed and never uses `expect()`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut pool = AsyncPool::<i32>::new(10)?;
    /// pool.spawn(async { 42 }).await.unwrap();
    /// ```
    ///
    /// With timeout:
    ///
    /// ```rust,ignore
    /// use tokio::time::{timeout, Duration};
    ///
    /// let mut pool = AsyncPool::<i32>::new(2).unwrap();
    /// // Fill the queue
    /// pool.try_spawn(async { 1 }).unwrap();
    /// pool.try_spawn(async { 2 }).unwrap();
    ///
    /// // This will timeout because the queue is full
    /// let result = timeout(Duration::from_millis(100), pool.spawn(async { 3 })).await;
    /// assert!(result.is_err());
    /// ```
    #[allow(clippy::significant_drop_tightening)]
    pub async fn spawn<F>(&self, future: F) -> Result<(), PoolError>
    where
        F: Future<Output = A> + Send + 'static,
    {
        let permit = self
            .queue_semaphore
            .acquire()
            .await
            .map_err(|_| PoolError::PoolClosed)?;

        let boxed_future: BoxedFuture<A> = Box::pin(future);
        self.sender
            .send(boxed_future)
            .await
            .map_err(|_| PoolError::PoolClosed)?;

        // Permit will be returned when dequeued in run_all
        permit.forget();

        Ok(())
    }

    /// Tries to spawn a future into the pool immediately.
    ///
    /// If the queue is full, this method returns `Err(PoolError::QueueFull)`
    /// immediately without waiting.
    ///
    /// # Arguments
    ///
    /// * `future` - The future to add to the pool.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the future was added successfully, `Err(PoolError::QueueFull)`
    /// if the queue is at capacity.
    ///
    /// # Errors
    ///
    /// Returns [`PoolError::QueueFull`] if the queue is at capacity.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pool = AsyncPool::<i32>::new(1).unwrap();
    /// assert!(pool.try_spawn(async { 1 }).is_ok());
    /// assert_eq!(pool.try_spawn(async { 2 }), Err(PoolError::QueueFull));
    /// ```
    pub fn try_spawn<F>(&self, future: F) -> Result<(), PoolError>
    where
        F: Future<Output = A> + Send + 'static,
    {
        let permit = match self.queue_semaphore.try_acquire() {
            Ok(permit) => permit,
            Err(tokio::sync::TryAcquireError::NoPermits) => {
                return Err(PoolError::QueueFull);
            }
            Err(tokio::sync::TryAcquireError::Closed) => {
                return Err(PoolError::PoolClosed);
            }
        };

        let boxed_future: BoxedFuture<A> = Box::pin(future);
        match self.sender.try_send(boxed_future) {
            Ok(()) => {
                permit.forget();
                Ok(())
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => Err(PoolError::QueueFull),
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => Err(PoolError::PoolClosed),
        }
    }

    /// Executes all futures in the pool, returning their results.
    ///
    /// This method drains the queue and executes all futures with bounded
    /// concurrency (limited to `capacity`). After execution, the pool is
    /// empty and can be reused.
    ///
    /// Tasks are dequeued in FIFO order, but execution start order is best-effort
    /// (depends on `buffer_unordered` internal scheduling).
    ///
    /// # Returns
    ///
    /// A `Vec<A>` containing the results of all futures. The order of results
    /// may not match the order futures were spawned due to concurrent execution.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut pool = AsyncPool::<i32>::new(10)?;
    /// pool.try_spawn(async { 1 }).unwrap();
    /// pool.try_spawn(async { 2 }).unwrap();
    ///
    /// let results = pool.run_all().await;
    /// assert_eq!(results.len(), 2);
    /// ```
    pub async fn run_all(&mut self) -> Vec<A> {
        use futures::stream::StreamExt;

        let mut futures: Vec<BoxedFuture<A>> = Vec::new();
        {
            let mut receiver = self.receiver.lock().await;
            while let Ok(future) = receiver.try_recv() {
                futures.push(future);
            }
        }

        let drained_count = futures.len();
        if drained_count > 0 {
            self.queue_semaphore.add_permits(drained_count);
        }

        if futures.is_empty() {
            return Vec::new();
        }

        futures::stream::iter(futures)
            .buffer_unordered(self.capacity)
            .collect()
            .await
    }

    /// Executes all futures with the specified concurrency limit.
    ///
    /// This method is similar to `run_all`, but allows overriding the
    /// concurrency limit for this specific execution. The effective limit
    /// is capped at `capacity` to maintain the pool's concurrency guarantee.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of concurrent executions. Must be > 0.
    ///   Values greater than `capacity` are capped to `capacity`.
    ///
    /// # Returns
    ///
    /// `Ok(Vec<A>)` with all results, or `Err(PoolError::InvalidConcurrencyLimit)`
    /// if limit is 0.
    ///
    /// # Errors
    ///
    /// Returns [`PoolError::InvalidConcurrencyLimit`] if `limit` is 0.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut pool = AsyncPool::<i32>::new(100)?;
    /// for i in 0..100 {
    ///     pool.try_spawn(async move { i }).unwrap();
    /// }
    ///
    /// // Execute with at most 10 concurrent futures
    /// let results = pool.run_buffered(10).await.unwrap();
    /// assert_eq!(results.len(), 100);
    /// ```
    pub async fn run_buffered(&mut self, limit: usize) -> Result<Vec<A>, PoolError> {
        use futures::stream::StreamExt;

        if limit == 0 {
            return Err(PoolError::InvalidConcurrencyLimit);
        }

        let effective_limit = limit.min(self.capacity);

        let mut futures: Vec<BoxedFuture<A>> = Vec::new();
        {
            let mut receiver = self.receiver.lock().await;
            while let Ok(future) = receiver.try_recv() {
                futures.push(future);
            }
        }

        let drained_count = futures.len();
        if drained_count > 0 {
            self.queue_semaphore.add_permits(drained_count);
        }

        if futures.is_empty() {
            return Ok(Vec::new());
        }

        Ok(futures::stream::iter(futures)
            .buffer_unordered(effective_limit)
            .collect()
            .await)
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // =========================================================================
    // AsyncPool::new Tests
    // =========================================================================

    #[rstest]
    fn new_creates_pool_with_capacity() {
        let pool = AsyncPool::<i32>::new(10).unwrap();
        assert_eq!(pool.capacity(), 10);
        assert_eq!(pool.queue_capacity(), 10);
    }

    #[rstest]
    fn new_returns_error_on_zero_capacity() {
        assert_eq!(
            AsyncPool::<i32>::new(0).unwrap_err(),
            PoolError::InvalidCapacity
        );
    }

    // =========================================================================
    // AsyncPool::with_queue_capacity Tests
    // =========================================================================

    #[rstest]
    fn with_queue_capacity_creates_pool_with_different_capacities() {
        let pool = AsyncPool::<i32>::with_queue_capacity(10, 5).unwrap();
        assert_eq!(pool.capacity(), 10);
        assert_eq!(pool.queue_capacity(), 5);
    }

    #[rstest]
    fn with_queue_capacity_returns_error_on_zero_capacity() {
        assert_eq!(
            AsyncPool::<i32>::with_queue_capacity(0, 10).unwrap_err(),
            PoolError::InvalidCapacity
        );
    }

    #[rstest]
    fn with_queue_capacity_returns_error_on_zero_queue_capacity() {
        assert_eq!(
            AsyncPool::<i32>::with_queue_capacity(10, 0).unwrap_err(),
            PoolError::InvalidCapacity
        );
    }

    #[rstest]
    fn with_queue_capacity_returns_error_when_queue_exceeds_capacity() {
        assert_eq!(
            AsyncPool::<i32>::with_queue_capacity(10, 50).unwrap_err(),
            PoolError::InvalidCapacity
        );
    }

    // =========================================================================
    // AsyncPool::try_new Tests
    // =========================================================================

    #[rstest]
    fn try_new_returns_ok_for_valid_capacity() {
        let result = AsyncPool::<i32>::try_new(5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().capacity(), 5);
    }

    #[rstest]
    fn try_new_returns_err_for_zero_capacity() {
        let result = AsyncPool::<i32>::try_new(0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PoolError::InvalidCapacity);
    }

    // =========================================================================
    // AsyncPool::try_with_queue_capacity Tests
    // =========================================================================

    #[rstest]
    fn try_with_queue_capacity_returns_ok_for_valid_capacities() {
        let result = AsyncPool::<i32>::try_with_queue_capacity(10, 5);
        assert!(result.is_ok());
        let pool = result.unwrap();
        assert_eq!(pool.capacity(), 10);
        assert_eq!(pool.queue_capacity(), 5);
    }

    #[rstest]
    fn try_with_queue_capacity_returns_err_for_zero_capacity() {
        let result = AsyncPool::<i32>::try_with_queue_capacity(0, 10);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PoolError::InvalidCapacity);
    }

    #[rstest]
    fn try_with_queue_capacity_returns_err_for_zero_queue_capacity() {
        let result = AsyncPool::<i32>::try_with_queue_capacity(10, 0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PoolError::InvalidCapacity);
    }

    #[rstest]
    fn try_with_queue_capacity_returns_err_when_queue_capacity_exceeds_capacity() {
        let result = AsyncPool::<i32>::try_with_queue_capacity(5, 10);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PoolError::InvalidCapacity);
    }

    // =========================================================================
    // AsyncPool::try_spawn Tests
    // =========================================================================

    #[rstest]
    fn try_spawn_adds_future_to_queue() {
        let pool = AsyncPool::<i32>::new(5).unwrap();
        let result = pool.try_spawn(async { 1 });
        assert!(result.is_ok());
        assert_eq!(pool.queue_len(), 1);
    }

    #[rstest]
    fn try_spawn_fails_when_queue_full() {
        let pool = AsyncPool::<i32>::new(1).unwrap();
        pool.try_spawn(async { 1 }).unwrap();
        let result = pool.try_spawn(async { 2 });
        assert_eq!(result, Err(PoolError::QueueFull));
    }

    #[rstest]
    fn try_spawn_respects_queue_capacity() {
        let pool = AsyncPool::<i32>::with_queue_capacity(10, 2).unwrap();
        pool.try_spawn(async { 1 }).unwrap();
        pool.try_spawn(async { 2 }).unwrap();
        let result = pool.try_spawn(async { 3 });
        assert_eq!(result, Err(PoolError::QueueFull));
    }

    // =========================================================================
    // Queue State Tests
    // =========================================================================

    #[rstest]
    fn queue_len_reflects_spawned_futures() {
        let pool = AsyncPool::<i32>::new(10).unwrap();
        assert_eq!(pool.queue_len(), 0);

        pool.try_spawn(async { 1 }).unwrap();
        assert_eq!(pool.queue_len(), 1);

        pool.try_spawn(async { 2 }).unwrap();
        assert_eq!(pool.queue_len(), 2);
    }

    #[rstest]
    fn is_queue_empty_returns_true_for_empty_pool() {
        let pool = AsyncPool::<i32>::new(10).unwrap();
        assert!(pool.is_queue_empty());

        pool.try_spawn(async { 1 }).unwrap();
        assert!(!pool.is_queue_empty());
    }

    #[rstest]
    fn is_queue_full_returns_true_when_at_capacity() {
        let pool = AsyncPool::<i32>::with_queue_capacity(10, 2).unwrap();
        assert!(!pool.is_queue_full());

        pool.try_spawn(async { 1 }).unwrap();
        assert!(!pool.is_queue_full());

        pool.try_spawn(async { 2 }).unwrap();
        assert!(pool.is_queue_full());
    }

    // =========================================================================
    // PoolError Tests
    // =========================================================================

    #[rstest]
    fn pool_error_display() {
        assert_eq!(
            PoolError::InvalidCapacity.to_string(),
            "pool capacity must be greater than 0"
        );
        assert!(PoolError::QueueFull.to_string().contains("queue is full"));
        assert!(
            PoolError::InvalidConcurrencyLimit
                .to_string()
                .contains("concurrency limit")
        );
    }

    #[rstest]
    fn pool_error_debug() {
        let debug = format!("{:?}", PoolError::QueueFull);
        assert!(debug.contains("QueueFull"));
    }

    // =========================================================================
    // Debug Implementation Tests
    // =========================================================================

    #[rstest]
    fn debug_shows_pool_state() {
        let pool = AsyncPool::<i32>::with_queue_capacity(20, 10).unwrap();
        let debug = format!("{pool:?}");
        assert!(debug.contains("AsyncPool"));
        assert!(debug.contains("capacity: 20"));
        assert!(debug.contains("queue_capacity: 10"));
    }
}
