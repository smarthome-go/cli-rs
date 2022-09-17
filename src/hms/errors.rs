use std::fmt::Display;

pub enum Error {
    Rustyline(rustyline::error::ReadlineError),
}

impl From<rustyline::error::ReadlineError> for Error {
    fn from(err: rustyline::error::ReadlineError) -> Self {
        Self::Rustyline(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Rustyline(err) => format!("REPL error: {err}"),
            }
        )
    }
}
