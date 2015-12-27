//! This module is the task scheduler for bkernel.
//!
//! The bkernel is fully asynchronous and doesn't allow blocking
//! operations *at all*.
//!
//! The bkernel doesn't have threads. Instead it has **tasks**. Tasks
//! should be small as they can't be preempted with the tasks of the
//! same or lower priority. Preemption with tasks of higher priority
//! is in the plans but not implemented yet, as it requires some
//! thoughts on proper resource management (bkernel won't have mutexes
//! as they're blocking).

#![feature(collections)]
#![no_std]

extern crate collections;

use ::collections::vec_deque::VecDeque;

pub struct Scheduler<'a> {
    #[allow(dead_code)]
    tasks: VecDeque<Task<'a>>,
}

struct Task<'a> {
    #[allow(dead_code)]
    name: &'a str,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Scheduler<'a> {
        Scheduler {
            tasks: VecDeque::new()
        }
    }

    pub fn schedule(&mut self) {

    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn has_new() {
        let _scheduler = Scheduler::new();
    }

    #[test]
    fn schedule_empty() {
        let mut scheduler = Scheduler::new();
        scheduler.schedule();
    }
}
