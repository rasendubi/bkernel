#![no_std]
#![feature(integer_atomics)]
#![feature(const_fn)]

#![allow(dead_code, unused_imports)]

extern crate futures;

extern crate stm32f4;

use ::core::sync::atomic::{AtomicU32, Ordering};
use ::core::cell::UnsafeCell;
use ::core::u32;

use futures::{Async, Future};

// static mut REACTOR: Reactor = Reactor::new();

// Id is stored internally as a mask.
#[derive(Debug, PartialEq)]
#[derive(Clone, Copy)]
pub struct TaskId(u32);

impl TaskId {
    /// Creates new unchecked task id.
    ///
    /// The argument must be lower than 32.
    pub const unsafe fn unsafe_new(id: u32) -> TaskId {
        TaskId((1 as u32) << id)
    }

    /// Creates new checked TaskId from priority.
    ///
    /// # Return values
    /// Returns `None` if id is too high.
    /// ```
    /// assert_eq!(None, breactor::TaskId::new(32));
    /// ```
    ///
    /// On success, returns some value.
    /// ```
    /// assert!(breactor::TaskId::new(31).is_some());
    /// ```
    pub fn new(id: u32) -> Option<TaskId> {
        (1 as u32).checked_shl(id).map(TaskId)
    }

    const fn get_mask(&self) -> u32 {
        self.0
    }
}

/// A single execution task. It is an entity that drives the given
/// future to completion.
struct Task<F> {
    id: TaskId,
    future: F,
}

impl<F, T, E> Task<F>
    where F: Future<Item=T, Error=E>
{
}

/// The reactor is an entity that controls execution of multiple
/// tasks.
///
/// There could be only one reactor in the application, as it relies
/// on global values.
///
/// Each task has an ID assigned. The ID plays two roles. First, it
/// distinguishes tasks, therefore it must be unique. Second, it
/// determines the priority. Higher ids mean higher priority.
pub struct Reactor<'a> {
    // TODO(rasen): should this be atomic?
    //
    // As far as I see, this must only be read from the system thread
    // and not interrupts, so there is no concurrent access.
    //
    // On the other hand, if we're going for task preemption, a switch
    // might occur right when the value is changed (or tasks reads its
    // id), leading to inconsistencies.
    current_task_mask: AtomicU32,
    tasks: [UnsafeCell<Option<&'a mut Future<Item=(), Error=()>>>; 32],

    /// This is a bread and butter of the reactor.
    ///
    /// This variable holds 32 individual bits, each representing a
    /// readiness state of the task with id equal to the bit
    /// number. (e.g., 0x05, binary 101, means tasks with id 0 and 2
    /// are ready to run.)
    ///
    /// Such representation allows selecting the task with highest
    /// priority by counting leading zeros (which is extremely
    /// efficient operation), and setting/resetting task status
    /// atomically. This all makes this reactor lock-free.
    ready_mask: AtomicU32,
}

unsafe impl<'a> Sync for Reactor<'a> {}

impl<'a> Reactor<'a> {
    pub const fn new() -> Reactor<'a> {
        Reactor {
            current_task_mask: AtomicU32::new(0),
            // Because the trait Copy is not implemented for &mut
            // Future<Item=(), Error=()>
            tasks: [
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
                UnsafeCell::new(None), UnsafeCell::new(None),
            ],
            ready_mask: AtomicU32::new(0),
        }
    }

    /// Creates a react with a predefined set of tasks.
    pub const fn from_array(tasks: [UnsafeCell<Option<&'a mut Future<Item=(), Error=()>>>; 32]) -> Reactor<'a> {
        Reactor {
            current_task_mask: AtomicU32::new(0),
            tasks: tasks,

            // All tasks are ready.
            //
            // TODO(rasen): maybe allow user to specify the mask?
            ready_mask: AtomicU32::new(u32::MAX),
        }
    }

    /// Marks the given task as ready.
    pub fn set_task_ready(&self, id: TaskId) {
        self.ready_mask.fetch_or(id.0, Ordering::SeqCst);
        unsafe { stm32f4::__set_event() };
    }

    pub fn get_current_task_mask(&self) -> u32 {
        self.current_task_mask.load(Ordering::SeqCst)
    }

    pub fn set_ready_task_mask(&self, mask: u32) {
        if mask != 0 {
            self.ready_mask.fetch_or(mask, Ordering::SeqCst);
            unsafe { stm32f4::__set_event() };
        }
    }

    /// Returns true if any task is ready to be polled.
    pub fn is_ready(&self) -> bool {
        self.ready_mask.load(Ordering::SeqCst) != 0
    }

    /// Returns next task to run.
    fn select_next_task(&self) -> Option<u32> {
        let mask = self.ready_mask.load(Ordering::SeqCst);
        let zeros = mask.leading_zeros();
        if zeros == 32 {
            None
        } else {
            Some(31 - zeros)
        }
    }

    /// Runs until all tasks get blocked.
    ///
    /// This allows putting processor into sleep when there is no job
    /// to do.
    ///
    /// This function is unsafe because the caller must ensure that
    /// only a single thread calls run at the same time.
    pub unsafe fn run(&self) {
        while let Some(task_id) = self.select_next_task() {
            let task_mask = 1u32 << task_id;
            self.ready_mask.fetch_and(!task_mask, Ordering::SeqCst);
            self.current_task_mask.store(task_mask, Ordering::SeqCst);

            let mtask = &mut *self.tasks[task_id as usize].get();
            *mtask = match *mtask {
                Some(ref mut task) => {
                    let res = task.poll();
                    match res {
                        Ok(Async::NotReady) => {
                            continue
                        },
                        _ => {
                            // Remove task if if has finished or
                            // failed.
                            None
                        },
                    }
                },
                None => {
                    // Nothing to do
                    continue
                },
            };
        }
    }

    /// Returns true if task was successfully added.
    /// Returns false if task_id is too high or already occupied.
    ///
    /// The caller must ensure it has unique write access to the
    /// reactor.
    pub unsafe fn add_task(&self, task_id: u32, f: &'a mut Future<Item=(), Error=()>) -> bool {
        if task_id >= 32 {
            false
        } else {
            let ptr = self.tasks[task_id as usize].get();
            if (*ptr).is_none() {
                *self.tasks[task_id as usize].get() = Some(f);
                self.set_task_ready(TaskId::unsafe_new(task_id));
                true
            } else {
                false
            }
        }
    }
}
