use stm32f4::usart::{Usart, USART1};
use led;
use brainfuck;
use led_music;

use queue::Queue;

use ::core::cell::UnsafeCell;
use ::alloc::boxed::Box;

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
";

// https://raw.githubusercontent.com/mbasaglia/ASCII-Pony/master/Ponies/vinyl-scratch-noglasses.txt
// https://github.com/mbasaglia/ASCII-Pony/
const PONY: &'static str = "
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
";

static mut COMMAND: [u8; 256] = [0; 256];
static mut CUR: usize = 0;

// Well, that's a trick to overcome `error: statics are not allowed to have destructors`.
// TODO: factor it out -- it was used in scheduler module too.
struct QueueCell(UnsafeCell<Option<Queue<u32>>>);
unsafe impl Sync for QueueCell { }
static QUEUE: QueueCell = QueueCell(UnsafeCell::new(None));

/// Starts a terminal.
///
/// Note that terminal is non-blocking and is driven by `put_char`.
/// (Well, it mostly non-blocking. There is no non-blocking version
/// for `puts_synchronous`.)
pub fn run_terminal(usart: &Usart) {
    unsafe {
        *QUEUE.0.get() = Some(Queue::new());
        usart.puts_synchronous(PROMPT);
        wait_char();
    }
}

fn wait_char() {
    unsafe {
        (*QUEUE.0.get()).as_mut().unwrap().get_task(::bscheduler::Task {
            name: "terminal::next_char",
            priority: 5,
            function: Box::new(process),
        });
    }
}

/// Puts a char to process.
///
/// Safe to call from ISR.
pub fn put_char(c: u32) {
    unsafe {
        (*QUEUE.0.get()).as_mut().unwrap().put(c);
    }
}

fn get_pending_char() -> Option<u32> {
    unsafe {
        let irq = ::stm32f4::save_irq();
        let c = (*QUEUE.0.get()).as_mut().unwrap().get();
        ::stm32f4::restore_irq(irq);
        c
    }
}

/// Processes all pending characters in the queue.
fn process() {
    // TODO: this flow can be abstracted out as it's useful for many
    // queue-processing tasks.
    while let Some(c) = get_pending_char() {
        process_char(&USART1, c);
    }
    wait_char();
}

/// Processes one character at a time. Calls `process_command` when
/// user presses Enter or command is too long.
fn process_char(usart: &Usart, c: u32) {
    unsafe {
        if c == '\r' as u32 {
            usart.puts_synchronous("\r\n");
            process_command(usart, &COMMAND[0 .. CUR]);
            CUR = 0;
            return;
        }

        if c == 0x8 { // backspace
            if CUR != 0 {
                usart.puts_synchronous("\x08 \x08");

                CUR -= 1;
            }
        } else {
            COMMAND[CUR] = c as u8;
            CUR += 1;
            usart.put_char(c);

            if CUR == 256 {
                usart.puts_synchronous("\r\n");
                process_command(usart, &COMMAND[0 .. CUR]);
                CUR = 0;
            }
        }
    }
}

fn process_command(usart: &Usart, command: &[u8]) {
    if command.len() >= 2 && &command[..2] == b"b " {
        brainfuck::interpret(&command[2..], &mut ::stm32f4::usart::UsartProxy(usart));
        usart.puts_synchronous("> ");
        return;
    }
    match command {
        b"help" => { usart.puts_synchronous(HELP_MESSAGE); },
        b"hi" => { usart.puts_synchronous("Hi, there!\r\n"); },
        b"pony" | b"p" => { usart.puts_synchronous(PONY); },
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
        b"" => {},
        _ => {
            usart.puts_synchronous("Unknown command: \"");
            usart.put_bytes(command);
            usart.puts_synchronous("\"\r\n");
        },
    }

    usart.puts_synchronous("> ");
}
