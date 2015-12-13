use stm32f4::usart::UsartProxy;
use brainfuck::Io;

impl<'a> Io for UsartProxy<'a> {
    fn read_byte(&mut self) -> u8 {
        self.0.get_char() as u8
    }
    fn write_byte(&mut self, byte: u8) {
        self.0.put_char(byte as u32)
    }
}
