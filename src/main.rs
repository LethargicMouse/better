use std::{
    fmt,
    io::{self, Read, Write},
    process,
};

use termion::{
    clear, cursor,
    raw::{IntoRawMode, RawTerminal},
};

fn main() {
    App::run_new().unwrap_or_else(|e| {
        eprint!("{e}");
        process::exit(1)
    })
}

struct App {
    write: RawTerminal<io::StdoutLock<'static>>,
    read: io::StdinLock<'static>,
}

impl App {
    fn run_new() -> Ress<()> {
        Self::new()?.run()
    }

    fn new() -> Ress<Self> {
        Ok(Self {
            write: io::stdout().lock().into_raw_mode()?,
            read: io::stdin().lock(),
        })
    }

    fn run(&mut self) -> Ress<()> {
        write!(self.write, "{}{}", clear::All, cursor::Hide)?;
        self.write.flush()?;
        loop {
            let mut buf = [b'\0'];
            self.read.read(&mut buf)?;
            match buf[0] {
                b'\x1b' => break,
                c => {
                    write!(self.write, "{}", c as char)?;
                    self.write.flush()?;
                }
            }
        }
        write!(
            self.write,
            "{}{}{}",
            clear::All,
            cursor::Goto(1, 1),
            cursor::Show
        )?;
        Ok(())
    }
}

type Ress<T> = Result<T, Error>;

enum Error {
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "! error: ")?;
        match self {
            Error::Io(error) => writeln!(f, "io: {error}"),
        }
    }
}
