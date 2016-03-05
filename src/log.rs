//! Logging.

use queue::Queue;
use stm32f4::{save_irq, restore_irq};
use stm32f4::usart;
use stm32f4::usart::USART1;
use ::core::cell::UnsafeCell;

struct QueueCell(UnsafeCell<Option<Queue<u32>>>);
unsafe impl Sync for QueueCell { }
static QUEUE: QueueCell = QueueCell(UnsafeCell::new(None));

/// Return queue.
fn get_queue() -> &'static mut Queue<u32> {
    unsafe {
        (*QUEUE.0.get()).as_mut().unwrap() as &mut _
    }
}

/// Should be called before any other function from this module.
pub fn init() {
    unsafe {
        *QUEUE.0.get() = Some(Queue::new());
    }
}

pub fn write_bytes(bytes: &[u8]) {
    for b in bytes {
        get_queue().put(*b as u32);
    }
    new_data_added();
}

pub fn write_str(s: &str) {
    for c in s.chars() {
        get_queue().put(c as u32);
    }
    new_data_added();
}

pub fn write_char(c: u32) {
    get_queue().put(c);
    new_data_added();
}

fn new_data_added() {
    unsafe {
        let irq = save_irq();
        if !USART1.it_enabled(usart::Interrupt::TXE) {
            match get_queue().get() {
                Some(c) => {
                    USART1.it_enable(usart::Interrupt::TXE);
                    USART1.put_unsafe(c);
                },
                None => {},
            }
        }
        restore_irq(irq);
    }
}

pub fn usart1_txe() {
    match get_queue().get() {
        Some(c) => unsafe { USART1.put_unsafe(c); },
        None => {
            USART1.it_disable(usart::Interrupt::TXE)
        },
    }
}
