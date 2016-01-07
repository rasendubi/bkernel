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
        while let Some(task) = self.tasks.pop_front() {
            (task.function)();
        }
    }

    pub fn add_task(&mut self, _priority: u32, task: Task<'a>) {
        self.tasks.push_back(task);
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

    #[test]
    fn dont_call_schedule() {
        let mut task_executed = false;

        {
            let task = Task {
                name: "random",
                function: &mut || { task_executed = true; },
            };

            let mut scheduler = Scheduler::new();
            scheduler.add_task(0, task);
        }

        assert_eq!(false, task_executed);
    }

    #[test]
    fn schedule_twice() {
        let mut call_counter = 0;

        {
            let task = Task {
                name: "random",
                function: &mut || { call_counter += 1; },
            };

            let mut scheduler = Scheduler::new();
            scheduler.add_task(0, task);
            scheduler.schedule();
            scheduler.schedule();
        }

        assert_eq!(1, call_counter);
    }

    #[test]
    fn multiple_tasks() {
        let mut task2_executed = false;
        let mut task1_executed = false;

        {
            // Not sure why I have to create this closure before task1
            let t2 = &mut || { task2_executed = true; };

            let task1 = Task {
                name: "task1",
                function: &mut || { task1_executed = true; },
            };
            let task2 = Task {
                name: "task2",
                function: t2,
            };

            let mut scheduler = Scheduler::new();
            scheduler.add_task(0, task1);
            scheduler.add_task(0, task2);
            scheduler.schedule();
        }

        assert_eq!(true, task1_executed);
        assert_eq!(true, task2_executed);
    }

    // add task from within another task
    // priorities
    // priority boost? (priority inversion)
    // task preemption?
    // locks?
}
