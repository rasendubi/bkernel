//! I2C module adapter for use with futures.

use ::core::cell::UnsafeCell;

use stm32f4::i2c::{self, I2c};

use futures::{Async, AsyncSink, Poll, Sink, StartSend};

pub struct I2cBus<'a> {
    i2c: &'a I2c,
    current_transaction: UnsafeCell<Option<I2cTransaction>>,
}

pub enum I2cTransaction {
    MasterTransmitter {
        slave_address: u8,
        buffer: *const u8,
        bytes_left: usize,
    },
}

pub static I2C1_BUS: I2cBus = I2cBus::new(unsafe{&i2c::I2C1});
pub static I2C2_BUS: I2cBus = I2cBus::new(unsafe{&i2c::I2C2});
pub static I2C3_BUS: I2cBus = I2cBus::new(unsafe{&i2c::I2C3});

unsafe impl Sync for I2cBus<'static> {
}

impl<'a> I2cBus<'a> {
    const fn new(i2c: &'a I2c) -> Self {
        I2cBus {
            i2c,
            current_transaction: UnsafeCell::new(None),
        }
    }
}

impl I2cTransaction {
    /// The buffer reference must remain valid until the result object
    /// is dropped.
    pub fn master_transmitter<'b>(slave_address: u8, buffer: *const u8, buffer_size: usize) -> I2cTransaction {
        I2cTransaction::MasterTransmitter {
            slave_address,
            buffer,
            bytes_left: buffer_size,
        }
    }
}

impl<'a, 'b> Sink for &'b I2cBus<'a> {
    type SinkItem = I2cTransaction;
    type SinkError = ();

    fn start_send(&mut self, item: I2cTransaction) -> StartSend<Self::SinkItem, Self::SinkError> {
        unsafe {
            // TODO(ashmalko): race condition
            let current_transaction = self.current_transaction.get();
            if (*current_transaction).is_none() {
                *current_transaction = Some(item);
                match *current_transaction {
                    Some(I2cTransaction::MasterTransmitter{..}) => {
                        self.i2c.it_enable(i2c::Interrupt::Evt);
                        self.i2c.it_enable(i2c::Interrupt::Buf);
                        self.i2c.it_enable(i2c::Interrupt::Err);
                        self.i2c.generate_start();
                    },
                    None => { },
                }
                Ok(AsyncSink::Ready)
            } else {
                Ok(AsyncSink::NotReady(item))
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        unsafe {
            if (*self.current_transaction.get()).is_none() {
                Ok(Async::Ready(()))
            } else {
                Ok(Async::NotReady)
            }
        }
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        self.poll_complete()
    }
}

#[no_mangle]
pub unsafe extern "C" fn __isr_i2c1_ev() {
    let bus = &I2C1_BUS;

    let event = bus.i2c.get_last_event();
    match ::core::mem::transmute(event) {
        i2c::Event::MasterModeSelect => {
            let current_transaction = bus.current_transaction.get();
            match *current_transaction {
                Some(I2cTransaction::MasterTransmitter{slave_address, ..}) => {
                    bus.i2c.send_7bit_address(
                        slave_address,
                        i2c::Direction::Transmitter,
                    );
                },
                None => {
                },
            }
        }
        _ => {
            panic!("__isr_i2c1_ev(): unknown event 0x{:x}", event);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn __isr_i2c1_er() {
    let bus = &I2C1_BUS;

    let event = bus.i2c.get_last_event();
    if event & (i2c::Sr1Masks::AF as u32) != 0 {
        panic!("__isr_i2c1_er(): 0x{:x} (acknowledge failure)", event);
    }
    panic!("__isr_i2c1_er(): 0x{:x}", event);
}

#[no_mangle]
pub extern "C" fn __isr_i2c2_ev() {
    panic!("__isr_i2c1_ev()");
}

#[no_mangle]
pub extern "C" fn __isr_i2c2_er() {
}

#[no_mangle]
pub extern "C" fn __isr_i2c3_ev() {
}

#[no_mangle]
pub extern "C" fn __isr_i2c3_er() {
}
