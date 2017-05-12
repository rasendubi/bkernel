//! Lock-free synchronization point for single-value single-producer,
//! single-consumer.
use ::core::cell::UnsafeCell;
use ::core::sync::atomic::{AtomicU32, Ordering};

use ::futures::{Async, Future, Poll};

use super::REACTOR;

/// Promise provides a lock-free synchronization point for producer
/// and consumer of the data, with Future-aware interface.
///
/// The promise can be shared between one producer and one consumer.
///
/// The consumer is assumed to hold the object and should not drop it
/// until it is resolved.
pub struct Promise<T, E> {
    /// Stores the mask of the owning task.
    ///
    /// If `task` is `0`, the Promise have been resolved.
    task: AtomicU32,

    /// Stores the result of Promise.
    ///
    /// When `task` is non-zero, result stores `None`, and should only
    /// be written by the producer.
    ///
    /// When `task` is zero, the result stores `Some`, and should only
    /// be read by the consumer.
    result: UnsafeCell<Option<Result<T, E>>>,
}

unsafe impl<T, E> Sync for Promise<T, E> {
}

impl<T, E> Promise<T, E> {
    /// Creates an empty Promise.
    ///
    /// The promise must be claimed with `claim()` before calling
    /// `poll()` or `resolve()`.
    pub const unsafe fn empty() -> Promise<T, E> {
        Promise {
            task: AtomicU32::new(0),
            result: UnsafeCell::new(None),
        }
    }

    /// Creates new promise and makes it be owned by the current task.
    ///
    /// Should only be called from within a task.
    pub fn new() -> Promise<T, E> {
        Promise {
            task: AtomicU32::new(REACTOR.get_current_task_mask()),
            result: UnsafeCell::new(None),
        }
    }

    pub const fn new_task(task_mask: u32) -> Promise<T, E> {
        Promise {
            task: AtomicU32::new(task_mask),
            result: UnsafeCell::new(None),
        }
    }

    /// Set the currently executed task as an owner.
    ///
    /// Should only be called from within a task.
    pub fn claim(&self) {
        let task = REACTOR.get_current_task_mask();
        self.task.store(task, Ordering::Relaxed);
    }

    /// Resolves the Promise notifying the waiting task.
    ///
    /// This should be called by the producer. The producer is not
    /// allowed to use the object after calling `resolve()`.
    // TODO(rasen): create additional struct for producer's end,
    // which will consume on resolve?
    //
    // The Promise can track this end, which could allow dropping
    // promise before resolve.
    //
    // Also, I should consider making Promise be owned by the
    // producer and tracking consumer's future-part.
    pub fn resolve(&self, result: Result<T, E>) {
        unsafe {
            *self.result.get() = Some(result);
        }

        let task = self.task.swap(0, Ordering::Release);
        debug_assert!(task != 0);
        REACTOR.set_ready_task_mask(task);
    }

    /// Returns true, if the promise is already resolved or not
    /// initialized.
    ///
    /// This method is not thread-safe with respect to `resolve()`.
    pub fn is_resolved(&self) -> bool {
        self.task.load(Ordering::Acquire) == 0
    }
}

impl<T, E> Future for Promise<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<T, E> {
        let task = self.task.load(Ordering::Acquire);
        if task == 0 {
            match unsafe { ::core::ptr::replace(self.result.get(), None) }.unwrap() {
                Ok(x) => Ok(Async::Ready(x)),
                Err(x) => Err(x),
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}
