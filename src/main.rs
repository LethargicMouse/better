use std::{
    collections::VecDeque,
    fmt,
    io::{self, Write},
    process,
    str::Chars,
};

use termion::{
    AsyncReader, async_stdin, clear, cursor,
    event::Key,
    input::{Keys, TermRead},
    raw::{IntoRawMode, RawTerminal},
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
}

#[derive(Clone, Copy)]
enum CommandExpr {
    Command,
}

struct Command {
    code: String,
    expected: VecDeque<CommandExpr>,
    args: fn(char) -> &'static [CommandExpr],
}

impl Command {
    fn new(args: fn(char) -> &'static [CommandExpr]) -> Self {
        Self {
            code: String::new(),
            expected: [CommandExpr::Command].into(),
            args,
        }
    }

    fn add(&mut self, c: char) {
        match self
            .expected
            .pop_front()
            .expect("command should not be ready")
        {
            CommandExpr::Command => {
                self.code.push(c);
                self.expected.extend((self.args)(c));
            }
        }
    }

    fn is_ready(&self) -> bool {
        self.expected.is_empty()
    }

    fn reset(&mut self) {
        self.code.clear();
        self.expected.clear();
        self.expected.push_back(CommandExpr::Command);
    }
}

struct App {
    write: RawTerminal<io::Stdout>,
    read: Keys<AsyncReader>,
    frame: Frame,
    command: Command,
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
            command: Command::new(Self::command_args),
        })
    }

    fn command_args(c: char) -> &'static [CommandExpr] {
        match c {
            _ => &[],
        }
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
            match key? {
                Key::Esc => self.frame = Frame::End,
                Key::Char(c) => self.command.add(c),
                _ => {}
            }
        }
        if self.command.is_ready() {
            if self.frame_active
            self.process(self.command.code.chars());
            self.command.reset();
        }
        Ok(())
    }

    fn draw(&mut self) -> Ress<()> {
        self.draw_command()?;
        self.write.flush()?;
        Ok(())
    }

    fn draw_command(&mut self) -> Ress<()> {
        write!(self.write, "{}{}", cursor::Goto(1, 1), self.command.code)?;
        Ok(())
    }

    fn process(&self, mut command: Chars) {
        match command.next() {
            _ => {}
        }
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
