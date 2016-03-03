use stm32f4::usart::Usart;
use led;
use brainfuck;
use led_music;

const PROMPT: &'static str = "> ";

const HELP_MESSAGE: &'static str = "
Available commands:\r
hi      -- welcomes you\r
pony    -- surprise!\r
-3/+3   -- turn off/on LED3\r
-4/+4   -- turn off/on LED4\r
-5/+5   -- turn off/on LED5\r
-6/+6   -- turn off/on LED6\r
led-fun -- some fun with LEDs\r
b       -- f*ck your brain\r
panic   -- throw a panic\r
help    -- print this help\r
exit    -- exits the terminal. That will make LED blink\r
";

pub fn run_terminal(usart: &Usart) {
    while handle_command(&usart) {}
}

pub fn handle_command(usart: &Usart) -> bool {
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

    process_command(usart, &command[0 .. cur])
}

// return true if should continue
fn process_command(usart: &Usart, command: &[u8]) -> bool {
    if command.len() >= 2 && &command[..2] == b"b " {
        brainfuck::interpret(&command[2..], &mut ::stm32f4::usart::UsartProxy(usart));
        return true;
    }
    match command {
        b"help" => { usart.puts_synchronous(HELP_MESSAGE); },
        b"hi" => { usart.puts_synchronous("Hi, there!\r\n"); },
        b"pony" | b"p" => {
            // https://raw.githubusercontent.com/mbasaglia/ASCII-Pony/master/Ponies/vinyl-scratch-noglasses.txt
            // https://github.com/mbasaglia/ASCII-Pony/
            usart.puts_synchronous("
                                                     __..___\r
                                               _.-'____<'``\r
                                         ___.-`.-'`     ```_'-.\r
                                        /  \\.'` __.----'','/.._\\\r
                                       ( /  \\_/` ,---''.' /   `-'\r
                                       | |    `,._\\  ,'  /``''-.,`.\r
                                      /( '.  \\ _____    ' )   `. `-;\r
                                     ( /\\   __/   __\\  / `:     \\\r
                                     || (\\_  (   /.- | |'.|      :\r
           _..._)`-._                || : \\ ,'\\ ((WW | \\W)j       \\\r
        .-`.--''---._'-.             |( (, \\   \\_\\_ /   ``-.  \\.   )\r
      /.-'`  __---__ '-.'.           ' . \\`.`.         \\__/-   )`. |\r
      /    ,'     __`-. '.\\           V(  \\ `-\\-,______.-'  `. |  `'\r
     /    /    .'`  ```:. \\)___________/\\ .`.     /.^. /| /.  \\|\r
    (    (    /   .'  '-':-'             \\|`.:   (/   V )/ |  )'\r
    (    (   (   (      /   |'-..             `   \\    /,  |  '\r
    (  ,  \\   \\   \\    |   _|``-|                  |       | /\r
     \\ |.  \\   \\-. \\   |  (_|  _|                  |       |'\r
      \\| `. '.  '.`.\\  |      (_|                  |\r
       '   '.(`-._\\ ` / \\        /             \\__/\r
              `  ..--'   |      /-,_______\\       \\\r
               .`      _/      /     |    |\\       \\\r
                \\     /       /     |     | `--,    \\\r
                 \\    |      |      |     |   /      )\r
                  \\__/|      |      |      | (       |\r
                      |      |      |      |  \\      |\r
                      |       \\     |       \\  `.___/\r
                       \\_______)     \\_______)\r
");
        },
        b"-3" => { led::LD3.turn_off(); },
        b"+3" => { led::LD3.turn_on(); },
        b"-4" => { led::LD4.turn_off(); },
        b"+4" => { led::LD4.turn_on(); },
        b"-5" => { led::LD5.turn_off(); },
        b"+5" => { led::LD5.turn_on(); },
        b"-6" => { led::LD6.turn_off(); },
        b"+6" => { led::LD6.turn_on(); },
        b"led-fun" => { led_music::led_fun(71000); },
        b"panic" => {
            panic!();
        }
        b"exit" => { return false },
        b"" => {},
        _ => {
            usart.puts_synchronous("Unknown command: \"");
            usart.put_bytes(command);
            usart.puts_synchronous("\"\r\n");
        },
    }
    true
}
