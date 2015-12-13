mod interpreter;

pub use self::interpreter::interpret;

pub trait Io {
    fn read_byte(&mut self) -> u8;
    fn write_byte(&mut self, byte: u8);
}
