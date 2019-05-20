//! ESP8266 AT command based driver.
use core::array::FixedSizeArray;
use core::marker::PhantomData;
use core::pin::Pin;
use core::str::FromStr;
use core::task::Context;

use futures::{Future, Poll, Sink, Stream, TryFutureExt};

use breactor::start_send_all_string::StartSendAllString;

#[allow(unused)]
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
pub struct Esp8266<Channel: Stream<Item = u8> + Sink<u8>> {
    usart: Channel,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Error {
    /// Generic error.
    Generic,
    /// Usart stream has finished.
    ///
    /// Practically, should never happen.
    UsartFinished,
    /// Usart stream has errored.
    UsartError,
    /// Internal buffer is too small to contain all ESP8266 output.
    BufferOverflow,
}

impl<S, E> From<TakeUntilError<S, E>> for Error {
    fn from(err: TakeUntilError<S, E>) -> Error {
        match err {
            TakeUntilError::Finished(_) => Error::UsartFinished,
            TakeUntilError::StreamError(_, _) => Error::UsartError,
            TakeUntilError::BufferOverflow(_) => Error::BufferOverflow,
        }
    }
}

impl<Channel: Stream<Item = u8> + Sink<u8> + Unpin> Esp8266<Channel> {
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
    pub const fn new(usart: Channel) -> Esp8266<Channel> {
        Esp8266 { usart }
    }

    /// Check if the USART is connected to ESP8266 (actually, anything
    /// that accepts AT commands).
    ///
    /// # Examples
    /// ```no_run
    /// # #![feature(const_fn)]
    /// # #![feature(async_await)]
    /// # #![feature(await_macro)]
    /// # extern crate futures;
    /// # extern crate dev;
    /// # extern crate stm32f4;
    /// # fn main() {
    /// # use dev::esp8266::Esp8266;
    /// # use dev::usart::Usart;
    /// # use futures::{Poll, Future};
    /// # async {
    /// static USART3: Usart<[u8; 32], [u8; 32]> =
    ///     Usart::new(unsafe{&::stm32f4::usart::USART3}, [0; 32], [0; 32]);
    ///
    /// let mut esp = Esp8266::new(&USART3);
    /// assert_eq!(Ok(true), await!(esp.check_at()));
    ///
    /// # };
    /// # }
    /// ```
    pub fn check_at<'a>(&'a mut self) -> impl Future<Output = Result<bool, Error>> + 'a {
        StartSendAllString::new(&mut self.usart, "AT\r\n")
            .map_err(|_err| Error::Generic)
            .and_then(|usart| {
                TakeUntil::new([0; 32], usart, [b"OK\r\n" as &[u8], b"ERROR\r\n" as &[u8]])
                    .map_err(|_err| Error::Generic)
            })
            .map_ok(|(_buffer, _size, _m, _usart)| {
                // If any pattern matched, the other side understands
                // AT commands.
                true
            })
            .map_err(|_err| Error::Generic)
    }

    /// List available access points.
    ///
    /// The resulting future returns a fixed-size array along with the
    /// actual number of access points returned from ESP8266. Note
    /// that the number may be higher than array requested.
    ///
    /// # Examples
    /// List up to 32 access points.
    ///
    /// ```no_run
    /// # #![feature(const_fn)]
    /// # extern crate futures;
    /// # extern crate dev;
    /// # extern crate stm32f4;
    /// # fn main() {
    /// # use dev::esp8266::{Esp8266, AccessPoint};
    /// # use dev::usart::Usart;
    /// # use futures::{Future, FutureExt, TryFutureExt};
    /// static USART3: Usart<[u8; 32], [u8; 32]> =
    ///     Usart::new(unsafe{&::stm32f4::usart::USART3}, [0; 32], [0; 32]);
    ///
    /// let mut esp = Esp8266::new(&USART3);
    /// let mut aps = esp.list_aps::<[AccessPoint; 32]>()
    ///     .and_then(|(aps, size)| {
    ///         println!("Access points (total {}):", size);
    ///         for i in 0 .. std::cmp::min(size, aps.len()) {
    ///             println!("{:?}", aps[i]);
    ///         }
    ///         futures::future::ready(Ok(()))
    ///     });
    /// # }
    /// ```
    // TODO(rasen): return Stream<Item=AccessPoint> to leverage
    // incremental processing. This way, we can decrease buffer size.
    pub fn list_aps<'a, R>(&'a mut self) -> impl Future<Output = Result<(R, usize), Error>> + 'a
    where
        R: FixedSizeArray<AccessPoint> + 'a,
    {
        StartSendAllString::new(&mut self.usart, "AT+CWLAP\r\n")
            .map_err(|_| Error::Generic)
            .and_then(|usart| {
                TakeUntil::new([0; 32], usart, [b"\r\r\n" as &[u8]]).map_err(From::from)
            })
            .and_then(|(_buffer, _size, _m, usart)| {
                TakeUntil::new(
                    [0; 2048],
                    usart,
                    [b"\r\n\r\nOK\r\n" as &[u8], b"\r\n\r\nERROR\r\n"],
                )
                .map_err(From::from)
            })
            .map_ok(move |(buffer, size, m, _usart)| parse_ap_list::<R>(&buffer[..size - m.len()]))
    }

    pub fn join_ap<'a>(
        &'a mut self,
        ap: &'a str,
        pass: &'a str,
    ) -> impl Future<Output = Result<bool, Error>> + 'a {
        futures::future::lazy(move |_| Ok(&mut self.usart))
            .and_then(|usart| StartSendAllString::new(usart, "AT+CWJAP=\""))
            .and_then(move |usart| StartSendAllString::new(usart, ap))
            .and_then(|usart| StartSendAllString::new(usart, "\",\""))
            .and_then(move |usart| StartSendAllString::new(usart, pass))
            .and_then(|usart| StartSendAllString::new(usart, "\"\r\n"))
            .map_err(|_err| Error::Generic)
            .and_then(|usart| {
                TakeUntil::new([0; 128], usart, [b"OK\r\n" as &[u8], b"ERROR\r\n" as &[u8]])
                    .map_err(|_err| Error::Generic)
            })
            .map_ok(|(_buffer, _size, m, _usart)| match m {
                b"OK\r\n" => true,
                b"ERROR\r\n" => false,
                _ => unreachable!(),
            })
            .map_err(|_err| Error::Generic)
    }
}

