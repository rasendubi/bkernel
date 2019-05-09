use futures::{Async, AsyncSink, Future, Poll, Sink};

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct StartSendAllString<'a, T> {
    sink: Option<T>,
    string: &'a str,
    cur: usize,
}

impl<'a, T> StartSendAllString<'a, T>
where
    T: Sink<SinkItem = u8>,
{
    pub fn new(sink: T, string: &'a str) -> StartSendAllString<'a, T> {
        StartSendAllString {
            sink: Some(sink),
            string,
            cur: 0,
        }
    }
}

impl<'a, T> StartSendAllString<'a, T>
where
    T: Sink<SinkItem = u8>,
{
    fn sink_mut(&mut self) -> &mut T {
        self.sink.as_mut().take().expect("")
    }

    fn take_result(&mut self) -> T {
        self.sink.take().expect("")
    }
}

impl<'a, T> Future for StartSendAllString<'a, T>
where
    T: Sink<SinkItem = u8>,
{
    type Item = T;
    type Error = T::SinkError;

    fn poll(&mut self) -> Poll<T, T::SinkError> {
        while self.cur < self.string.as_bytes().len() {
            let item = self.string.as_bytes()[self.cur];
            if let AsyncSink::NotReady(_) = self.sink_mut().start_send(item)? {
                return Ok(Async::NotReady);
            }

            self.cur += 1;
        }
        Ok(Async::Ready(self.take_result()))
    }
}
