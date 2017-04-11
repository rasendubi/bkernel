use led;
use led_music;

use futures::{Async, AsyncSink, Future, IntoFuture, Poll, Sink, Stream};

use start_send_all_string::StartSendAllString;

const PROMPT: &'static str = "\r\n> ";

const HELP_MESSAGE: &'static str =
"Available commands:\r
hi      -- welcomes you\r
pony    -- surprise!\r
-3/+3   -- turn off/on LED3\r
-4/+4   -- turn off/on LED4\r
-5/+5   -- turn off/on LED5\r
-6/+6   -- turn off/on LED6\r
led-fun -- some fun with LEDs\r
panic   -- throw a panic\r
help    -- print this help";

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

pub enum CommandResult<S> {
    Sink(Option<S>),
    EchoChar(Option<S>, u8),
    EchoCharStr(StartSendAllString<'static, S>),
    FlushString(StartSendAllString<'static, S>),
    FlushPrompt(StartSendAllString<'static, S>),
}

impl<S> CommandResult<S>
    where S: Sink<SinkItem = u8>
{
    pub fn echo_char(sink: S, c: u8) -> CommandResult<S> {
        match c as char {
            // backspace
            '\u{8}' => CommandResult::EchoCharStr(StartSendAllString::new(sink, "\u{8} \u{8}")),
            _ => CommandResult::EchoChar(Some(sink), c),
        }
    }

    pub fn flush(sink: S, string: &'static str) -> CommandResult<S> {
        CommandResult::FlushString(StartSendAllString::new(sink, string))
    }

    pub fn flush_prompt(sink: S) -> CommandResult<S> {
        CommandResult::FlushPrompt(StartSendAllString::new(sink, PROMPT))
    }

    pub fn sink(sink: S) -> CommandResult<S> {
        CommandResult::Sink(Some(sink))
    }
}

impl<S> Future for CommandResult<S>
    where S: Sink<SinkItem=u8, SinkError=()> + 'static
{
    type Item = S;
    type Error = S::SinkError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let next = match *self {
            CommandResult::EchoChar(ref mut sink, c) => {
                if let AsyncSink::NotReady(_) = try!(sink.as_mut().take().expect("").start_send(c)) {
                    return Ok(Async::NotReady)
                }

                let sink = sink.take().expect("");
                if c == '\r' as u8 {
                    Some(process_enter(sink))
                } else {
                    return Ok(Async::Ready(sink))
                }
            }
            CommandResult::EchoCharStr(ref mut f) => {
                let sink = try_ready!(f.poll());
                return Ok(Async::Ready(sink))
            }
            CommandResult::Sink(ref mut sink) => {
                return Ok(Async::Ready(
                    sink.take().expect("Attempted to poll CommandResult after completion")))
            }
            CommandResult::FlushString(ref mut f) => {
                let sink = try_ready!(f.poll());
                Some(CommandResult::flush_prompt(sink))
            }
            CommandResult::FlushPrompt(ref mut f) => {
                let sink = try_ready!(f.poll());
                return Ok(Async::Ready(sink))
            }
        };

        if let Some(x) = next {
            *self = x;
            self.poll()
        } else {
            Ok(Async::NotReady)
        }
    }
}

/// Starts a terminal.
///
/// Note that terminal is non-blocking and is driven by `put_char`.
pub fn run_terminal<St, Si>(stream: St, sink: Si) -> impl IntoFuture<Item=Si, Error=()> + 'static
    where St: Stream<Item=u8, Error=()> + 'static,
          Si: Sink<SinkItem=u8, SinkError=()> + 'static
{
    ::futures::stream::iter(PROMPT.as_bytes().into_iter().map(|x| Ok(*x) as Result<u8, ()>))
        .forward(sink)
        .and_then(|(_, sink)| {
            stream.fold(sink, |sink, c| {
                process_char(sink, c)
            })
        })
}

static mut COMMAND: [u8; 256] = [0; 256];
static mut CUR: usize = 0;

/// Processes one character at a time. Calls `process_command` when
/// user presses Enter or command is too long.
fn process_char<Si>(sink: Si, c: u8) -> impl IntoFuture<Item=Si, Error=()> + 'static
    where Si: Sink<SinkItem=u8, SinkError=()> + 'static
{
    let c = c as u32;

    let mut command = unsafe{&mut COMMAND};
    let mut cur = unsafe{&mut CUR};

    if c == '\r' as u32 && *cur == 0 {
        return CommandResult::flush_prompt(sink)
    }

    if c == 0x8 { // backspace
        if *cur != 0 {
            *cur -= 1;
        } else {
            return CommandResult::sink(sink)
        }
    } else {
        command[*cur] = c as u8;
        *cur += 1;

        if *cur == 256 {
            *cur = 0;
        }
    }

    CommandResult::echo_char(sink, c as u8)
}

fn process_enter<Si>(sink: Si) -> CommandResult<Si>
    where Si: Sink<SinkItem=u8, SinkError=()> + 'static
{
    let command = unsafe{&mut COMMAND};
    let mut cur = unsafe{&mut CUR};

    let command = &command[0 .. *cur - 1];
    *cur = 0;

    match command {
        b"help" => CommandResult::flush(sink, HELP_MESSAGE),
        b"hi" => CommandResult::flush(sink, "Hi, there!"),
        b"pony" | b"p" => CommandResult::flush(sink, PONY),
        b"-3" => { led::LD3.turn_off(); CommandResult::flush_prompt(sink) },
        b"+3" => { led::LD3.turn_on(); CommandResult::flush_prompt(sink) },
        b"-4" => { led::LD4.turn_off(); CommandResult::flush_prompt(sink) },
        b"+4" => { led::LD4.turn_on(); CommandResult::flush_prompt(sink) },
        b"-5" => { led::LD5.turn_off(); CommandResult::flush_prompt(sink) },
        b"+5" => { led::LD5.turn_on(); CommandResult::flush_prompt(sink) },
        b"-6" => { led::LD6.turn_off(); CommandResult::flush_prompt(sink) },
        b"+6" => { led::LD6.turn_on(); CommandResult::flush_prompt(sink) },
        b"led-fun" => { led_music::led_fun(71000); CommandResult::flush_prompt(sink) },
        b"panic" => {
            panic!();
        }
        b"" => CommandResult::flush_prompt(sink),
        _ => {
            CommandResult::flush(sink, "Unknown command\n")
        },
    }
}
