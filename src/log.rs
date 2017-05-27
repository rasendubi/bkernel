//! Logging.

use ::dev::usart::Usart;

use ::core::array::FixedSizeArray;

#[allow(missing_debug_implementations)]
pub struct Logger<'a, A: FixedSizeArray<u8> + 'a, B: FixedSizeArray<u8> + 'a> {
    inner: &'a Usart<A, B>,
}

impl<'a, A: FixedSizeArray<u8> + 'a, B: FixedSizeArray<u8> + 'a> Logger<'a, A, B> {
    pub const fn new(usart: &Usart<A, B>) -> Logger<A, B> {
        Logger {
            inner: usart,
        }
    }
}

/// This is very bad implementation for several reasons:
///
/// 1. It fails when the buffer is full, printing only the first part
/// of the string.
///
/// 2. It requires getting a mutable reference to the buffer, which is
/// not safe.
impl<'a, A: FixedSizeArray<u8>, B: FixedSizeArray<u8>> ::core::fmt::Write for Logger<'a, A, B> {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for b in s.as_bytes() {
            if !self.inner.try_push_writer(*b) {
                return Err(::core::fmt::Error);
            }
        }

        Ok(())
    }
}
