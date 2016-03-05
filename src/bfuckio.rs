use brainfuck::Io;
use log;

pub enum SystemLog {
    Log,
}

impl Io for SystemLog {
    fn write_byte(&mut self, byte: u8) {
        log::write_char(byte as u32);
    }

    fn read_byte(&mut self) -> u8 {
        panic!("Reading is not implemented for brainfuck interpreter");
    }
}
