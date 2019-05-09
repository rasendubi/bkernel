//! I2C module adapter for use with futures.

use ::core::cell::UnsafeCell;
use ::core::marker::PhantomData;

use stm32f4::i2c::{self, I2c};

use futures::{Async, Future, Poll};

use breactor::mutex::{Mutex, MutexLock};
use breactor::promise::Promise;

pub static I2C1_BUS: I2cBus = I2cBus::new(unsafe{&i2c::I2C1});
pub static I2C2_BUS: I2cBus = I2cBus::new(unsafe{&i2c::I2C2});
pub static I2C3_BUS: I2cBus = I2cBus::new(unsafe{&i2c::I2C3});

#[allow(missing_debug_implementations)]
pub struct I2cBus {
    i2c: &'static I2c,
    mutex: Mutex,
    slave_address: UnsafeCell<u16>,
    buffer: UnsafeCell<*mut u8>,
    buf_left: UnsafeCell<usize>,

    result: UnsafeCell<Promise<(), Error>>,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Error {
    /// Failed to lock I2C bus.
    ///
    /// This should practically never occur.
    LockError,

    /// Acknowledgement failure.
    ///
    /// The device has not acknowledged its address or data byte.
    AcknowledgementFailure,

    ArbitrationLost,

    BusError,

    /// Unknown I2C error.
    ///
    /// The internal value is I2C event.
    Unknown(u32),
}

#[allow(missing_debug_implementations)]
pub struct I2cTransfer {
    #[allow(dead_code)]
    lock: MutexLock<'static>,

    bus: &'static I2cBus,
}

unsafe impl Sync for I2cBus {
}

impl I2cBus {
    const fn new(i2c: &'static I2c) -> Self {
        I2cBus {
            i2c,
            mutex: Mutex::new(),
            slave_address: UnsafeCell::new(0),
            buffer: UnsafeCell::new(::core::ptr::null_mut()),
            buf_left: UnsafeCell::new(0),
            result: UnsafeCell::new(unsafe { Promise::empty() }),
        }
    }

    pub const fn start_transfer(&'static self) -> StartTransferFuture {
        StartTransferFuture { bus: self }
    }
}

#[allow(missing_debug_implementations)]
pub struct StartTransferFuture {
    bus: &'static I2cBus,
}

impl Future for StartTransferFuture {
    type Item = I2cTransfer;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<I2cTransfer>, Error> {
        self.bus.mutex.lock()
            .map(move |lock| I2cTransfer { lock, bus: self.bus })
            .map_err(|_| Error::LockError)
            .poll()
    }
}

#[allow(missing_debug_implementations)]
pub struct Transmission<'a> {
    transfer: Option<I2cTransfer>,

    data: *mut u8,
    size: usize,

    __phantom: PhantomData<&'a u8>,
}

impl<'a> Future for Transmission<'a> {
    type Item = (I2cTransfer, &'a [u8]);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let result = self.transfer.as_ref().unwrap().bus.result.get();
        unsafe {
            try_ready!((*result).poll());
            Ok(Async::Ready((self.transfer.take().unwrap(),
                             ::core::slice::from_raw_parts(self.data, self.size))))
        }
    }
}

impl I2cTransfer {
    pub fn master_transmitter(self, addr: u16, data: &[u8]) -> Transmission {
        self.master_transmitter_raw(addr, data.as_ptr(), data.len())
    }

    pub fn master_transmitter_raw<'a>(self, addr: u16, data_ptr: *const u8, data_size: usize) -> Transmission<'a> {
        unsafe {
            *self.bus.slave_address.get() = addr;
            *self.bus.buffer.get() = data_ptr as *mut u8;
            *self.bus.buf_left.get() = data_size;
            *self.bus.result.get() = Promise::new();

            self.bus.i2c.generate_start();

            self.bus.i2c.it_enable(i2c::Interrupt::Evt);
            self.bus.i2c.it_enable(i2c::Interrupt::Buf);
            self.bus.i2c.it_enable(i2c::Interrupt::Err);
        }

        Transmission {
            transfer: Some(self),
            data: data_ptr as *mut _,
            size: data_size,
            __phantom: PhantomData,
        }
    }

