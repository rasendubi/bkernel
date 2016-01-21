pub use ::bscheduler::Task;
use ::bscheduler::Scheduler;
use ::core::cell::UnsafeCell;

struct SchedulerCell(UnsafeCell<Option<Scheduler<'static>>>);
unsafe impl Sync for SchedulerCell { }

static SCHEDULER: SchedulerCell = SchedulerCell(UnsafeCell::new(None));

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
