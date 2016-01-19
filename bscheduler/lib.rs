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

#![feature(collections, alloc, fnbox, const_fn)]
#![no_std]

extern crate alloc;
extern crate collections;

use ::alloc::boxed::{Box, FnBox};
use ::collections::vec_deque::VecDeque;

pub struct Scheduler<'a> {
    tasks: VecDeque<Task<'a>>,
}

pub struct Task<'a> {
    pub name: &'a str,
    pub priority: u32,
    pub function: Box<FnBox()>,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Scheduler<'a> {
        Scheduler {
            tasks: VecDeque::new(),
        }
    }

    pub fn schedule(&mut self) {
        while let Some(task) = self.tasks.pop_front() {
            (task.function)();
        }
    }

    pub fn add_task(&mut self, task: Task<'a>) {
        let i = self.index_to_insert(&task);
        self.tasks.insert(i, task);
    }

    fn index_to_insert(&self, task: &Task<'a>) -> usize {
        // tasks are sorted by priority
        let mut i = 0;
        let mut it = self.tasks.iter();
        while let Some(x) = it.next() {
            if x.priority > task.priority {
                break;
            }
            i += 1;
        }
        i
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ::alloc::boxed::Box;
    use ::alloc::rc::Rc;
    use ::core::cell::Cell;

    mod scheduler {
        use super::super::*;
        use ::core::cell::UnsafeCell;

        struct SyncCell(UnsafeCell<Option<Scheduler<'static>>>);
        unsafe impl Sync for SyncCell { }

        static SCHEDULER: SyncCell = SyncCell(UnsafeCell::new(None));

        pub fn init() {
            unsafe {
                *SCHEDULER.0.get() = Some(Scheduler::new());
            }
        }

        pub fn schedule() {
            unsafe {
                (*SCHEDULER.0.get()).as_mut().unwrap().schedule();
            }
        }

        pub fn add_task(task: Task<'static>) {
            unsafe {
                (*SCHEDULER.0.get()).as_mut().unwrap().add_task(task);
            }
        }
    }

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
        let task_executed = Rc::new(Cell::new(false));

        let te = task_executed.clone();
        let task = Task {
            name: "random",
            priority: 0,
            function: Box::new(move || { te.set(true); }),
        };

        let mut scheduler = Scheduler::new();
        scheduler.add_task(task);
        scheduler.schedule();

        assert_eq!(true, task_executed.get());
    }

    #[test]
    fn dont_call_schedule() {
        let task_executed = Rc::new(Cell::new(false));

        let te = task_executed.clone();
        let task = Task {
            name: "random",
            priority: 0,
            function: Box::new(move || { te.set(true); }),
        };

        let mut scheduler = Scheduler::new();
        scheduler.add_task(task);

        assert_eq!(false, task_executed.get());
    }

    #[test]
    fn schedule_twice() {
        let call_counter = Rc::new(Cell::new(0));

        let cc = call_counter.clone();
        let task = Task {
            name: "random",
            priority: 0,
            function: Box::new(move || { cc.set(cc.get() + 1); }),
        };

        let mut scheduler = Scheduler::new();
        scheduler.add_task(task);
        scheduler.schedule();
        scheduler.schedule();

        assert_eq!(1, call_counter.get());
    }

    #[test]
    fn multiple_tasks() {
        let task1_executed = Rc::new(Cell::new(false));
        let task2_executed = Rc::new(Cell::new(false));

        let t1e = task1_executed.clone();
        let t2e = task2_executed.clone();

        let task1 = Task {
            name: "task1",
            priority: 0,
            function: Box::new(move || { t1e.set(true); }),
        };
        let task2 = Task {
            name: "task2",
            priority: 0,
            function: Box::new(move || { t2e.set(true); }),
        };

        let mut scheduler = Scheduler::new();
        scheduler.add_task(task1);
        scheduler.add_task(task2);
        scheduler.schedule();

        assert_eq!(true, task1_executed.get());
        assert_eq!(true, task2_executed.get());
    }

    #[test]
    fn add_task_from_task() {
        scheduler::init();

        let task1_executed = Rc::new(Cell::new(false));
        let task2_executed = Rc::new(Cell::new(false));

        let t1e = task1_executed.clone();
        let t2e = task2_executed.clone();

        let task1 = Task {
            name: "task1",
            priority: 0,
            function: Box::new(move || {
                t1e.set(true);
                let task2 = Task {
                    name: "task2",
                    priority: 0,
                    function: Box::new(move || {
                        t2e.set(true);
                    }),
                };
                scheduler::add_task(task2);
            }),
        };
        scheduler::add_task(task1);
        scheduler::schedule();

        assert_eq!(true, task1_executed.get());
        assert_eq!(true, task2_executed.get());
    }

    #[test]
    fn priorities() {
        scheduler::init();

        let task1_executed = Rc::new(Cell::new(false));
        let task2_executed = Rc::new(Cell::new(false));
        let task3_executed = Rc::new(Cell::new(false));

        let t11 = task1_executed.clone();
        let t12 = task2_executed.clone();
        let t13 = task3_executed.clone();

        let t21 = task1_executed.clone();
        let t22 = task2_executed.clone();
        let t23 = task3_executed.clone();

        let t31 = task1_executed.clone();
        let t32 = task2_executed.clone();
        let t33 = task3_executed.clone();

        let task1 = Task {
            name: "task1",
            priority: 0,
            function: Box::new(move || {
                assert_eq!(false, t11.get());
                assert_eq!(false, t12.get());
                assert_eq!(false, t13.get());
                t11.set(true);
            }),
        };
        let task2 = Task {
            name: "task2",
            priority: 3,
            function: Box::new(move || {
                assert_eq!(true, t21.get());
                assert_eq!(false, t22.get());
                assert_eq!(true, t23.get());
                t22.set(true);
            }),
        };
        let task3 = Task {
            name: "task3",
            priority: 2,
            function: Box::new(move || {
                assert_eq!(true, t31.get());
                assert_eq!(false, t32.get());
                assert_eq!(false, t33.get());
                t33.set(true);
            }),
        };

        scheduler::add_task(task1);
        scheduler::add_task(task2);
        scheduler::add_task(task3);
        scheduler::schedule();

        assert_eq!(true, task1_executed.get());
        assert_eq!(true, task2_executed.get());
        assert_eq!(true, task3_executed.get());
    }

    // priority boost? (priority inversion)
    // task preemption?
    // locks?
}
