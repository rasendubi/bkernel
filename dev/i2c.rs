//! I2C module adapter for use with futures.

use ::core::cell::UnsafeCell;
use ::core::marker::PhantomData;

use stm32f4::i2c::{self, I2c};

use futures::{Async, Future, Poll};

use breactor::mutex::{Mutex, MutexLock};
use breactor::promise::Promise;

pub struct I2cBus {
    i2c: &'static I2c,
    mutex: Mutex,
    slave_address: UnsafeCell<u16>,
    buffer: UnsafeCell<*mut u8>,
    buf_left: UnsafeCell<usize>,

    result: UnsafeCell<Promise<(), u32>>,
}

pub struct I2cTransfer {
    #[allow(dead_code)]
    lock: MutexLock<'static>,

    bus: &'static I2cBus,
}

pub static I2C1_BUS: I2cBus = I2cBus::new(unsafe{&i2c::I2C1});
pub static I2C2_BUS: I2cBus = I2cBus::new(unsafe{&i2c::I2C2});
pub static I2C3_BUS: I2cBus = I2cBus::new(unsafe{&i2c::I2C3});

unsafe impl Sync for I2cBus {
}

impl I2cBus {
    const fn new(i2c: &'static I2c) -> Self {
        I2cBus {
            i2c,
            mutex: Mutex::new(),
            slave_address: UnsafeCell::new(0),
            buffer: UnsafeCell::new(0 as *mut u8),
            buf_left: UnsafeCell::new(0),
            result: UnsafeCell::new(unsafe { Promise::empty() }),
        }
    }

    pub fn start_transfer(&'static self) -> impl Future<Item=I2cTransfer, Error=()> + Sized {
        self.mutex.map(move |lock| I2cTransfer { lock, bus: self })
    }
}

pub struct Transmission<'a> {
    transfer: Option<I2cTransfer>,

    data: *mut u8,
    size: usize,

    __phantom: PhantomData<&'a u8>,
}

impl<'a> Future for Transmission<'a> {
    type Item = (I2cTransfer, &'a [u8]);
    type Error = u32;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let result = self.transfer.as_ref().unwrap().bus.result.get();
        unsafe {
            let _ = try_ready!((*result).poll());
            Ok(Async::Ready((self.transfer.take().unwrap(),
                             ::core::slice::from_raw_parts(self.data, self.size))))
        }
    }
}

impl I2cTransfer {
    pub fn master_transmitter<'a>(self, addr: u16, data: &'a [u8]) -> Transmission<'a> {
        unsafe {
            *self.bus.slave_address.get() = addr;
            *self.bus.buffer.get() = data.as_ptr() as *mut u8;
            *self.bus.buf_left.get() = data.len();
            *self.bus.result.get() = Promise::new();

            let mut event = self.bus.i2c.get_last_event();
            while event & 0x20000 != 0 {
                event = self.bus.i2c.get_last_event();
            }

            // if event & 0x0100 != 0 {
            //     self.bus.i2c.it_clear_pending(0x0100);
            // }

            self.bus.i2c.generate_start();

            self.bus.i2c.it_enable(i2c::Interrupt::Evt);
            self.bus.i2c.it_enable(i2c::Interrupt::Buf);
            self.bus.i2c.it_enable(i2c::Interrupt::Err);
        }

        Transmission {
            transfer: Some(self),
            data: data.as_ptr() as *mut _,
            size: data.len(),
            __phantom: PhantomData,
        }
    }

    pub fn master_receiver<'a>(self, addr: u16, data: &'a mut [u8]) -> Transmission<'a> {
        unsafe {
            *self.bus.slave_address.get() = addr | 0x01;
            *self.bus.buffer.get() = data.as_mut_ptr();
            *self.bus.buf_left.get() = data.len();
            *self.bus.result.get() = Promise::new();

            self.bus.i2c.generate_start();
            self.bus.i2c.set_acknowledge(true);

            self.bus.i2c.it_enable(i2c::Interrupt::Evt);
            self.bus.i2c.it_enable(i2c::Interrupt::Buf);
            self.bus.i2c.it_enable(i2c::Interrupt::Err);
        }

        Transmission {
            transfer: Some(self),
            data: data.as_mut_ptr(),
            size: data.len(),
            __phantom: PhantomData,
        }
    }

    pub fn stop(&mut self) {
        // TODO: check START has been generated before?
        unsafe {
            self.bus.i2c.generate_stop();
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn __isr_i2c1_ev() {
    let bus = &I2C1_BUS;

    let event = bus.i2c.get_last_event();

    match ::core::mem::transmute(event) {
        i2c::Event::MasterModeSelect => {
            let slave_address = *bus.slave_address.get();
            // not really data, but who cares
            bus.i2c.send_data(slave_address as u8);
        },
        i2c::Event::MasterTransmitterModeSelected => {
        },
        i2c::Event::MasterReceiverModeSelected => {
        },
        i2c::Event::MasterByteTransmitted => {
            if *bus.buf_left.get() == 0 {
                bus.i2c.it_disable(i2c::Interrupt::Evt);
                bus.i2c.it_disable(i2c::Interrupt::Buf);
                bus.i2c.it_disable(i2c::Interrupt::Err);

                let result = bus.result.get();
                (*result).resolve(Ok(()));
            }
        },
        i2c::Event::MasterByteTransmitting => { // | i2c::Event::MasterByteTransmitted => {
            let mut buffer = bus.buffer.get();
            let mut buf_left = bus.buf_left.get();

            if *buf_left > 0 {
                bus.i2c.send_data(**buffer);

                *buf_left -= 1;
                (*buffer) = (*buffer).offset(1);
            }
        },
        i2c::Event::MasterByteReceived => {
            let mut buffer = bus.buffer.get();
            let mut buf_left = bus.buf_left.get();

            debug_assert!(*buf_left > 0);

            **buffer = bus.i2c.receive_data();

            *buf_left -= 1;
            (*buffer) = (*buffer).offset(1);

            if *buf_left == 1 {
                bus.i2c.set_acknowledge(false);
            } else if *buf_left == 0 {
                let result = bus.result.get();
                (*result).resolve(Ok(()));

                bus.i2c.it_disable(i2c::Interrupt::Evt);
                bus.i2c.it_disable(i2c::Interrupt::Buf);
                bus.i2c.it_disable(i2c::Interrupt::Err);
            }
        },
        _ => {
            panic!("__isr_i2c1_ev(): unknown event 0x{:x}", event);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn __isr_i2c1_er() {
    let bus = &I2C1_BUS;

    let event = bus.i2c.get_last_event();

    let result = bus.result.get();
    (*result).resolve(Err(event));

    panic!("__isr_i2c1_er(): 0x{:x}", event);
}

#[no_mangle]
pub extern "C" fn __isr_i2c2_ev() {
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
