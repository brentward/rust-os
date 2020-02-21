use std;
use stack_vec::StackVec;
use console::{kprint, kprintln, CONSOLE};
use pi::timer;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

const CR: u8 = b'\r';
const LF: u8 = b'\n';
const BELL: u8 = 7;
const BACK: u8 = 8;
const DEL: u8 = 127;

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str) -> ! {
    let init_msg = "Press any key to continue...";
    kprint!("\r\n");
    loop {
        kprint!("{}", init_msg);
        let mut console = CONSOLE.lock();
        let byte = console.read_byte();
        match byte {
            byte if byte >= 32 && byte <= 127 => {
                break;
            },
            _ => {
                for _ in 0..init_msg.len() {
                    console.write_byte(BACK);
                }
            }
        }
    }

    kprintln!("\r\n\r\nWelcome to the shell!");

    loop {
        kprint!("{}", prefix);
        let mut input_buf = [0u8; 512];
        let mut input = StackVec::new(&mut input_buf);
        'read_char: loop {
            let byte = CONSOLE.lock().read_byte();
            match byte {
                DEL | BACK => {
                    if !input.is_empty() {
                        input.pop();
                        CONSOLE.lock().write_byte(BACK);
                        kprint!(" ");
                        CONSOLE.lock().write_byte(BACK);
                    } else {
                        CONSOLE.lock().write_byte(BELL);
                    }
                }
                CR | LF => break 'read_char,
                byte if byte < 32 || byte > 127 => CONSOLE.lock().write_byte(BELL),
                byte => {
                    if input.push(byte).is_ok() {
                        CONSOLE.lock().write_byte(byte);
                    } else {
                        CONSOLE.lock().write_byte(BELL);
                    }
                }
            }
        }
        kprintln!("");
        let input_str = std::str::from_utf8(input.as_slice())
            .expect("input bytes failed to cast back to string");
        let mut args_buf = [""; 64];
        match Command::parse(input_str, &mut args_buf) {
            Ok(command) => {
                match command.path() {
                    "echo" => echo(&command.args),
                    path => kprintln!("unknown command: {}", path)
                }
            } // TODO execute command
            Err(Error::TooManyArgs) => {
                kprintln!("error: too many arguments");
            }
            Err(Error::Empty) => (),
        }
    }
}

fn echo(args: &StackVec<&str>) {
    for &arg in args[1..].iter() {
        kprint!("{} ", arg);
    }
    kprint!("\r\n");
}