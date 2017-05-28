//! ESP8266 AT command based driver.
use ::core::array::FixedSizeArray;
use ::core::marker::PhantomData;

use ::futures::{Async, Future, Poll, Stream};

use ::breactor::start_send_all_string::StartSendAllString;

use usart::Usart;

macro_rules! debug_log {
    ( $( $x:expr ),* ) => {
        {
            use ::core::fmt::Write;
            let _lock = unsafe { ::stm32f4::IrqLock::new() };

            let _ = write!(unsafe{&::stm32f4::usart::USART2}, $($x),*);
        }
    };
}

#[allow(missing_debug_implementations)]
pub struct Esp8266<'a, A, B>
    where A: FixedSizeArray<u8> + 'a,
          B: FixedSizeArray<u8> + 'a,
{
    usart: &'a Usart<A, B>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Error {
    Generic,
}

impl<'a, A, B> Esp8266<'a, A, B>
    where A: FixedSizeArray<u8> + 'a,
          B: FixedSizeArray<u8> + 'a,
{
    /// Creates new ESP instance from a USART.
    ///
    /// # Examples
    /// ```no_run
    /// # #![feature(const_fn)]
    /// # extern crate dev;
    /// # extern crate stm32f4;
    /// # fn main() {
    /// # use ::dev::esp8266::Esp8266;
    /// # use ::dev::usart::Usart;
    /// static USART3: Usart<[u8; 32], [u8; 32]> =
    ///     Usart::new(unsafe{&::stm32f4::usart::USART3}, [0; 32], [0; 32]);
    ///
    /// let esp = Esp8266::new(&USART3);
    /// # }
    /// ```
    pub const fn new(usart: &'a Usart<A, B>) -> Esp8266<'a, A, B> {
        Esp8266 {
            usart,
        }
    }

    /// Check if the USART is connected to ESP8266 (actually, anything
    /// that accepts AT commands).
    ///
    /// # Examples
    /// ```no_run
    /// # #![feature(const_fn)]
    /// # extern crate futures;
    /// # extern crate dev;
    /// # extern crate stm32f4;
    /// # fn main() {
    /// # use ::dev::esp8266::Esp8266;
    /// # use ::dev::usart::Usart;
    /// # use ::futures::{Async, Future};
    /// static USART3: Usart<[u8; 32], [u8; 32]> =
    ///     Usart::new(unsafe{&::stm32f4::usart::USART3}, [0; 32], [0; 32]);
    ///
    /// let mut esp = Esp8266::new(&USART3);
    /// assert_eq!(Ok(Async::Ready(true)), esp.check_at().poll());
    /// # }
    /// ```
    pub fn check_at(&'a mut self) -> impl Future<Item=bool, Error=Error> + 'a {
        // TODO(rasen): make const fn alternative to future::lazy
        ::futures::future::lazy(move || {
            while let Some(_) = self.usart.try_pop_reader() {
            }

            Ok(self.usart)
        })
            .and_then(|usart| {
                StartSendAllString::new(usart, "AT\r\n")
            })
            .then(|res| {
                match res {
                    Ok(usart) => {
                        TakeUntil::new([0; 32], usart, [
                            b"OK\r\n" as &[u8],
                            b"ERROR\r\n" as &[u8],
                        ])
                    },
                    Err(_err) => {
                        unsafe {
                            // Usart sink never errors
                            ::core::intrinsics::unreachable();
                        }
                    },
                }
            })
            .and_then(|(_buffer, _size, _m, _usart)| {
                // If any pattern matched, the other side understands
                // AT commands.
                Ok(true)
            })
            .map_err(|_err| {
                Error::Generic
            })
    }
}

#[allow(missing_debug_implementations)]
struct TakeUntil<'a, A, S, M> {
    buffer: A,
    stream: Option<S>,
    matches: M,
    cur: usize,
    __phantom: PhantomData<&'a u8>,
}

impl<'a, A, S, M> TakeUntil<'a, A, S, M>
    where A: FixedSizeArray<u8>,
          S: Stream<Item=u8>,
          M: FixedSizeArray<&'static [u8]>,
{
    pub fn new(buffer: A, stream: S, matches: M) -> TakeUntil<'a, A, S, M> {
        TakeUntil {
            buffer,
            stream: Some(stream),
            matches,
            cur: 0,
            __phantom: PhantomData,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum TakeUntilError<S, E> {
    /// The stream has finished.
    Finished(S),

    /// Stream has errored while polling.
    StreamError(S, E),

    /// Provided buffer is too small.
    BufferOverflow(S),
}

impl<'a, A, S, M> Future for TakeUntil<'a, A, S, M>
    where A: FixedSizeArray<u8> + Clone,
          S: Stream<Item=u8>,
          M: FixedSizeArray<&'static [u8]>,
{
    type Item = (A, usize, &'static [u8], S);
    type Error = TakeUntilError<S, S::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if self.cur >= self.buffer.as_slice().len() {
                return Err(TakeUntilError::BufferOverflow(
                    self.stream.take().unwrap()));
            }

            match self.stream.as_mut().take().unwrap().poll() {
                Ok(Async::Ready(Some(c))) => {
                    self.buffer.as_mut_slice()[self.cur] = c;
                    self.cur += 1;

                    for m in self.matches.as_slice() {
                        if self.buffer.as_slice()[.. self.cur].ends_with(m) {
                            return Ok(Async::Ready((
                                self.buffer.clone(),
                                self.cur,
                                m,
                                self.stream.take().unwrap())));
                        }
                    }
                },

                Ok(Async::Ready(None)) => {
                    return Err(TakeUntilError::Finished(
                        self.stream.take().unwrap()));
                },

                Ok(Async::NotReady) => {
                    return Ok(Async::NotReady);
                },

                Err(err) => {
                    return Err(TakeUntilError::StreamError(
                        self.stream.take().unwrap(),
                        err));
                },
            }
        }
    }
}
