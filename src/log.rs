//! Logging.

use stm32f4::usart::{self, USART2};

use lock_free::CircularBuffer;

use futures::{Async, AsyncSink, Sink, Stream, StartSend, Poll};

use core::sync::atomic::{AtomicU32, Ordering};
use ::core::array::FixedSizeArray;

use super::REACTOR;

#[allow(missing_debug_implementations)]
pub struct IoBuffer<A: FixedSizeArray<u8>> {
    writer_task_mask: AtomicU32,
    reader_task_mask: AtomicU32,
    intern: CircularBuffer<u8, A>,
}

impl<A: FixedSizeArray<u8>> IoBuffer<A> {
    pub const fn new(init: A) -> IoBuffer<A> {
        IoBuffer {
            writer_task_mask: AtomicU32::new(0),
            reader_task_mask: AtomicU32::new(0),
            intern: CircularBuffer::new(init),
        }
    }

    pub fn try_pop(&mut self) -> Option<u8> {
        let res = self.intern.pop();
        if res.is_some() {
            let task_mask = self.writer_task_mask.swap(0, Ordering::SeqCst);
            REACTOR.set_ready_task_mask(task_mask);
        }
        res
    }

    pub fn try_push(&mut self, item: u8) -> bool {
        let res = self.intern.push(item);
        if res {
            let task_mask = self.reader_task_mask.swap(0, Ordering::SeqCst);
            REACTOR.set_ready_task_mask(task_mask);
        }
        res
    }

    // pub fn force_put(&mut self, item: u8) {
    //     let _irq_lock = unsafe { IrqLock::new() };

    //     if self.size == self.buffer.len() {
    //         self.size -= 1;
    //         self.cur = (self.cur + 1) % self.buffer.len();
    //     }

    //     let idx = (self.cur + self.size) % self.buffer.len();
    //     self.buffer[idx] = item;
    //     self.size += 1;
    // }
}

impl<A: FixedSizeArray<u8>> Sink for &'static mut IoBuffer<A> {
    type SinkItem = u8;
    type SinkError = ();

    fn start_send(&mut self, item: u8) -> StartSend<u8, Self::SinkError> {
        self.writer_task_mask.store(REACTOR.get_current_task_mask(), Ordering::SeqCst);

        if self.try_push(item) {
            self.writer_task_mask.store(0, Ordering::SeqCst);

            // This triggers TXE interrupt if transmitter is already
            // empty, so the USART catches up with new data.
            unsafe{&USART2}.it_enable(usart::Interrupt::TXE);

            Ok(AsyncSink::Ready)
        } else {
            Ok(AsyncSink::NotReady(item))
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.writer_task_mask.store(REACTOR.get_current_task_mask(), Ordering::SeqCst);

        if self.intern.was_empty() {
            self.writer_task_mask.store(0, Ordering::SeqCst);
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        self.poll_complete()
    }
}

impl<A: FixedSizeArray<u8>> Stream for &'static mut IoBuffer<A> {
    type Item = u8;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<u8>, Self::Error> {
        self.reader_task_mask.store(REACTOR.get_current_task_mask(), Ordering::SeqCst);

        match self.try_pop() {
            Some(x) => {
                self.reader_task_mask.store(0, Ordering::SeqCst);
                Ok(Async::Ready(Some(x)))
            },
            None => Ok(Async::NotReady),
        }
    }
}

/// This is very bad implementation for several reasons:
///
/// 1. It fails when the buffer is full, printing only the first part
/// of the string.
///
/// 2. It requires getting a mutable reference to the buffer, which is
/// not safe.
impl<A: FixedSizeArray<u8>> ::core::fmt::Write for IoBuffer<A> {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for b in s.as_bytes() {
            if !self.try_push(*b) {
                unsafe{&USART2}.it_enable(usart::Interrupt::TXE);

                return Err(::core::fmt::Error);
            }
        }

        unsafe{&USART2}.it_enable(usart::Interrupt::TXE);

        Ok(())
    }
}

pub static mut STDOUT: IoBuffer<[u8; 128]> = IoBuffer::new([0; 128]);
pub static mut STDIN: IoBuffer<[u8; 128]> = IoBuffer::new([0; 128]);

#[no_mangle]
pub unsafe extern fn __isr_usart2() {
    if USART2.it_status(usart::Interrupt::RXNE) {
        let c = USART2.get_unsafe();
        // If the buffer is full, we discard _new_ input.
        // That's not ideal :(
        let _ = STDIN.try_push(c);
    }

    if USART2.it_status(usart::Interrupt::TXE) {
        if let Some(c) = STDOUT.try_pop() {
            USART2.put_unsafe(c);
        } else {
            USART2.it_disable(usart::Interrupt::TXE);
        }
    }
}
