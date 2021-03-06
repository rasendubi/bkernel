//! Future-based USART driver.

use core::pin::Pin;
use core::task::Context;
use stm32f4::usart;

use crate::circular_buffer::CircularBuffer;
use crate::resettable_stream::ResettableStream;

use futures::{Poll, Sink, Stream};

use core::array::FixedSizeArray;
use core::sync::atomic::{AtomicU32, Ordering};

use breactor::REACTOR;

#[allow(missing_debug_implementations)]
pub struct Usart<A, B> {
    usart: &'static usart::Usart,
    writer_task_mask: AtomicU32,
    reader_task_mask: AtomicU32,
    writer_buffer: CircularBuffer<u8, A>,
    reader_buffer: CircularBuffer<u8, B>,
}

impl<A: FixedSizeArray<u8>, B: FixedSizeArray<u8>> Usart<A, B> {
    pub const fn new(
        usart: &'static usart::Usart,
        writer_buffer: A,
        reader_buffer: B,
    ) -> Usart<A, B> {
        Usart {
            usart,
            writer_task_mask: AtomicU32::new(0),
            reader_task_mask: AtomicU32::new(0),
            writer_buffer: CircularBuffer::new(writer_buffer),
            reader_buffer: CircularBuffer::new(reader_buffer),
        }
    }

    pub fn try_push_writer(&self, item: u8) -> bool {
        let res = self.writer_buffer.push(item);
        if res {
            self.writer_task_mask.store(0, Ordering::SeqCst);

            // This triggers TXE interrupt if transmitter is already
            // empty, so the USART catches up with new data.
            self.usart.it_enable(usart::Interrupt::TXE);
        }
        res
    }

    pub fn try_pop_writer(&self) -> Option<u8> {
        let res = self.writer_buffer.pop();
        if res.is_some() {
            let task_mask = self.writer_task_mask.swap(0, Ordering::SeqCst);
            REACTOR.set_ready_task_mask(task_mask);
        }
        res
    }

    pub fn try_push_reader(&self, item: u8) -> bool {
        let res = self.reader_buffer.push(item);
        if res {
            let task_mask = self.reader_task_mask.swap(0, Ordering::SeqCst);
            REACTOR.set_ready_task_mask(task_mask);
        }
        res
    }

    pub fn try_pop_reader(&self) -> Option<u8> {
        self.reader_buffer.pop()
    }

    /// Interrupt service routine.
    ///
    /// It should be called for the corresponding USART interrupt.
    ///
    /// # Example
    /// ```no_run
    /// # #![feature(const_fn)]
    /// # extern crate dev;
    /// # extern crate stm32f4;
    /// # use dev::usart::Usart;
    /// static USART2: Usart<[u8; 32], [u8;32]> = Usart::new(unsafe {&stm32f4::usart::USART2}, [0; 32], [0;32]);
    ///
    /// pub unsafe extern fn __isr_usart2() {
    ///     USART2.isr()
    /// }
    /// # pub fn main() {
    /// # }
    /// ```
    pub unsafe fn isr(&self) {
        if self.usart.it_status(usart::Interrupt::RXNE) {
            let c = self.usart.get_unsafe();
            // If the buffer is full, we discard _new_ input.
            // That's not ideal :(
            let _ = self.try_push_reader(c);
        }

        if self.usart.it_status(usart::Interrupt::TXE) {
            if let Some(c) = self.try_pop_writer() {
                self.usart.put_unsafe(c);
            } else {
                self.usart.it_disable(usart::Interrupt::TXE);
            }
        }
    }
}

impl<'a, A: FixedSizeArray<u8>, B: FixedSizeArray<u8>> Sink<u8> for &'a Usart<A, B> {
    type SinkError = ();

    fn poll_ready(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::SinkError>> {
        self.writer_task_mask
            .store(REACTOR.get_current_task_mask(), Ordering::SeqCst);

        if self.writer_buffer.was_full() {
            Poll::Pending
        } else {
            self.writer_task_mask.store(0, Ordering::SeqCst);

            Poll::Ready(Ok(()))
        }
    }

    fn start_send(self: Pin<&mut Self>, item: u8) -> Result<(), Self::SinkError> {
        if self.try_push_writer(item) {
            // This triggers TXE interrupt if transmitter was
            // empty, so the USART catches up with new data.
            self.usart.it_enable(usart::Interrupt::TXE);

            Ok(())
        } else {
            panic!("Usart: start_send was called, but the queue is not ready");
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::SinkError>> {
        self.writer_task_mask
            .store(REACTOR.get_current_task_mask(), Ordering::SeqCst);

        if self.writer_buffer.was_empty() {
            self.writer_task_mask.store(0, Ordering::SeqCst);
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::SinkError>> {
        self.poll_flush(cx)
    }
}

impl<'a, A: FixedSizeArray<u8>, B: FixedSizeArray<u8>> Stream for &'a Usart<A, B> {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.reader_task_mask
            .store(REACTOR.get_current_task_mask(), Ordering::SeqCst);

        match self.try_pop_reader() {
            Some(x) => {
                self.reader_task_mask.store(0, Ordering::SeqCst);
                Poll::Ready(Some(x))
            }
            None => Poll::Pending,
        }
    }
}

impl<'a, A: FixedSizeArray<u8>, B: FixedSizeArray<u8>> ResettableStream for &'a Usart<A, B> {
    fn reset(&mut self) {
        while let Some(_) = self.try_pop_reader() {}
    }
}
