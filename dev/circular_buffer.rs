//! Lock-free ring buffer.
use ::core::array::FixedSizeArray;
use ::core::cell::UnsafeCell;
use ::core::marker::PhantomData;
use ::core::sync::atomic::{AtomicUsize, Ordering};

/// A lock-free _single-producer_, _single-consumer_ buffer.
///
/// Do *NOT* use it with either multiple producers or multiple
/// consumers.
#[allow(missing_debug_implementations)]
pub struct CircularBuffer<T, A> {
    array: UnsafeCell<A>,
    tail: AtomicUsize,
    head: AtomicUsize,
    __phantom: PhantomData<T>,
}

impl<T: Copy, A: FixedSizeArray<T>> CircularBuffer<T, A> {
    /// Construct a new CircularBuffer initializing all elements to
    /// `init`.
    ///
    /// Note that you can't access these values and it is there merely
    /// to make this function `const`.
    ///
    /// ::core::mem::uninitialized would work here, but it is not
    /// const. (Have no idea why.)
    pub const fn new(init: A) -> CircularBuffer<T, A> {
        CircularBuffer {
            array: UnsafeCell::new(init),
            tail: AtomicUsize::new(0),
            head: AtomicUsize::new(0),
            __phantom: PhantomData,
        }
    }

    fn increment(&self, idx: usize) -> usize {
        unsafe { (idx + 1) % (*self.array.get()).as_slice().len() }
    }

    /// Push an item into the buffer.
    ///
    /// Returns `true` if push was successful.
    /// `false` means the buffer was full.
    pub fn push(&self, item: T) -> bool {
        let current_tail = self.tail.load(Ordering::Relaxed);
        let next_tail = self.increment(current_tail);
        if next_tail == self.head.load(Ordering::Acquire) {
            // Queue is full
            false
        } else {
            unsafe {
                (*self.array.get()).as_mut_slice()[current_tail] = item;
            }
            self.tail.store(next_tail, Ordering::Release);

            true
        }
    }

    /// Pops element from the buffer.
    ///
    /// `None` means the buffer was empty.
    pub fn pop(&self) -> Option<T> {
        let current_head = self.head.load(Ordering::Relaxed);
        if current_head == self.tail.load(Ordering::Acquire) {
            None
        } else {
            let item = unsafe { &mut *self.array.get() }.as_slice()[current_head];
            self.head
                .store(self.increment(current_head), Ordering::Release);

            Some(item)
        }
    }

    /// If the buffer was empty at the time of querying.
    ///
    /// Note that the status may have already changed by the time the
    /// function returns.
    pub fn was_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed)
    }
}

unsafe impl<T, A: FixedSizeArray<T>> Sync for CircularBuffer<T, A> {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_init_is_empty() {
        let cb = CircularBuffer::new([0; 32]);
        assert_eq!(None, cb.pop());
    }

    #[test]
    fn test_push_pop() {
        let cb = CircularBuffer::new([0; 32]);
        assert_eq!(true, cb.push(5));
        assert_eq!(Some(5), cb.pop());
    }
}
