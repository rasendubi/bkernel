use crate::led;
use crate::led_music;

use ::futures::{Async, AsyncSink, Future, Poll, Sink, Stream};

use ::breactor::start_send_all_string::StartSendAllString;

const PROMPT: &str = "> ";

const HELP_MESSAGE: &str =
"Available commands:\r
hi      -- welcomes you\r
pony    -- surprise!\r
-3/+3   -- turn off/on LED3\r
-4/+4   -- turn off/on LED4\r
-5/+5   -- turn off/on LED5\r
-6/+6   -- turn off/on LED6\r
led-fun -- some fun with LEDs\r
temp    -- read temperature from HTU21D sensor\r
panic   -- throw a panic\r
help    -- print this help\r
";

macro_rules! log {
    ( $( $x:expr ),* ) => {
        {
            use ::core::fmt::Write;
            let _ = write!(super::log::Logger::new(&super::USART2), $($x),*);
        }
    };
}

// https://raw.githubusercontent.com/mbasaglia/ASCII-Pony/master/Ponies/vinyl-scratch-noglasses.txt
// https://github.com/mbasaglia/ASCII-Pony/
const PONY: &'static str = "\r
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
    Temperature(
        Option<S>,
        ::futures::future::Join<
            ::dev::htu21d::Htu21dCommand<::dev::htu21d::HoldMaster, ::dev::htu21d::Temperature>,
            ::dev::htu21d::Htu21dCommand<::dev::htu21d::HoldMaster, ::dev::htu21d::Humidity>>),
    EchoChar(Option<S>, u8),
    EchoCharStr(u8, StartSendAllString<'static, S>),
    FlushString(StartSendAllString<'static, S>),
    FlushPrompt(StartSendAllString<'static, S>),
}

impl<S> CommandResult<S>
    where S: Sink<SinkItem = u8>
{
    pub fn echo_char(sink: S, c: u8) -> CommandResult<S> {
        match c as char {
            // backspace
            '\u{8}' => CommandResult::EchoCharStr(c, StartSendAllString::new(sink, "\u{8} \u{8}")),
            '\r' => CommandResult::EchoCharStr(c, StartSendAllString::new(sink, "\r\n")),
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

    pub fn temperature(sink: S) -> CommandResult<S> {
        CommandResult::Temperature(Some(sink),
                                   super::HTU21D.read_temperature_hold_master().join(
                                       super::HTU21D.read_humidity_hold_master()))
    }
}

impl<S> Future for CommandResult<S>
    where S: Sink<SinkItem=u8, SinkError=()> + 'static
{
    type Item = S;
    type Error = S::SinkError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            *self = match *self {
                CommandResult::EchoChar(ref mut sink, c) => {
                    if let AsyncSink::NotReady(_) = sink.as_mut().take().unwrap().start_send(c)? {
                        return Ok(Async::NotReady)
                    } else {
                        return Ok(Async::Ready(sink.take().unwrap()))
                    }
                }
                CommandResult::EchoCharStr(c, ref mut f) => {
                    let sink = try_ready!(f.poll());
                    if c == '\r' as u8 {
                        process_enter(sink)
                    } else {
                        return Ok(Async::Ready(sink))
                    }
                }
                CommandResult::Temperature(ref mut sink, ref mut f) => {
                    let res = f.poll();
                    match res {
                        Ok(Async::Ready((temperature, humidity))) => {
                            // TODO: don't use log
                            log!("Temperature: {} C    Humidity: {}%\r\n", temperature, humidity);
                            CommandResult::flush_prompt(sink.take().unwrap())
                        },
                        Ok(Async::NotReady) => {
                            return Ok(Async::NotReady);
                        },
                        Err(err) => {
                            log!("{:?}\r\n", err);
                            CommandResult::flush(sink.take().unwrap(),
                                                 "Temperature read error\r\n")
                        },
                    }
                },
                CommandResult::Sink(ref mut sink) => {
                    return Ok(Async::Ready(
                        sink.take().unwrap()))
                }
                CommandResult::FlushString(ref mut f) => {
                    let sink = try_ready!(f.poll());
                    CommandResult::flush_prompt(sink)
                }
                CommandResult::FlushPrompt(ref mut f) => {
                    let sink = try_ready!(f.poll());
                    return Ok(Async::Ready(sink))
                }
            };
        }
    }
}

/// Starts a terminal.
pub fn run_terminal<St, Si>(stream: St, sink: Si) -> impl Future<Item=Si, Error=()> + 'static
    where St: Stream<Item=u8, Error=()> + 'static,
          Si: Sink<SinkItem=u8, SinkError=()> + 'static
{
    StartSendAllString::new(sink, PROMPT)
        .and_then(|sink| stream.fold(sink, process_char))
}

static mut COMMAND: [u8; 32] = [0; 32];
static mut CUR: usize = 0;

/// Processes one character at a time. Calls `process_command` when
/// user presses Enter or command is too long.
fn process_char<Si>(sink: Si, c: u8) -> impl Future<Item=Si, Error=()> + 'static
    where Si: Sink<SinkItem=u8, SinkError=()> + 'static
{
    let command = unsafe{&mut COMMAND};
    let cur = unsafe{&mut CUR};

    if c == 0x8 { // backspace
        if *cur != 0 {
            *cur -= 1;
        } else {
            // If there is nothing to delete, do nothing
            return CommandResult::sink(sink)
        }
    } else {
        command[*cur] = c;
        *cur += 1;

        if *cur == command.len() {
            // If command length is too long, emulate Enter was pressed
            return CommandResult::echo_char(sink, '\r' as u8)
        }
    }

    CommandResult::echo_char(sink, c)
}

fn process_enter<Si>(sink: Si) -> CommandResult<Si>
    where Si: Sink<SinkItem=u8, SinkError=()> + 'static
{
    let command = unsafe{&mut COMMAND};
    let cur = unsafe{&mut CUR};

    let command = &command[0 .. *cur - 1];
    *cur = 0;

    match command {
        b"help" => CommandResult::flush(sink, HELP_MESSAGE),
        b"hi" => CommandResult::flush(sink, "Hi, there!\r\n"),
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
        b"temp" | b"temperature" => {
            CommandResult::temperature(sink)
        },
        b"panic" => {
            panic!();
        }
        b"" => CommandResult::flush_prompt(sink),
        _ => {
            CommandResult::flush(sink, "Unknown command\r\n")
        },
    }
}
