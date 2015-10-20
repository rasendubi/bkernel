//! Volatile wrappers.
//!
//! This module provides a wrapper around `volatile_load` and
//! `volatile_store`, so user shouldn't use compiler intrinsics
//! directly.

use core::intrinsics::{volatile_load, volatile_store};

use core::fmt::{Debug, Formatter, Error};

/// Represents a volatile register.
///
/// `Volatile<T>` represents a volatile register of type `T`.
/// It's analagous to C's: `volatile T *` type.
pub struct Volatile<T>(pub *mut T);

impl<T> Debug for Volatile<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Volatile({0:p})", self.0)
    }
}

impl<T> PartialEq for Volatile<T> {
    fn eq(&self, other: &Volatile<T>) -> bool {
        self.0 == other.0
    }
}

impl<T> Volatile<T> {
    /// Use instead of `volatile_store`.
    pub unsafe fn set(&self, value: T) {
        volatile_store(self.0, value)
    }

    /// Use instead of `volatile_load`.
    pub unsafe fn get(&self) -> T {
        volatile_load(self.0)
    }
}

/// Define a set of registers with a shorter syntax.
///
/// # Examples
/// ```
/// # #[macro_use] extern crate kernel;
/// # use kernel::volatile::Volatile;
/// # fn main() {
/// const RCC_BASE: usize = 0x40023800;
/// registers! {
///     RCC_BASE, u32 => {
///         RCC_CR      = 0x00,
///         RCC_PLLCFGR = 0x04,
///     }
/// }
/// assert_eq!(Volatile(0x40023800 as *mut u32), RCC_CR);
/// assert_eq!(Volatile(0x40023804 as *mut u32), RCC_PLLCFGR);
/// # }
/// ```
///
/// This also support explicit type for all registers:
///
/// ```
/// # #[macro_use] extern crate kernel;
/// # use kernel::volatile::Volatile;
/// # fn main() {
/// const USART1_BASE: usize = 0x40011000;
/// registers! {
///     USART1_BASE => {
///         USART1_SR: u32 = 0x0,
///         USART1_DR: u8  = 0x4
///     }
/// }
/// assert_eq!(Volatile(0x40011000 as *mut u32), USART1_SR);
/// assert_eq!(Volatile(0x40011004 as *mut u8), USART1_DR);
/// # }
/// ```
///
/// # Known bugs
/// It's not possible to attach a doc to a register.
///
/// The following doesn't compile:
///
/// ```ignore
/// # #[macro_use] extern crate kernel;
/// # use kernel::volatile::Volatile;
/// # fn main() {
/// const USART1_BASE: usize = 0x40011000;
/// registers! {
///     USART1_BASE => {
///         USART1_SR: u32 = 0x0,
///         /// Data register
///         USART1_DR: u8  = 0x4
///     }
/// }
/// # }
/// ```
#[macro_export]
macro_rules! registers {
    ( $base:expr => { $($v:ident : $t:ty = $e:expr),* } ) => (
        $(
            const $v: $crate::volatile::Volatile<$t> = $crate::volatile::Volatile(($base as usize + $e) as *mut $t);
        )*
    );
    ( $base:expr => { $($v:ident : $t:ty = $e:expr),* , } ) => (
        $(
            const $v: $crate::volatile::Volatile<$t> = $crate::volatile::Volatile(($base as usize + $e) as *mut $t);
        )*
    );

    ( $base:expr , $t:ty => { $($v:ident = $e:expr),* } ) => (
        $(
            const $v: $crate::volatile::Volatile<$t> = $crate::volatile::Volatile(($base as usize + $e) as *mut $t);
        )*
    );
    ( $base:expr , $t:ty => { $($v:ident = $e:expr),* , } ) => (
        $(
            const $v: $crate::volatile::Volatile<$t> = $crate::volatile::Volatile(($base as usize + $e) as *mut $t);
        )*
    );
}
