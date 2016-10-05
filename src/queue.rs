//! Asynchronous working queue

use ::bscheduler::Task;
use scheduler::add_task;

const N: usize = 4096;

pub struct Queue<T> {
    values: [T; N],
    handler: *mut Task<'static>,
    start: usize,
    end: usize,
    size: usize,
}

impl<T> Queue<T> where T: Copy {
    pub fn new_empty() -> Queue<T> {
        Queue {
            values: unsafe { ::core::mem::uninitialized() },
            handler: ::core::ptr::null_mut(),
            start: 0,
            end: 0,
            size: 0,
        }
    }

    pub fn new(handler: *mut Task<'static>) -> Queue<T> {
        Queue {
            values: unsafe { ::core::mem::uninitialized() },
            handler: handler,
            start: 0,
            end: 0,
            size: 0,
        }
    }

    pub fn put(&mut self, value: T) -> bool {
        if self.size != N {
            self.values[self.end] = value;
            self.size += 1;
            self.end = (self.end + 1) % N;

            if !self.handler.is_null() {
                add_task(self.handler);
            }
            true
        } else {
            false
        }
    }

    pub fn get(&mut self) -> Option<T> {
        if self.size != 0 {
            let i = self.start;
            self.start = (self.start + 1) % N;
            self.size -= 1;
            Some(self.values[i])
        } else {
            None
        }
    }
}
