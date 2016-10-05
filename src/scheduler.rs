pub use ::bscheduler::Task;
use ::bscheduler::Scheduler;

use ::stm32f4::{save_irq, restore_irq};

static SCHEDULER: Scheduler<'static> = Scheduler::new();

pub fn schedule() -> ! {
    loop {
        unsafe {
            // we reschedule with interrupts disabled. A task can enable
            // interrupts if it can handle that
            let irq = save_irq();
            SCHEDULER.reschedule();
            restore_irq(irq);
        }
    }
}

pub fn add_task(task: *mut Task<'static>) {
    unsafe {
        let irq = save_irq();
        SCHEDULER.add_task(task);
        restore_irq(irq);
    }
}
