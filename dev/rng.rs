use ::core::sync::atomic::{AtomicU32, Ordering};

use stm32f4::rng;
use stm32f4::IrqLock;

use breactor::REACTOR;

use futures::{Async, Poll, Stream};

pub static mut RNG: Rng = Rng {
    inner: unsafe{&rng::RNG},
    task: AtomicU32::new(0),
};

#[allow(missing_debug_implementations)]
pub struct Rng<'a> {
    inner: &'a rng::Rng,
    task: AtomicU32,
}

impl<'a> Rng<'a> {
    pub fn enable(&self) {
        self.inner.enable();
    }

    pub fn disable(&self) {
        self.inner.disable();
    }
}

impl<'a> Stream for Rng<'a> {
    type Item = u32;
    type Error = rng::RngError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let task = REACTOR.get_current_task_mask();

        self.task.fetch_or(task, Ordering::SeqCst);

        // TODO(rasen): disable RNG interrupt only?
        let _lock = unsafe { IrqLock::new() };
        match self.inner.get() {
            Ok(Some(x)) => {
                self.task.fetch_and(!task, Ordering::SeqCst);
                Ok(Async::Ready(Some(x)))
            },
            Err(err) => {
                self.task.fetch_and(!task, Ordering::SeqCst);
                Err(err)
            },
            Ok(None) => {
                self.inner.it_enable();
                Ok(Async::NotReady)
            },
        }
    }
}

#[no_mangle]
pub unsafe fn __isr_hash_rng() {
    let task = RNG.task.swap(0, Ordering::SeqCst);
    REACTOR.set_ready_task_mask(task);
    RNG.inner.it_disable();
}
