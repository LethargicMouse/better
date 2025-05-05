use std::{
    fmt,
    fs::DirEntry,
    io::{self, Write},
    mem,
    path::Path,
    process,
};

use termion::{
    AsyncReader, async_stdin, clear, color, cursor,
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

    fn manage_key(self, key: Key) -> Ress<Frame> {
        match key {
            Key::Esc => return Ok(Frame::End),
            Key::Char('f') => return Ok(Frame::Dir(Dir::new(".")?)),
            _ => {}
        }

        Ok(Frame::Intro(self))
    }

    fn draw(&self, out: &mut impl Write) -> Ress<()> {
        let (w, h) = terminal_size()?;
        write!(
            out,
            "{}BetTer{}Better  Terminal",
            cursor::Goto(w / 2 - 3, h / 3),
            cursor::Goto(w / 2 - 7, h / 3 + 2)
        )?;
        Ok(())
    }
}

struct Dir {
    entries: Vec<DirEntry>,
    cursor: usize,
}

impl Dir {
    fn new(path: impl AsRef<Path>) -> Ress<Self> {
        let entries = std::fs::read_dir(path)?.collect::<Result<_, _>>()?;
        Ok(Self { entries, cursor: 0 })
    }

    fn manage_key(mut self, key: Key) -> Ress<Frame> {
        match key {
            Key::Esc => return Ok(Frame::Intro(Intro::new())),
            Key::Char('j') => self.cursor = (self.cursor + 1) % self.entries.len(),
            Key::Char('k') => {
                self.cursor = (self.cursor + self.entries.len() - 1) % self.entries.len()
            }
            Key::Char('\n') => return Ok(Frame::Dir(Dir::new(self.entries[self.cursor].path())?)),
            _ => {}
        }
        Ok(Frame::Dir(self))
    }

    fn draw(&self, out: &mut impl Write) -> Ress<()> {
        write!(out, "{}", cursor::Goto(1, 2))?;
        for (i, entry) in self.entries.iter().enumerate() {
            if i == self.cursor {
                write!(out, "{}", color::Fg(color::LightGreen))?;
            }
            write!(
                out,
                "   {}{}\n\r",
                entry.path().display(),
                if entry.path().is_dir() { "/" } else { "" }
            )?;
            if i == self.cursor {
                write!(out, "{}", color::Fg(color::Reset))?;
            }
        }
        Ok(())
    }
}

enum Frame {
    End,
    Intro(Intro),
    Dir(Dir),
}

impl Frame {
    fn begin() -> Self {
        Self::Intro(Intro::new())
    }

    fn is_end(&self) -> bool {
        matches!(self, Self::End)
    }

    fn draw(&self, out: &mut impl Write) -> Ress<()> {
        match self {
            Frame::End => Ok(()),
            Frame::Intro(intro) => intro.draw(out),
            Frame::Dir(dir) => dir.draw(out),
        }
    }

    fn manage_key(self, key: Key) -> Ress<Self> {
        match self {
            Frame::End => Ok(self),
            Frame::Intro(intro) => intro.manage_key(key),
            Frame::Dir(dir) => Ok(dir.manage_key(key)?),
        }
    }
}

struct App {
    write: RawTerminal<io::Stdout>,
    read: Keys<AsyncReader>,
    frame: Frame,
    timer: u16,
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
            timer: 1,
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
            self.frame = frame.manage_key(key?)?;
        }
        self.timer -= 1;
        if self.timer == 0 {
            write!(self.write, "{}", clear::All)?;
            self.timer = 200;
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
