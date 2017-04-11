//! Logging.

use stm32f4::IrqLock;
use stm32f4::usart::{self, USART1};

use lock_free::CircularBuffer;

use futures::{Async, AsyncSink, Sink, Stream, StartSend, Poll};

use core::intrinsics::unreachable;

pub struct IoBuffer {
    intern: CircularBuffer<u8>,
}

impl IoBuffer {
    pub const fn new() -> IoBuffer {
        IoBuffer {
            intern: CircularBuffer::new(0),
        }
    }

    pub fn try_pop(&mut self) -> Option<u8> {
        self.intern.pop()
    }

    pub fn try_push(&mut self, item: u8) -> bool {
        self.intern.push(item)
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
        if self.intern.push(item) {
            // TODO(rasen): I don't like this.
            //
            // That means we can't use IoBuffer for something other
            // than Usart1.
            //
            // On the other hand, the Sink implementation is trivial,
            // so it might be easy to add more wrappers.
            // It would be even easier if CircularBuffer implements
            // Sink.
            new_data_added();

            Ok(AsyncSink::Ready)
        } else {
            Ok(AsyncSink::NotReady(item))
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        if self.intern.was_empty() {
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
        match self.try_pop() {
            Some(x) => Ok(Async::Ready(Some(x))),
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

fn new_data_added() {
    unsafe {
        if !USART1.it_enabled(usart::Interrupt::TXE) {
            if let Some(c) = STDOUT.try_pop() {
                let _irq_lock = IrqLock::new();
                USART1.it_enable(usart::Interrupt::TXE);
                USART1.put_unsafe(c as u32);
            } else {
                unreachable();
            }
        }
    }
}
