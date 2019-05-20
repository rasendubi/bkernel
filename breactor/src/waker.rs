use core::task::{RawWaker, RawWakerVTable, Waker};

use super::REACTOR;

pub const WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake, waker_wake_by_ref, waker_drop);

unsafe fn waker_clone(data: *const ()) -> RawWaker {
    RawWaker::new(data, &WAKER_VTABLE)
}

unsafe fn waker_wake(data: *const ()) {
    let task_mask = data as u32;
    REACTOR.set_ready_task_mask(task_mask)
}

unsafe fn waker_wake_by_ref(data: *const ()) {
    let task_mask = data as u32;
    REACTOR.set_ready_task_mask(task_mask)
}

unsafe fn waker_drop(_data: *const ()) {}

pub fn new_task_waker(task_mask: u32) -> Waker {
    unsafe { Waker::from_raw(RawWaker::new(task_mask as *const (), &WAKER_VTABLE)) }
}
