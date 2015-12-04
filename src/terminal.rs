use stm32f4::usart::Usart;
use led;

const PROMPT: &'static str = "> ";

pub fn run_terminal(usart: &Usart) {
    loop {
        handle_command(&usart);
    }
}

pub fn handle_command(usart: &Usart) {
    let mut command: [u8; 256] = unsafe { ::core::mem::uninitialized() };
    let mut cur = 0;

    usart.puts_synchronous(PROMPT);

    let mut c = usart.get_char();
    while c != '\r' as u32 {
        if c == 0x8 { // backspace
            if cur != 0 {
                usart.puts_synchronous("\x08 \x08");

                cur -= 1;
            }
        } else {
            command[cur] = c as u8;
            cur += 1;
            usart.put_char(c);

            if cur == 256 {
                break;
            }
        }

        c = usart.get_char();
    }
    usart.puts_synchronous("\r\n");

    process_command(usart, &command[0 .. cur]);
}

fn process_command(usart: &Usart, command: &[u8]) {
    match command {
        b"hi" => { usart.puts_synchronous("Hi, there!\r\n"); },
        b"-3" => { led::LD3.turn_off(); },
        b"+3" => { led::LD3.turn_on(); },
        b"-4" => { led::LD4.turn_off(); },
        b"+4" => { led::LD4.turn_on(); },
        b"-5" => { led::LD5.turn_off(); },
        b"+5" => { led::LD5.turn_on(); },
        b"-6" => { led::LD6.turn_off(); },
        b"+6" => { led::LD6.turn_on(); },
        b"panic" => {
            panic!();
        }
        b"" => {},
        _ => {
            usart.puts_synchronous("Unknown command: \"");
            usart.put_bytes(command);
            usart.puts_synchronous("\"\r\n");
        },
    }
}
