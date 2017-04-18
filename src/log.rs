//! Logging.

use stm32f4::usart::{self, USART1};

use lock_free::CircularBuffer;

use futures::{Async, AsyncSink, Sink, Stream, StartSend, Poll};

use core::sync::atomic::{AtomicU32, Ordering};

use super::REACTOR;

pub struct IoBuffer {
    writer_task_mask: AtomicU32,
    reader_task_mask: AtomicU32,
    intern: CircularBuffer<u8>,
}

impl IoBuffer {
    pub const fn new() -> IoBuffer {
        IoBuffer {
            writer_task_mask: AtomicU32::new(0),
            reader_task_mask: AtomicU32::new(0),
            intern: CircularBuffer::new(0),
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

impl Sink for &'static mut IoBuffer {
    type SinkItem = u8;
    type SinkError = ();

    fn start_send(&mut self, item: u8) -> StartSend<u8, Self::SinkError> {
        self.writer_task_mask.store(REACTOR.get_current_task_mask(), Ordering::SeqCst);

        if self.try_push(item) {
            self.writer_task_mask.store(0, Ordering::SeqCst);

            // This triggers TXE interrupt if transmitter is already
            // empty, so the USART catches up with new data.
            unsafe{&USART1}.it_enable(usart::Interrupt::TXE);

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

impl Stream for &'static mut IoBuffer {
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

pub static mut STDOUT: IoBuffer = IoBuffer::new();
pub static mut STDIN: IoBuffer = IoBuffer::new();

#[no_mangle]
pub unsafe extern fn __isr_usart1() {
    if USART1.it_status(usart::Interrupt::RXNE) {
        let c = USART1.get_unsafe();
        // If the buffer is full, we discard _new_ input.
        // That's not ideal :(
        let _ = STDIN.try_push(c as u8);
    }

    if USART1.it_status(usart::Interrupt::TXE) {
        if let Some(c) = STDOUT.try_pop() {
            USART1.put_unsafe(c as u32);
        } else {
            USART1.it_disable(usart::Interrupt::TXE);
        }
    }
}
