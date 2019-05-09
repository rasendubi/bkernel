use futures::{Poll, Async, Future, AsyncSink, Stream, Sink};
use futures::stream::Fuse;

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct StartSendAll<T, U: Stream> {
    sink: Option<T>,
    stream: Option<Fuse<U>>,
    buffered: Option<U::Item>,
}

#[allow(dead_code)]
pub fn new<T, U>(sink: T, stream: U) -> StartSendAll<T, U>
    where T: Sink,
          U: Stream<Item = T::SinkItem>,
          T::SinkError: From<U::Error>,
{
    StartSendAll {
        sink: Some(sink),
        stream: Some(stream.fuse()),
        buffered: None,
    }
}

impl<T, U> StartSendAll<T, U>
    where T: Sink,
          U: Stream<Item = T::SinkItem>,
          T::SinkError: From<U::Error>,
{
    fn sink_mut(&mut self) -> &mut T {
        self.sink.as_mut().take().expect("")
    }

    fn stream_mut(&mut self) -> &mut Fuse<U> {
        self.stream.as_mut().take().expect("")
    }

    fn take_result(&mut self) -> (T, U) {
        let sink = self.sink.take().expect("");
        let fuse = self.stream.take().expect("");
        (sink, fuse.into_inner())
    }

    fn try_start_send(&mut self, item: U::Item) -> Poll<(), T::SinkError> {
        debug_assert!(self.buffered.is_none());
        if let AsyncSink::NotReady(item) = self.sink_mut().start_send(item)? {
            self.buffered = Some(item);
            return Ok(Async::NotReady)
        }
        Ok(Async::Ready(()))
    }
}

impl<T, U> Future for StartSendAll<T, U>
    where T: Sink,
          U: Stream<Item = T::SinkItem>,
          T::SinkError: From<U::Error>,
{
    type Item = (T, U);
    type Error = T::SinkError;

    fn poll(&mut self) -> Poll<(T, U), T::SinkError> {
        // If we've got an item buffered already, we need to write it to the
        // sink before we can do anything else
        if let Some(item) = self.buffered.take() {
            try_ready!(self.try_start_send(item))
        }

        loop {
            match self.stream_mut().poll()? {
                Async::Ready(Some(item)) => try_ready!(self.try_start_send(item)),
                Async::Ready(None) => {
                    return Ok(Async::Ready(self.take_result()))
                }
                Async::NotReady => {
                    return Ok(Async::NotReady)
                }
            }
        }
    }
}
