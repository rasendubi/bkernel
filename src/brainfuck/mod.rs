mod interpreter;

pub use self::interpreter::interpret;

pub trait Io {
    fn read_byte(&self) -> u8;
    fn write_byte(&self, byte: u8);
}
