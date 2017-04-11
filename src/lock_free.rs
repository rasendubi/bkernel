//! Lock-free structures.

use core::sync::atomic::{AtomicUsize, Ordering};

/// A lock-free _single-producer_, _single-consumer_ buffer.
///
/// Do *NOT* use it with either multiple producers or multiple
/// consumers.
///
/// It is currently hard-coded to save 31 element maximum.
/// Waiting for the [Pi trilogy][rust-lang/rfcs#1930] to
/// complete. (Actually, core one would be enough.)
///
/// [rust-lang/rfcs#1930]: https://github.com/rust-lang/rfcs/issues/1930
pub struct CircularBuffer<T> {
    array: [T; 32],
    tail: AtomicUsize,
    head: AtomicUsize,
}

impl<T: Copy> CircularBuffer<T> {
    /// Construct a new CircularBuffer initializing all elements to
    /// `init`.
    ///
    /// Note that you can't access these values and it is there merely
    /// to make this function `const`.
    ///
    /// ::core::mem::uninitialized would work here, but it is not
    /// const. (Have no idea why.)
    pub const fn new(init: T) -> CircularBuffer<T> {
        CircularBuffer {
            array: [init; 32],
            tail: AtomicUsize::new(0),
            head: AtomicUsize::new(0),
        }
    }

    const fn increment(idx: usize) -> usize {
        (idx + 1) % 32
    }

    /// Push an item into the buffer.
    ///
    /// Returns `true` if push was successful.
    /// `false` means the buffer was full.
    pub fn push(&mut self, item: T) -> bool {
        let current_tail = self.tail.load(Ordering::Relaxed);
        let next_tail = Self::increment(current_tail);
        if next_tail == self.head.load(Ordering::Acquire) {
            // Queue is full
            false
        } else {
            self.array[current_tail] = item;
            self.tail.store(next_tail, Ordering::Release);

            true
        }
    }

    /// Pops element from the buffer.
    ///
    /// `None` means the buffer was empty.
    pub fn pop(&mut self) -> Option<T> {
        let current_head = self.head.load(Ordering::Relaxed);
        if current_head == self.tail.load(Ordering::Acquire) {
            None
        } else {
            let item = self.array[current_head];
            self.head.store(Self::increment(current_head), Ordering::Release);

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_init_is_empty() {
        let mut cb = CircularBuffer::new(0);
        assert_eq!(None, cb.pop());
    }

    #[test]
    fn test_push_pop() {
        let mut cb = CircularBuffer::new(0);
        assert_eq!(true, cb.push(5));
        assert_eq!(Some(5), cb.pop());
    }
}
