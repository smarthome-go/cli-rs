//! Terminal output: styles, logging and progress spinners.
//!
//! Everything is written through `anstream`, which strips colors when the output is not a
//! terminal, `NO_COLOR` is set or `--color never` was passed.

use std::{fmt::Display, future::Future, time::Duration};

use anstream::{ColorChoice, eprintln};
use anstyle::{AnsiColor, Style};
use indicatif::{ProgressBar, ProgressStyle};
use log::{Level, LevelFilter, Log, Metadata, Record};
use tabled::{
    Table, Tabled,
    settings::{format::Format, object::Rows},
};

pub const HEADER: Style = AnsiColor::Green.on_default().bold();
pub const GOOD: Style = AnsiColor::Green.on_default().bold();
pub const BAD: Style = AnsiColor::Red.on_default().bold();
pub const WARN: Style = AnsiColor::Yellow.on_default().bold();
pub const ACCENT: Style = AnsiColor::Cyan.on_default().bold();
pub const DIM: Style = AnsiColor::BrightBlack.on_default();
pub const BOLD: Style = Style::new().bold();
pub const ADDED: Style = AnsiColor::Green.on_default();
pub const REMOVED: Style = AnsiColor::Red.on_default();
pub const HUNK: Style = AnsiColor::Cyan.on_default();

/// Renders `text` in the given style
pub fn paint(style: Style, text: impl Display) -> String {
    format!("{style}{text}{style:#}")
}

/// Renders rows as a table with a highlighted header
pub fn table<T: Tabled>(rows: impl IntoIterator<Item = T>) -> String {
    let mut table = Table::new(rows);
    table
        .with(tabled::settings::Style::modern().remove_horizontal())
        .modify(Rows::first(), Format::content(|header| paint(HEADER, header)));
    table.to_string()
}

struct Logger {
    level: LevelFilter,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level && metadata.target().starts_with(env!("CARGO_CRATE_NAME"))
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        match record.level() {
            Level::Error => eprintln!("{} {}", paint(BAD, "error:"), record.args()),
            Level::Warn => eprintln!("{} {}", paint(WARN, "warning:"), record.args()),
            Level::Info => eprintln!("{}", record.args()),
            Level::Debug | Level::Trace => eprintln!(
                "{}",
                paint(DIM, format_args!("{}: {}", record.level().as_str().to_lowercase(), record.args()))
            ),
        }
    }

    fn flush(&self) {}
}

/// Status messages go to stderr so that stdout only carries the requested output
pub fn init_logger(verbose: bool) {
    let level = match verbose {
        true => LevelFilter::Trace,
        false => LevelFilter::Info,
    };
    log::set_boxed_logger(Box::new(Logger { level })).expect("The logger is only initialized once");
    log::set_max_level(level);
}

/// Shows a spinner on stderr while `future` runs; hidden when stderr is not a terminal
pub async fn spin<F: Future>(message: impl Into<String>, future: F) -> F::Output {
    let template = match ColorChoice::global() {
        ColorChoice::Never => "{spinner} {msg}",
        _ => "{spinner:.cyan} {msg}",
    };
    let spinner = ProgressBar::new_spinner().with_message(message.into());
    spinner.set_style(ProgressStyle::with_template(template).expect("The template is valid"));
    spinner.enable_steady_tick(Duration::from_millis(80));
    let output = future.await;
    spinner.finish_and_clear();
    output
}
