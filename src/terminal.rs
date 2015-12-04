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

    process_command(
        usart,
        unsafe { ::core::str::from_utf8_unchecked(&command[0 .. cur])});
}

fn process_command(usart: &Usart, command: &str) {
    match command {
        "hi" => { usart.puts_synchronous("Hi, there!\r\n"); },
        "-3" => { led::LD3.turn_off(); },
        "+3" => { led::LD3.turn_on(); },
        "-4" => { led::LD4.turn_off(); },
        "+4" => { led::LD4.turn_on(); },
        "-5" => { led::LD5.turn_off(); },
        "+5" => { led::LD5.turn_on(); },
        "-6" => { led::LD6.turn_off(); },
        "+6" => { led::LD6.turn_on(); },
        "panic" => {
            panic!();
        }
        "" => {},
        _ => {
            usart.puts_synchronous("Unknown command: \"");
            usart.puts_synchronous(command);
            usart.puts_synchronous("\"\r\n");
        },
    }
}
