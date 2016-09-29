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

#![feature(const_fn)]
#![no_std]

#[cfg(test)]
extern crate std;
#[macro_use]
#[cfg(test)]
extern crate lazy_static;

use ::core::cell::Cell;

pub struct Scheduler<'a> {
    tasks: Cell<*mut Task<'a>>,
    current_priority: Cell<u32>,
}

pub struct Task<'a> {
    #[allow(dead_code)]
    name: &'a str,
    priority: u32,
    f: unsafe fn(*const ()),
    arg: *const (),
    next: *mut Task<'a>,
}

impl<'a> Task<'a> {
    #[inline(always)]
    pub const unsafe fn from_fnonce<T: FnOnce() + 'a>(name: &'a str, priority: u32, f: *const T) -> Task<'a> {
        unsafe fn call_once_ptr<T: FnOnce()>(p: *const ()) {
            ::core::ptr::read(::core::mem::transmute::<_, *mut T>(p))();
        }

        Task {
            name: name,
            priority: priority,
            f: call_once_ptr::<T>,
            arg: f as *const (),
            next: 0 as *mut _,
        }
    }

    #[inline(always)]
    pub const unsafe fn from_fn<T: Fn()>(name: &'a str, priority: u32, f: &'a T) -> Task<'a> {
        unsafe fn call_fn_ptr<T: Fn()>(p: *const ()) {
            (*(p as *const T))();
        }

        Task {
            name: name,
            priority: priority,
            f: call_fn_ptr::<T>,
            arg: f as *const T as *const (),
            next: 0 as *mut _,
        }
    }

    #[inline(always)]
    pub const unsafe fn from_fnmut<T: FnMut()>(name: &'a str, priority: u32, f: &'a mut T) -> Task<'a> {
        unsafe fn call_fn_mut_ptr<T: FnMut()>(p: *const ()) {
            (*(p as *mut T))();
        }

        Task {
            name: name,
            priority: priority,
            f: call_fn_mut_ptr::<T>,
            arg: f as *const T as *const (),
            next: 0 as *mut _,
        }
    }

    #[inline(always)]
    pub const fn from_raw(name: &'a str, priority: u32, f: unsafe fn(*const ()), arg: *const ()) -> Task<'a> {
        Task {
            name: name,
            priority: priority,
            f: f,
            arg: arg,
            next: 0 as *mut _,
        }
    }

    #[inline(always)]
    pub const fn from_safe(name: &'a str, priority: u32, f: fn(*const ()), arg: *const ()) -> Task<'a> {
        Task::from_raw(name, priority, f, arg)
    }

    unsafe fn call(&self) {
        (self.f)(self.arg);
    }
}

impl<'a> Scheduler<'a> {
    pub const fn new() -> Scheduler<'a> {
        Scheduler {
            tasks: Cell::new(0 as *mut _),
            current_priority: Cell::new(u32::max_value()),
        }
    }

    fn next_task(&self) -> Option<&Task<'a>> {
        let task = self.tasks.get();
        if task.is_null() {
            None
        } else {
            Some(unsafe { &*task })
        }
    }

    fn pop_task(&self) -> Option<&mut Task<'a>> {
        let task = self.tasks.get();
        if task.is_null() {
            None
        } else {
            unsafe {
                self.tasks.set((*task).next);
                Some(&mut *task)
            }
        }
    }

    pub unsafe fn schedule(&self) {
        while let Some(task) = self.pop_task() {
            task.call();
        }
    }

    /// Try to preempt current task with new one.
    /// Returns true if task was executed.
    ///
    /// This function should be called with interrupts disabled.
    pub unsafe fn reschedule(&self) -> bool {
        if let Some(task) = self.next_task() {
            if task.priority < self.current_priority.get() {
                match self.pop_task() {
                    Some(task) => {
                        let priority = self.current_priority.get();
                        self.current_priority.set(task.priority);
                        task.call();
                        self.current_priority.set(priority);
                        return true;
                    },
                    _ => panic!("Something went wrong!"),
                }
            }
        }
        false
    }

    pub unsafe fn add_task(&self, task: *mut Task<'a>) {
        let head = self.tasks.get();
        if head.is_null() || (*head).priority > (*task).priority {
            self.tasks.set(task);
            return;
        }

        let mut cur = head;
        while !(*cur).next.is_null() && (*(*cur).next).priority <= (*task).priority {
            cur = (*cur).next;
        }
        (*task).next = (*cur).next;
        (*cur).next = task;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ::std::rc::Rc;
    use ::core::cell::Cell;

    mod scheduler {
        use super::super::*;
        use ::core::cell::UnsafeCell;

        use ::std::sync::{Mutex, MutexGuard};

        lazy_static! {
            pub static ref LOCK: Mutex<()> = Mutex::new(());
        }

        struct SyncCell(UnsafeCell<Option<Scheduler<'static>>>);
        unsafe impl Sync for SyncCell { }

        static SCHEDULER: SyncCell = SyncCell(UnsafeCell::new(None));

        #[must_use]
        pub fn test_init() -> MutexGuard<'static,()> {
            let lock = LOCK.lock().unwrap();
            unsafe {
                *SCHEDULER.0.get() = Some(Scheduler::new());
            }
            lock
        }

        pub fn schedule() {
            unsafe {
                (*SCHEDULER.0.get()).as_mut().unwrap().schedule();
            }
        }

        pub fn add_task(task: *mut Task<'static>) {
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
        unsafe {
            let scheduler = Scheduler::new();
            scheduler.schedule();
        }
    }

    #[test]
    fn add_task() {
        let task_executed = Rc::new(Cell::new(false));

        let te = task_executed.clone();
        let f = move || { te.set(true) };
        let mut task = unsafe { Task::from_fn(
            "random",
            0,
            &f,
        ) };

        unsafe {
            let scheduler = Scheduler::new();
            scheduler.add_task(&mut task);
            scheduler.schedule();
        }

        assert_eq!(true, task_executed.get());
    }

    #[test]
    fn dont_call_schedule() {
        let task_executed = Rc::new(Cell::new(false));

        let te = task_executed.clone();
        let f = move || { te.set(true) };
        let mut task = unsafe { Task::from_fn(
            "random",
            0,
            &f,
        ) };

        unsafe {
            let scheduler = Scheduler::new();
            scheduler.add_task(&mut task);
        }

        assert_eq!(false, task_executed.get());
    }

    #[test]
    fn schedule_twice() {
        let call_counter = Rc::new(Cell::new(0));

        let cc = call_counter.clone();
        let f = move || { cc.set(cc.get() + 1); };
        let mut task = unsafe { Task::from_fn(
            "random",
            0,
            &f,
        ) };

        unsafe {
            let scheduler = Scheduler::new();
            scheduler.add_task(&mut task);
            scheduler.schedule();
            scheduler.schedule();
        }

        assert_eq!(1, call_counter.get());
    }

    #[test]
    fn multiple_tasks() {
        let task1_executed = Rc::new(Cell::new(false));
        let task2_executed = Rc::new(Cell::new(false));

        let t1e = task1_executed.clone();
        let t2e = task2_executed.clone();

        let f1 = move || { t1e.set(true); };
        let f2 = move || { t2e.set(true); };
        let mut task1 = unsafe { Task::from_fn(
            "task1",
            0,
            &f1,
        ) };
        let mut task2 = unsafe { Task::from_fn(
            "task2",
            0,
            &f2,
        ) };

        unsafe {
            let scheduler = Scheduler::new();
            scheduler.add_task(&mut task1);
            scheduler.add_task(&mut task2);
            scheduler.schedule();
        }

        assert_eq!(true, task1_executed.get());
        assert_eq!(true, task2_executed.get());
    }

    #[test]
    fn add_task_from_task() {
        let _lock = scheduler::test_init();

        let task1_executed = Rc::new(Cell::new(false));
        let task2_executed = Rc::new(Cell::new(false));

        let t1e = task1_executed.clone();
        let t2e = task2_executed.clone();

        let f2 = move || {
            t2e.set(true);
        };
        let mut task2 = unsafe { Task::from_fnonce(
            "task2",
            0,
            &f2,
        ) };

        let f1 = move || {
            t1e.set(true);
            scheduler::add_task(&mut task2);
        };
        let mut task1 = unsafe { Task::from_fnonce(
            "task1",
            0,
            &f1,
        ) };
        scheduler::add_task(&mut task1);
        scheduler::schedule();

        assert_eq!(true, task1_executed.get());
        assert_eq!(true, task2_executed.get());
        ::core::mem::forget(f1);
        ::core::mem::forget(f2);
    }

    #[test]
    fn add_task_from_task_multiple() {
        for _ in 1..10000 {
            add_task_from_task();
        }
    }

    #[test]
    fn priorities() {
        let _lock = scheduler::test_init();

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

        let f1 = move || {
            assert_eq!(false, t11.get());
            assert_eq!(false, t12.get());
            assert_eq!(false, t13.get());
            t11.set(true);
        };
        let f2 = move || {
            assert_eq!(true, t21.get());
            assert_eq!(false, t22.get());
            assert_eq!(true, t23.get());
            t22.set(true);
        };
        let f3 = move || {
            assert_eq!(true, t31.get());
            assert_eq!(false, t32.get());
            assert_eq!(false, t33.get());
            t33.set(true);
        };

        let mut task1 = unsafe { Task::from_fnonce(
            "task1",
            0,
            &f1,
        ) };
        let mut task2 = unsafe { Task::from_fnonce(
            "task2",
            3,
            &f2,
        ) };
        let mut task3 = unsafe { Task::from_fnonce(
            "task3",
            2,
            &f3,
        ) };

        scheduler::add_task(&mut task1);
        scheduler::add_task(&mut task2);
        scheduler::add_task(&mut task3);
        scheduler::schedule();

        assert_eq!(true, task1_executed.get());
        assert_eq!(true, task2_executed.get());
        assert_eq!(true, task3_executed.get());

        ::core::mem::forget(f1);
        ::core::mem::forget(f2);
        ::core::mem::forget(f3);
    }

    // priority boost? (priority inversion)
    // task preemption?
    // locks?
}