fn parse_ap_list<A>(b: &[u8]) -> (A, usize)
where
    A: FixedSizeArray<AccessPoint>,
{
    let mut result: A = unsafe { ::core::mem::uninitialized() };
    let mut cur = 0;

    for line in unsafe { ::core::str::from_utf8_unchecked(b) }.lines() {
        if cur < result.as_slice().len() {
            result.as_mut_slice()[cur] = parse_ap(line);
        }

        cur += 1;
    }

    (result, cur)
}

// TODO(rasen): error handling
fn parse_ap(s: &str) -> AccessPoint {
    let mut s = s;
    // drop "+CWLAP:(" and final ")"
    s = &s[8..s.len() - 1];

    // TODO(rasen): comma in ESSID is not allowed
    let mut s = s.split(',');

    let ecn = i32::from_str(s.next().unwrap_or("")).unwrap_or(0);

    let ssid_s = s.next().unwrap_or("\"\"");
    let ssid_s = &ssid_s[1..ssid_s.len() - 1];
    let ssid_len = ssid_s.len();
    let mut ssid: [u8; 32] = unsafe { ::core::mem::zeroed() };
    (&mut ssid[..ssid_len]).clone_from_slice(&ssid_s.as_bytes());

    let rssi = i32::from_str(s.next().unwrap_or("")).unwrap_or(0);

    let mac_s = s.next().unwrap_or("\"\"");
    let mut mac_parts = mac_s[1..mac_s.len() - 1]
        .split(':')
        .map(|hex| i32::from_str_radix(hex, 16).unwrap_or(0x00) as u8);
    let mut mac: [u8; 6] = [0; 6];
    mac[0] = mac_parts.next().unwrap_or(0);
    mac[1] = mac_parts.next().unwrap_or(0);
    mac[2] = mac_parts.next().unwrap_or(0);
    mac[3] = mac_parts.next().unwrap_or(0);
    mac[4] = mac_parts.next().unwrap_or(0);
    mac[5] = mac_parts.next().unwrap_or(0);

    let ch = i32::from_str(s.next().unwrap_or("")).unwrap_or(0);

    let freq_offset = i32::from_str(s.next().unwrap_or("")).unwrap_or(0);

    let freq_calibration = i32::from_str(s.next().unwrap_or("")).unwrap_or(0);

    AccessPoint {
        ecn: unsafe { ::core::mem::transmute(ecn as u8) },
        ssid_len: ssid_len as u8,
        ssid,
        rssi,
        mac,
        ch: ch as u8,
        freq_offset,
        freq_calibration,
    }
}

