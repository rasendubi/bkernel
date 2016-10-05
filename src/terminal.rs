use led;
use led_music;
use log;

use queue::Queue;

use scheduler::Task;

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

static mut QUEUE: Option<Queue<u8>> = None;

/// Starts a terminal.
///
/// Note that terminal is non-blocking and is driven by `put_char`.
pub fn run_terminal() {
    unsafe {
        QUEUE = Some(Queue::new(&mut PROCESS_TASK as *mut _));
    }
    log::write_str(PROMPT);
}

static mut PROCESS_TASK: Task<'static> = Task::from_safe("terminal::next_char", 5, process, 0 as *const _);

/// Puts a char to process.
///
/// Safe to call from ISR.
pub fn put_char(c: u32) {
    unsafe{&mut QUEUE}.as_mut().unwrap().put(c as u8);
}

fn get_pending_char() -> Option<u32> {
    unsafe {
        let irq = ::stm32f4::save_irq();
        let c = QUEUE.as_mut().unwrap().get();
        ::stm32f4::restore_irq(irq);
        c.map(|x| x as u32)
    }
}

/// Processes all pending characters in the queue.
fn process(_arg: *const ()) {
    // TODO: this flow can be abstracted out as it's useful for many
    // queue-processing tasks.
    while let Some(c) = get_pending_char() {
        process_char(c);
    }
}

/// Processes one character at a time. Calls `process_command` when
/// user presses Enter or command is too long.
fn process_char(c: u32) {
    static mut COMMAND: [u8; 256] = [0; 256];
    static mut CUR: usize = 0;

    let mut command = unsafe{&mut COMMAND}; // COMMAND is only used in this function
    let mut cur = unsafe{&mut CUR};

    if c == '\r' as u32 {
        log::write_str("\r\n");
        process_command(&command[0 .. *cur]);
        *cur = 0;
        return;
    }

    if c == 0x8 { // backspace
        if *cur != 0 {
            log::write_str("\x08 \x08");

            *cur -= 1;
        }
    } else {
        command[*cur] = c as u8;
        *cur += 1;
        log::write_char(c);

        if *cur == 256 {
            log::write_str("\r\n");
            process_command(&command[0 .. *cur]);
            *cur = 0;
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
