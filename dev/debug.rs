use futures::prelude::*;
use std::collections::VecDeque;

use crate::resettable_stream::ResettableStream;

/// A `Sink + Stream` implementation backed by `Vec` and `VecDeque`. Should only be used for
/// testing.
pub struct TestChannel<T> {
    sink: Vec<T>,
    stream: VecDeque<T>,
}

impl<T> TestChannel<T> {
    pub fn new() -> TestChannel<T> {
        TestChannel {
            sink: Vec::new(),
            stream: VecDeque::new(),
        }
    }

    pub fn sink(&self) -> &Vec<T> {
        &self.sink
    }

    pub fn stream(&mut self) -> &mut VecDeque<T> {
        &mut self.stream
    }
}

impl<T> Sink<T> for TestChannel<T> {
    type SinkError = ();

    fn start_send(&mut self, item: T) -> StartSend<T, Self::SinkError> {
        self.sink.push(item);
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        self.poll_complete()
    }
}

impl<T> Stream for TestChannel<T> {
    type Item = T;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(self.stream.pop_front().into())
    }
}

impl<T> ResettableStream for TestChannel<T> {
    fn reset(&mut self) {}
}
