//! Logging.

use stm32f4::IrqLock;
use stm32f4::usart::{self, USART1};

use futures::{Async, AsyncSink, Sink, Stream, StartSend, Poll};

pub struct IoBuffer {
    buffer: [u8; 32],
    size: usize,
    cur: usize,
}

impl IoBuffer {
    pub const fn new() -> IoBuffer {
        IoBuffer {
            buffer: [0; 32],
            size: 0,
            cur: 0,
        }
    }

    pub fn try_get(&mut self) -> Option<u8> {
        let _irq_lock = unsafe { IrqLock::new() };

        if self.size > 0 {
            let result = self.buffer[self.cur];
            self.cur = (self.cur + 1) % self.buffer.len();
            self.size -= 1;

            Some(result)
        } else {
            None
        }
    }

    pub fn force_put(&mut self, item: u8) {
        let _irq_lock = unsafe { IrqLock::new() };

        if self.size == self.buffer.len() {
            self.size -= 1;
            self.cur = (self.cur + 1) % self.buffer.len();
        }

        let idx = (self.cur + self.size) % self.buffer.len();
        self.buffer[idx] = item;
        self.size += 1;
    }
}

impl Sink for &'static mut IoBuffer {
    type SinkItem = u8;
    type SinkError = ();

    fn start_send(&mut self, item: u8) -> StartSend<u8, Self::SinkError> {
        let _irq_lock = unsafe { IrqLock::new() };

        if item == '\r' as u8 || item == '\n' as u8 {
            if self.size + 1 < self.buffer.len() {
                let idx = (self.cur + self.size) % self.buffer.len();
                self.buffer[idx] = '\r' as u8;
                self.size += 1;
                let idx = (self.cur + self.size) % self.buffer.len();
                self.buffer[idx] = '\n' as u8;
                self.size += 1;

                new_data_added();

                Ok(AsyncSink::Ready)
            } else {
                Ok(AsyncSink::NotReady(item))
            }
        } else if self.size < self.buffer.len() {
            let idx = (self.cur + self.size) % self.buffer.len();
            self.buffer[idx] = item;
            self.size += 1;

            new_data_added();

            Ok(AsyncSink::Ready)
        } else {
            Ok(AsyncSink::NotReady(item))
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        let _irq_lock = unsafe { IrqLock::new() };

        if self.size == 0 {
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
        match self.try_get() {
            Some(x) => Ok(Async::Ready(Some(x))),
            _ => Ok(Async::NotReady),
        }
    }
}

pub static mut LOGGER: IoBuffer = IoBuffer::new();
pub static mut INPUT: IoBuffer = IoBuffer::new();


#[no_mangle]
pub unsafe extern fn __isr_usart1() {
    if USART1.it_status(usart::Interrupt::RXNE) {
        let c = USART1.get_unsafe();
        INPUT.force_put(c as u8);
    }

    if USART1.it_status(usart::Interrupt::TXE) {
        match LOGGER.try_get() {
            Some(c) => {
                USART1.put_unsafe(c as u32);
            },
            None => {
                USART1.it_disable(usart::Interrupt::TXE)
            },
        }
    }
}

fn new_data_added() {
    unsafe {
        let _irq_lock = IrqLock::new();

        if !USART1.it_enabled(usart::Interrupt::TXE) {
            match LOGGER.try_get() {
                Some(c) => {
                    USART1.it_enable(usart::Interrupt::TXE);
                    USART1.put_unsafe(c as u32);
                },
                None => {},
            }
        }
    }
}
