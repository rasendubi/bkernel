use core::intrinsics::{volatile_load, volatile_store};

pub struct Volatile<T>(pub *mut T);

impl <T> Volatile<T> {
    fn addr(&self) -> *mut T {
        let Volatile(addr) = *self;
        addr
    }

    pub unsafe fn set(&self, value: T) {
        volatile_store(self.addr(), value)
    }

    pub unsafe fn get(&self) -> T {
        volatile_load(self.addr())
    }
}

#[macro_export]
macro_rules! registers {
    ( $($v:ident : $t:ty = $e:expr);* ; ) => (
        $(
            const $v: ::volatile::Volatile<$t> = ::volatile::Volatile($e as *mut $t);
        )*
    )
}
