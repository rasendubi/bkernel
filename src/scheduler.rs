pub use ::bscheduler::Task;
use ::bscheduler::Scheduler;
use ::core::cell::UnsafeCell;

use ::stm32f4::{save_irq, restore_irq};

struct SchedulerCell(UnsafeCell<Option<Scheduler<'static>>>);
unsafe impl Sync for SchedulerCell { }

static SCHEDULER: SchedulerCell = SchedulerCell(UnsafeCell::new(None));

pub fn init() {
    unsafe {
        let irq = save_irq();
        *SCHEDULER.0.get() = Some(Scheduler::new());
        restore_irq(irq);
    }
}

pub fn schedule() -> ! {
    loop {
        unsafe {
            // we reschedule with interrupts disabled. A task can enable
            // interrupts if it can handle that
            let irq = save_irq();
            (*SCHEDULER.0.get()).as_mut().unwrap().reschedule();
            restore_irq(irq);
        }
    }
}

pub fn add_task(task: Task<'static>) {
    unsafe {
        let irq = save_irq();
        (*SCHEDULER.0.get()).as_mut().unwrap().add_task(task);
        restore_irq(irq);
    }
}
