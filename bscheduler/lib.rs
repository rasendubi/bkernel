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

pub struct Task<'a> {
    #[allow(dead_code)]
    pub name: &'a str,
    pub function: &'a mut FnMut() -> (),
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Scheduler<'a> {
        Scheduler {
            tasks: VecDeque::new()
        }
    }

    pub fn schedule(&mut self) {

    }

    pub fn add_task(&mut self, _priority: i32, task: Task) {
        (task.function)();
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

    #[test]
    fn add_task() {
        let mut task_executed = false;

        {
            let task = Task {
                name: "random",
                function: &mut || { task_executed = true; },
            };

            let mut scheduler = Scheduler::new();
            scheduler.add_task(0, task);
            scheduler.schedule();
        }

        assert_eq!(true, task_executed);
    }
}
