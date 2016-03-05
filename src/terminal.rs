use global::Global;
use led;
use led_music;
use log;

use queue::Queue;

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

static QUEUE: Global<Queue<u8>> = Global::new_empty();

/// Starts a terminal.
///
/// Note that terminal is non-blocking and is driven by `put_char`.
/// (Well, it mostly non-blocking. There is no non-blocking version
/// for `puts_synchronous`.)
pub fn run_terminal() {
    QUEUE.init(Queue::new());
    log::write_str(PROMPT);
    wait_char();
}

fn wait_char() {
    QUEUE.get().get_task(::bscheduler::Task {
        name: "terminal::next_char",
        priority: 5,
        function: Box::new(process),
    });
}

/// Puts a char to process.
///
/// Safe to call from ISR.
pub fn put_char(c: u32) {
    QUEUE.get().put(c as u8);
}

fn get_pending_char() -> Option<u32> {
    unsafe {
        let irq = ::stm32f4::save_irq();
        let c = QUEUE.get().get();
        ::stm32f4::restore_irq(irq);
        c.map(|x| x as u32)
    }
}

/// Processes all pending characters in the queue.
fn process() {
    // TODO: this flow can be abstracted out as it's useful for many
    // queue-processing tasks.
    while let Some(c) = get_pending_char() {
        process_char(c);
    }
    wait_char();
}

/// Processes one character at a time. Calls `process_command` when
/// user presses Enter or command is too long.
fn process_char(c: u32) {
    unsafe {
        if c == '\r' as u32 {
            log::write_str("\r\n");
            process_command(&COMMAND[0 .. CUR]);
            CUR = 0;
            return;
        }

        if c == 0x8 { // backspace
            if CUR != 0 {
                log::write_str("\x08 \x08");

                CUR -= 1;
            }
        } else {
            COMMAND[CUR] = c as u8;
            CUR += 1;
            log::write_char(c);

            if CUR == 256 {
                log::write_str("\r\n");
                process_command(&COMMAND[0 .. CUR]);
                CUR = 0;
            }
        }
    }
}

fn process_command(command: &[u8]) {
    match command {
        b"help" => { log::write_str(HELP_MESSAGE); },
        b"hi" => { log::write_str("Hi, there!\r\n"); },
        b"pony" | b"p" => { log::write_str(PONY); },
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
            log::write_str("Unknown command: \"");
            log::write_bytes(command);
            log::write_str("\"\r\n");
        },
    }

    log::write_str(PROMPT);
}
