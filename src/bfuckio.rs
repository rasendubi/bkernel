use stm32f4::usart::Usart;
use brainfuck::Io;

impl Io for Usart {
    fn read_byte(&self) -> u8 {
        self.get_char() as u8
    }
    fn write_byte(&self, byte: u8) {
        self.put_char(byte as u32)
    }
}
