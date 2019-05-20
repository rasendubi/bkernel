use core::pin::Pin;
use futures::task::Context;
use futures::{Future, Poll, Sink};

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct StartSendAllString<'a, T> {
    sink: Option<T>,
    string: &'a str,
    cur: usize,
}

impl<'a, T> Unpin for StartSendAllString<'a, T> where T: Sink<u8> + Unpin {}

impl<'a, T> StartSendAllString<'a, T>
where
    T: Sink<u8> + Unpin,
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
    T: Sink<u8>,
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
    T: Sink<u8> + Unpin,
{
    type Output = Result<T, T::SinkError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;

        while this.cur < this.string.as_bytes().len() {
            match Pin::new(this.sink_mut()).poll_ready(cx) {
                Poll::Ready(Ok(())) => {
                    let item = this.string.as_bytes()[this.cur];
                    Pin::new(this.sink_mut()).start_send(item)?;
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }

            this.cur += 1;
        }

        Poll::Ready(Ok(self.take_result()))
    }
}
