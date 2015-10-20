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
    ( $base: expr => { $($v:ident : $t:ty = $e:expr),* } ) => (
        $(
            const $v: ::volatile::Volatile<$t> = ::volatile::Volatile(($base as usize + $e) as *mut $t);
        )*
    );
    ( $base: expr => { $($v:ident : $t:ty = $e:expr),* , } ) => (
        $(
            const $v: ::volatile::Volatile<$t> = ::volatile::Volatile(($base as usize + $e) as *mut $t);
        )*
    );

    ( $base: expr , $t:ty => { $($v:ident = $e:expr),* } ) => (
        $(
            const $v: ::volatile::Volatile<$t> = ::volatile::Volatile(($base as usize + $e) as *mut $t);
        )*
    );
    ( $base: expr , $t:ty => { $($v:ident = $e:expr),* , } ) => (
        $(
            const $v: ::volatile::Volatile<$t> = ::volatile::Volatile(($base as usize + $e) as *mut $t);
        )*
    );
}
