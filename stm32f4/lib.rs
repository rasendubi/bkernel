//! STM32F4xx drivers.
#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(asm)]

#![no_std]

pub mod isr_vector;

#[macro_use]
pub mod volatile;
pub mod rcc;
pub mod gpio;
pub mod usart;
pub mod timer;
pub mod nvic;
pub mod i2c;
pub mod crc;
pub mod rng;

pub mod lang_items;

#[inline(always)]
#[cfg(not(target_arch = "arm"))]
pub unsafe fn __wait_for_interrupt() {
    panic!("__wait_for_interrupt is not implemented");
}

#[inline(always)]
#[cfg(target_arch = "arm")]
pub unsafe fn __wait_for_interrupt() {
    asm!("wfi" : : : : "volatile");
}

#[inline(always)]
#[cfg(not(target_arch = "arm"))]
pub unsafe fn __wait_for_event() {
    panic!("__wait_for_event is not implemented");
}

#[inline(always)]
#[cfg(target_arch = "arm")]
pub unsafe fn __wait_for_event() {
    asm!("wfe" : : : : "volatile");
}

#[inline(always)]
#[cfg(not(target_arch = "arm"))]
pub unsafe fn __set_event() {
    panic!("__set_event is not implemented");
}

#[inline(always)]
#[cfg(target_arch = "arm")]
pub unsafe fn __set_event() {
    asm!("sev" : : : : "volatile");
}

#[inline(always)]
#[cfg(not(target_arch = "arm"))]
pub unsafe fn __enable_irq() {
    panic!("enable_irq is not implemented");
}

#[inline(always)]
#[cfg(not(target_arch = "arm"))]
pub unsafe fn __disable_irq() {
    panic!("disable_irq is not implemented");
}

/// Get priority mask.
#[inline(always)]
#[cfg(not(target_arch = "arm"))]
pub unsafe fn __get_primask() -> u32{
    panic!("get_primask is not implemented");
}

#[inline(always)]
#[cfg(target_arch = "arm")]
pub unsafe fn __enable_irq() {
    asm!("cpsie i" : : : : "volatile");
}

#[inline(always)]
#[cfg(target_arch = "arm")]
pub unsafe fn __disable_irq() {
    asm!("cpsid i" : : : : "volatile");
}

#[inline(always)]
#[cfg(target_arch = "arm")]
pub unsafe fn __get_primask() -> u32 {
    let result: u32;
    asm!("MRS $0, primask" : "=r" (result) : : : "volatile");
    result
}

/// Saves current irq status and disables interrupts.
/// Interrupts should always be restored with `restore_irq()`.
///
/// # Examples
///
/// ```no_run
/// # use stm32f4::{save_irq, restore_irq};
/// # unsafe {
/// let irq = save_irq();
/// // Do work with interrupts disabled
/// restore_irq(irq);
/// # }
/// ```
#[inline(always)]
pub unsafe fn save_irq() -> u32 {
    let primask = __get_primask();
    __disable_irq();
    primask
}

/// Enables interrupts if primask is non-zero.
///
/// Must be used in pair with `save_irq()`.
#[inline(always)]
pub unsafe fn restore_irq(primask: u32) {
    if primask == 0 {
        __enable_irq();
    }
}

/// A convenience wrapper around `save_irq` and `restore_irq`.
#[derive(Debug)]
pub struct IrqLock(u32);

impl IrqLock {
    pub unsafe fn new() -> IrqLock {
        IrqLock(save_irq())
    }
}

impl Drop for IrqLock {
    fn drop(&mut self) {
        unsafe { restore_irq(self.0); }
    }
}

/// Returns the unique device identifier.
///
/// The 96-bit unique device identifier provides a reference number
/// which is unique for any device and in any context.
pub fn get_device_id() -> u128 {
    const REG: *const u32 = 0x1FFF_7A10 as _;
    unsafe {
        (*REG as u128) | ((*REG.offset(1) as u128) << 32) | ((*REG.offset(2) as u128) << 64)
    }
}

/// Returns the flash memory size in kbytes.
pub fn get_flash_size() -> u16 {
    const REG: *const u16 = 0x1FFF_7A22 as _;
    unsafe {
        *REG
    }
}
