use core::pin::Pin;
use futures::stream::Fuse;
use futures::stream::StreamExt;
use futures::task::Context;
use futures::{Future, Poll, Sink, Stream};

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct StartSendAll<Si, St: Stream> {
    sink: Option<Si>,
    stream: Option<Fuse<St>>,
    buffered: Option<St::Item>,
}

impl<Si, St> Unpin for StartSendAll<Si, St>
where
    Si: Sink<St::Item> + Unpin,
    St: Stream + Unpin,
{
}

#[allow(dead_code)]
pub fn new<Si, St>(sink: Si, stream: St) -> StartSendAll<Si, St>
where
    Si: Sink<St::Item>,
    St: Stream,
{
    StartSendAll {
        sink: Some(sink),
        stream: Some(stream.fuse()),
        buffered: None,
    }
}

impl<Si, St> StartSendAll<Si, St>
where
    Si: Sink<St::Item> + Unpin,
    St: Stream + Unpin,
{
    fn sink_mut(&mut self) -> &mut Si {
        self.sink.as_mut().take().expect("")
    }

    fn stream_mut(&mut self) -> &mut Fuse<St> {
        self.stream.as_mut().take().expect("")
    }

    fn take_result(&mut self) -> (Si, St) {
        let sink = self.sink.take().expect("");
        let fuse = self.stream.take().expect("");
        (sink, fuse.into_inner())
    }

    fn try_start_send(
        &mut self,
        cx: &mut Context<'_>,
        item: St::Item,
    ) -> Poll<Result<(), Si::SinkError>> {
        debug_assert!(self.buffered.is_none());
        match Pin::new(self.sink_mut()).poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Pin::new(self.sink_mut()).start_send(item)),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => {
                self.buffered = Some(item);
                Poll::Pending
            }
        }
    }
}

impl<Si, St> Future for StartSendAll<Si, St>
where
    Si: Sink<St::Item> + Unpin,
    St: Stream + Unpin,
{
    type Output = Result<(Si, St), Si::SinkError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;

        // If we've got an item buffered already, we need to write it to the
        // sink before we can do anything else
        if let Some(item) = this.buffered.take() {
            ready!(this.try_start_send(cx, item))?
        }

        loop {
            match Pin::new(this.stream_mut()).poll_next(cx) {
                Poll::Ready(Some(item)) => try_ready!(this.try_start_send(cx, item)),
                Poll::Ready(None) => return Poll::Ready(Ok(this.take_result())),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