/// Encryption method used by Access Point.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum EncryptionMethod {
    Open = 0,
    Wep = 1,
    WpaPsk = 2,
    Wpa2Psk = 3,
    WpaWpa2Psk = 4,
    Wpa2Enterprise = 5,
}

/// Access Point detected by ESP8266.
pub struct AccessPoint {
    /// Encryption method.
    pub ecn: EncryptionMethod,

    pub ssid_len: u8,
    /// String parameter, SSID of the AP.
    ///
    /// Only first `ssid_len` bytes are valid.
    pub ssid: [u8; 32],

    /// Signal strength.
    pub rssi: i32,

    /// MAC address of the AP.
    // TODO(rasen): Create MAC structure
    pub mac: [u8; 6],

    /// Channel.
    pub ch: u8,

    /// Frequency offset of AP; unit: KHz.
    pub freq_offset: i32,

    /// Calibration for frequency offset.
    pub freq_calibration: i32,
}

impl AccessPoint {
    /// Returns SSID as a string.
    pub fn ssid(&self) -> &str {
        unsafe { ::core::str::from_utf8_unchecked(&self.ssid[..self.ssid_len as usize]) }
    }
}

impl ::core::fmt::Debug for AccessPoint {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(
            f,
            "AccessPoint({:?}, \"{}\", {}, {:?}, {}, {}, {})",
            self.ecn,
            self.ssid(),
            self.rssi,
            // TODO(rasen): better MAC formatting
            self.mac,
            self.ch,
            self.freq_offset,
            self.freq_calibration
        )
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
where
    A: FixedSizeArray<u8>,
    S: Stream<Item = u8> + Unpin,
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
enum TakeUntilError<S, E> {
    /// The stream has finished.
    Finished(S),

    /// Stream has errored while polling.
    StreamError(S, E),

    /// Provided buffer is too small.
    BufferOverflow(S),
}

impl<'a, A, S, M> Unpin for TakeUntil<'a, A, S, M>
where
    A: FixedSizeArray<u8>,
    S: Stream<Item = u8> + Unpin,
    M: FixedSizeArray<&'static [u8]>,
{
}

impl<'a, A, S, M> Future for TakeUntil<'a, A, S, M>
where
    A: FixedSizeArray<u8>,
    S: Stream<Item = u8> + Unpin,
    M: FixedSizeArray<&'static [u8]>,
{
    type Output = Result<(A, usize, &'static [u8], S), TakeUntilError<S, ()>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        loop {
            if self.cur >= self.buffer.as_slice().len() {
                return Poll::Ready(Err(TakeUntilError::BufferOverflow(
                    self.stream.take().unwrap(),
                )));
            }

            match Pin::new(self.stream.as_mut().take().unwrap()).poll_next(cx) {
                Poll::Ready(Some(c)) => {
                    let cur = self.cur;
                    self.buffer.as_mut_slice()[cur] = c;
                    self.cur += 1;

                    for m in self.matches.as_slice() {
                        if self.buffer.as_slice()[..self.cur].ends_with(m) {
                            let mut b: A = unsafe { ::core::mem::uninitialized() };
                            b.as_mut_slice()[..self.cur]
                                .clone_from_slice(&self.buffer.as_slice()[..self.cur]);

                            return Poll::Ready(Ok((b, self.cur, m, self.stream.take().unwrap())));
                        }
                    }
                }

                Poll::Ready(None) => {
                    return Poll::Ready(Err(TakeUntilError::Finished(self.stream.take().unwrap())));
                }

                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }
}
