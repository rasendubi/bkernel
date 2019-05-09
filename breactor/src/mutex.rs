//! Mutual exclusion for futures.

use ::core::sync::atomic::{AtomicU32, Ordering};
use ::futures::{Async, Future, Poll};

use super::REACTOR;

/// Mutex guarantees exclusive access for a task.
///
/// Unlike pthread mutexes, this one is allowed to be released from a
/// different thread of execution.
///
/// This mutex is NOT recursive. The same task can not acquire it
/// without releasing the previous lock.
///
/// This mutex is lock-free.
///
/// ## Access tokens
/// The primary use case is issuing access tokens to shared resources
/// (e.g., buses).
///
/// ```
/// # #![feature(const_fn)]
/// # #![feature(conservative_impl_trait)]
/// # extern crate breactor;
/// # extern crate futures;
/// # use ::futures::Future;
/// # use breactor::mutex::*;
/// static BUS: Bus = Bus { mutex: Mutex::new() };
///
/// pub struct Bus {
///     mutex: Mutex,
/// }
///
/// pub struct AccessToken {
///     lock: MutexLock<'static>,
/// }
///
/// impl Bus {
///     pub fn access(&'static self) -> impl Future<Item=AccessToken, Error=()> {
///         self.mutex.lock().map(|lock| AccessToken { lock })
///     }
/// }
///
/// impl AccessToken {
///     pub fn exclusive_operation(&mut self) {
///         println!("This operation is exclusive");
///     }
/// }
///
/// # fn main() {
/// # }
/// ```
#[allow(missing_debug_implementations)]
pub struct Mutex {
    /// The tasks, that are currently waiting on the mutex.
    ///
    /// When the mutex is released, all those tasks are woken up. This
    /// usually results in the highest priority task acquiring a lock.
    wait_task_mask: AtomicU32,

    /// The current owner of the mutex lock.
    ///
    /// When 0, the mutex is empty.
    owner: AtomicU32,
}

/// If you have this lock, you have locked the underlying mutex.
#[allow(missing_debug_implementations)]
pub struct MutexLock<'a> {
    mutex: &'a Mutex,
}

#[allow(missing_debug_implementations)]
pub struct LockFuture<'a> {
    mutex: &'a Mutex,
}

impl<'a> Drop for MutexLock<'a> {
    fn drop(&mut self) {
        self.mutex.release()
    }
}

impl Mutex {
    /// Creates new empty mutex.
    pub const fn new() -> Mutex {
        Mutex {
            wait_task_mask: AtomicU32::new(0),
            owner: AtomicU32::new(0),
        }
    }

    /// Return a future that will eventually lock the given mutex.
    pub const fn lock(&self) -> LockFuture {
        LockFuture { mutex: self }
    }

    /// Release the mutex, notifying all waiting tasks.
    fn release(&self) {
        self.owner.store(0, Ordering::SeqCst);
        let tasks = self.wait_task_mask.swap(0, Ordering::SeqCst);
        REACTOR.set_ready_task_mask(tasks);
    }
}

impl<'a> Future for LockFuture<'a> {
    type Item = MutexLock<'a>;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let task = REACTOR.get_current_task_mask();

        self.mutex.wait_task_mask.fetch_or(task, Ordering::SeqCst);

        let prev = self.mutex.owner.compare_and_swap(0, task, Ordering::SeqCst);
        if prev == 0 {
            // Mutex locked
            Ok(Async::Ready(MutexLock { mutex: self.mutex }))
        } else {
            Ok(Async::NotReady)
        }
    }
}
