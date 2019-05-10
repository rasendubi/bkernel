//! Volatile wrappers.
//!
//! This module provides a wrapper around `volatile_load` and
//! `volatile_store`, so user shouldn't use compiler intrinsics
//! directly.

use core::intrinsics::{volatile_load, volatile_store};

use core::fmt::{Debug, Error, Formatter};

use core::ops::{BitAnd, BitOr, Not};

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
    /// Cast-constructor for `Volatile`. It creates a volatile
    /// variable implicitly casting from `usize`, so you don't have to
    /// cast yourself.
    ///
    /// # Example
    /// ```
    /// # use stm32f4::volatile::Volatile;
    /// assert_eq!(Volatile(0x40020100 as *mut u32), Volatile::new(0x40020100));
    /// ```
    pub fn new(addr: usize) -> Volatile<T> {
        Volatile(addr as *mut T)
    }

    /// Use instead of `volatile_store`.
    pub unsafe fn set(&self, value: T) {
        volatile_store(self.0, value)
    }

    /// Use instead of `volatile_load`.
    pub unsafe fn get(&self) -> T {
        volatile_load(self.0)
    }
}

/// Read-only register
#[repr(C)]
#[allow(missing_debug_implementations)]
pub struct RO<T>(T);

impl<T> RO<T> {
    /// Volatile read
    pub unsafe fn get(&self) -> T {
        volatile_load(&self.0)
    }
}

/// Write-only register
#[repr(C)]
#[allow(missing_debug_implementations)]
pub struct WO<T>(T);

impl<T> WO<T> {
    /// Volatile store
    pub unsafe fn set(&self, value: T) {
        volatile_store(&self.0 as *const T as *mut T, value)
    }
}

/// Read-write register
#[repr(C)]
#[allow(missing_debug_implementations)]
pub struct RW<T>(T);

impl<T> RW<T> {
    /// Volatile read
    pub unsafe fn get(&self) -> T {
        volatile_load(&self.0)
    }

    /// Volatile store
    pub unsafe fn set(&self, value: T) {
        volatile_store(&self.0 as *const T as *mut T, value)
    }

    /// Updates value of a register
    ///
    /// # Examples
    /// ```
    /// # use stm32f4::volatile::RW;
    /// # unsafe {
    /// let reg: RW<u32> = std::mem::uninitialized();
    /// reg.set(0x2e);
    /// reg.update(|x| {
    ///     assert_eq!(0x2e, x);
    ///     0x3f
    /// });
    /// assert_eq!(0x3f, reg.get());
    /// # }
    /// ```
    pub unsafe fn update<F>(&self, f: F)
    where
        F: FnOnce(T) -> T,
    {
        self.set(f(self.get()))
    }

    /// Performs read-modify-write and updates part of register under
    /// `mask`.
    ///
    /// # Examples
    /// ```
    /// # use stm32f4::volatile::RW;
    /// # unsafe {
    /// let reg: RW<u32> = std::mem::uninitialized();
    /// reg.set(0xdeadbabe);
    /// reg.update_with_mask(0xffff0000, 0xcafe0000);
    /// assert_eq!(0xcafebabe, reg.get());
    /// # }
    /// ```
    pub unsafe fn update_with_mask(&self, mask: T, value: T)
    where
        T: Not<Output = T> + BitAnd<T, Output = T> + BitOr<T, Output = T>,
    {
        self.update(|x| x & !mask | value);
    }

    /// Sets flag in the register.
    ///
    /// # Examples
    /// ```
    /// # use stm32f4::volatile::RW;
    /// # unsafe {
    /// let reg: RW<u32> = std::mem::uninitialized();
    /// reg.set(0x2e);
    /// reg.set_flag(0x11);
    /// assert_eq!(0x3f, reg.get());
    /// # }
    /// ```
    pub unsafe fn set_flag(&self, value: T)
    where
        T: BitOr<T, Output = T>,
    {
        self.update(|x| x | value);
    }

    /// Clears flag in the register.
    ///
    /// # Examples
    /// ```
    /// # use stm32f4::volatile::RW;
    /// # unsafe {
    /// let reg: RW<u32> = std::mem::uninitialized();
    /// reg.set(0x3f);
    /// reg.clear_flag(0x11);
    /// assert_eq!(0x2e, reg.get());
    /// # }
    /// ```
    pub unsafe fn clear_flag(&self, value: T)
    where
        T: Not<Output = T> + BitAnd<T, Output = T>,
    {
        self.update(|x| x & !value);
    }
}

/// Reserved register.
///
/// There is no operations defined and the structure is hidden, so
/// there is nothing you can do with reserved register - it's reserved
/// after all.
#[repr(C)]
#[allow(missing_debug_implementations)]
pub struct RES<T>(T);

/// Define a set of registers with a shorter syntax.
///
/// # Examples
/// ```
/// # #[macro_use] extern crate stm32f4;
/// # use stm32f4::volatile::Volatile;
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
/// # #[macro_use] extern crate stm32f4;
/// # use stm32f4::volatile::Volatile;
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
/// # use stm32f4::volatile::Volatile;
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
