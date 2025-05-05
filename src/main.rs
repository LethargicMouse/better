use std::{
    fmt,
    io::{self, Write},
    mem, process,
};

use termion::{
    AsyncReader, async_stdin, clear, cursor,
    event::Key,
    input::{Keys, TermRead},
    raw::{IntoRawMode, RawTerminal},
    terminal_size,
};

fn main() {
    App::run_new().unwrap_or_else(|e| {
        eprint!("{e}");
        process::exit(1)
    })
}

struct Intro {}

impl Intro {
    fn new() -> Self {
        Self {}
    }

    fn manage_key(self, key: Key) -> Frame {
        match key {
            Key::Esc => return Frame::End,
            _ => {}
        }

        Frame::Intro(self)
    }
}

enum Frame {
    End,
    Intro(Intro),
}

impl Frame {
    fn begin() -> Self {
        Self::Intro(Intro::new())
    }

    fn is_end(&self) -> bool {
        matches!(self, Self::End)
    }

    fn draw(&self, out: &mut impl Write) -> Ress<()> {
        let (w, h) = terminal_size()?;
        write!(
            out,
            "{}BetTer{}Better  Terminal",
            cursor::Goto(w / 2 - 3, h / 3), cursor::Goto(w / 2 - 7, h / 3 + 2)
        )?;
        Ok(())
    }

    fn manage_key(self, key: Key) -> Self {
        match self {
            Frame::End => self,
            Frame::Intro(intro) => intro.manage_key(key),
        }
    }
}

struct App {
    write: RawTerminal<io::Stdout>,
    read: Keys<AsyncReader>,
    frame: Frame,
}

impl App {
    fn run_new() -> Ress<()> {
        Self::new()?.run()
    }

    fn new() -> Ress<Self> {
        Ok(Self {
            write: io::stdout().into_raw_mode()?,
            read: async_stdin().keys(),
            frame: Frame::begin(),
        })
    }

    fn run(&mut self) -> Ress<()> {
        self.init()?;
        while !self.frame.is_end() {
            self.update()?;
            self.draw()?;
        }
        self.deinit()?;
        Ok(())
    }

    fn deinit(&mut self) -> Result<(), Error> {
        write!(
            self.write,
            "{}{}{}",
            clear::All,
            cursor::Goto(1, 1),
            cursor::Show
        )?;
        Ok(())
    }

    fn init(&mut self) -> Ress<()> {
        write!(self.write, "{}{}", cursor::Hide, clear::All)?;
        self.write.flush()?;
        Ok(())
    }

    fn update(&mut self) -> Ress<()> {
        if let Some(key) = self.read.next() {
            let frame = mem::replace(&mut self.frame, Frame::End);
            self.frame = frame.manage_key(key?);
        }
        Ok(())
    }

    fn draw(&mut self) -> Ress<()> {
        self.frame.draw(&mut self.write)?;
        self.write.flush()?;
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