    pub fn master_receiver(self, addr: u16, data: &mut [u8]) -> Transmission {
        self.master_receiver_raw(addr, data.as_mut_ptr(), data.len())
    }

    pub fn master_receiver_raw<'a>(self, addr: u16, data_ptr: *mut u8, data_size: usize) -> Transmission<'a> {
        unsafe {
            *self.bus.slave_address.get() = addr | 0x01;
            *self.bus.buffer.get() = data_ptr;
            *self.bus.buf_left.get() = data_size;
            *self.bus.result.get() = Promise::new();

            self.bus.i2c.generate_start();
            self.bus.i2c.set_acknowledge(true);

            self.bus.i2c.it_enable(i2c::Interrupt::Evt);
            self.bus.i2c.it_enable(i2c::Interrupt::Buf);
            self.bus.i2c.it_enable(i2c::Interrupt::Err);
        }

        Transmission {
            transfer: Some(self),
            data: data_ptr,
            size: data_size,
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
pub unsafe extern fn __isr_i2c1_ev() {
    let bus = &I2C1_BUS;

    let event = bus.i2c.get_last_event();

    if event == 0x30000 { // MSL, BUSY
        return;
    }
    if event == 0x0 {
        return;
    }

    match ::core::mem::transmute(event) {
        i2c::Event::MasterModeSelect => {
            let slave_address = *bus.slave_address.get();
            // not really data, but who cares
            // TODO(ashmalko): handle ADDR10
            bus.i2c.send_data(slave_address as u8);
        },
        i2c::Event::MasterTransmitterModeSelected |
        i2c::Event::MasterReceiverModeSelected => {
            let buf_left = bus.buf_left.get();
            if (*buf_left) == 1 {
                bus.i2c.set_acknowledge(false);
            }
        },
        i2c::Event::MasterByteTransmitted => {
            let buf_left = bus.buf_left.get();

            if *buf_left == 0 {
                bus.i2c.it_disable(i2c::Interrupt::Evt);
                bus.i2c.it_disable(i2c::Interrupt::Buf);
                bus.i2c.it_disable(i2c::Interrupt::Err);

                let result = bus.result.get();
                (*result).resolve(Ok(()));
            }
        }
        i2c::Event::MasterByteTransmitting => {
            let buffer = bus.buffer.get();
            let buf_left = bus.buf_left.get();

            if *buf_left > 0 {
                bus.i2c.send_data(**buffer);

                *buf_left -= 1;
                (*buffer) = (*buffer).offset(1);
            }
        },
        i2c::Event::MasterByteReceived => {
            let buffer = bus.buffer.get();
            let buf_left = bus.buf_left.get();

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
            // TODO(ashmalko): this function should be rewritten to
            // check particular status flags, and not matching events
            // as whole.
            // panic!("__isr_i2c1_ev(): unknown event 0x{:x}", event);
        }
    }
}

#[no_mangle]
pub unsafe extern fn __isr_i2c1_er() {
    let bus = &I2C1_BUS;

    let event = bus.i2c.get_last_event();

    bus.i2c.it_disable(i2c::Interrupt::Evt);
    bus.i2c.it_disable(i2c::Interrupt::Buf);
    bus.i2c.it_disable(i2c::Interrupt::Err);

    let error = if event & (i2c::Sr1Masks::AF as u32) != 0 {
        bus.i2c.it_clear_pending(i2c::Sr1Masks::AF as u32);
        Error::AcknowledgementFailure
    } else if event & (i2c::Sr1Masks::ARLO as u32) != 0 {
        bus.i2c.it_clear_pending(i2c::Sr1Masks::ARLO as u32);
        Error::ArbitrationLost
    } else if event & (i2c::Sr1Masks::BERR as u32) != 0 {
        bus.i2c.it_clear_pending(i2c::Sr1Masks::BERR as u32);
        Error::BusError
    } else {
        Error::Unknown(event)
    };

    let result = bus.result.get();
    (*result).resolve(Err(error));
}

#[no_mangle]
pub extern fn __isr_i2c2_ev() {
}

#[no_mangle]
pub extern fn __isr_i2c2_er() {
}

#[no_mangle]
pub extern fn __isr_i2c3_ev() {
}

#[no_mangle]
pub extern fn __isr_i2c3_er() {
}
