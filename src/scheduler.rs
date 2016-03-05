use global::Global;
pub use ::bscheduler::Task;
use ::bscheduler::Scheduler;

use ::stm32f4::{save_irq, restore_irq};

static SCHEDULER: Global<Scheduler<'static>> = Global::new_empty();

pub fn init() {
    unsafe {
        let irq = save_irq();
        SCHEDULER.init(Scheduler::new());
        restore_irq(irq);
    }
}

pub fn schedule() -> ! {
    loop {
        unsafe {
            // we reschedule with interrupts disabled. A task can enable
            // interrupts if it can handle that
            let irq = save_irq();
            SCHEDULER.get().reschedule();
            restore_irq(irq);
        }
    }
}

pub fn add_task(task: Task<'static>) {
    unsafe {
        let irq = save_irq();
        SCHEDULER.get().add_task(task);
        restore_irq(irq);
    }
}
