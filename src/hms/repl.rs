use std::borrow::Cow::{self, Borrowed, Owned};
use std::{env, io};

use anstream::{ColorChoice, eprintln, println};
use log::{error, info};
use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter, MatchingBracketHighlighter};
use rustyline::hint::HistoryHinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, CompletionType, Config, EditMode, Editor, KeyEvent};
use rustyline_derive::{Completer, Helper, Hinter, Validator};
use smarthome_sdk_rs::{Client, HmsRunMode};

use super::errors::{Error, Result, render_diagnostics};
use crate::term::{self, ACCENT, BOLD, GOOD, paint};

#[derive(Helper, Completer, Hinter, Validator)]
struct ReplHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Highlighter for ReplHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        match default {
            true => Borrowed(&self.colored_prompt),
            false => Borrowed(prompt),
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(paint(BOLD, hint))
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

pub async fn start(client: &Client, use_history: bool) -> Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .color_mode(match ColorChoice::global() {
            ColorChoice::Never => rustyline::ColorMode::Disabled,
            ColorChoice::Always | ColorChoice::AlwaysAnsi => rustyline::ColorMode::Forced,
            ColorChoice::Auto => rustyline::ColorMode::Enabled,
        })
        .build();

    let username = client.username.clone().unwrap_or_else(|| "e".to_string());
    let hostname = client
        .smarthome_url
        .host_str()
        .expect("Client can only exist with a valid URL")
        .to_string();

    let helper = ReplHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter::new(),
        colored_prompt: format!("{}@{}> ", paint(GOOD, &username), paint(ACCENT, &hostname)),
        validator: MatchingBracketValidator::new(),
    };

    let mut rl: Editor<ReplHelper, DefaultHistory> = Editor::with_config(config)?;
    rl.set_helper(Some(helper));
    rl.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);

    let hist_path = match use_history {
        true => Some(hist_file_path().ok_or_else(|| {
            Error::IO(io::Error::new(
                io::ErrorKind::NotFound,
                "could not determine the history file location: is $HOME set?",
            ))
        })?),
        false => None,
    };
    if let Some(hist_path) = &hist_path
        && rl.load_history(hist_path).is_err()
    {
        info!("Created a new REPL history file at `{hist_path}`");
    }

    let prompt = format!("{username}@{hostname}> ");
    loop {
        match rl.readline(&prompt) {
            Ok(line) => {
                // Skip empty lines
                if line.trim().is_empty() {
                    continue;
                }
                rl.add_history_entry(line.as_str())?;

                match term::spin(
                    "Running",
                    client.exec_homescript_code(&line, vec![], HmsRunMode::Execute),
                )
                .await
                {
                    Ok(res) => {
                        if !res.output.is_empty() {
                            println!("{}", res.output.trim_end());
                        }
                        if !res.success {
                            eprintln!("{}", render_diagnostics(&res.errors, &line, &res.file_contents));
                        }
                    }
                    Err(err) => error!("{err}"),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => return Err(err.into()),
        }
    }

    if let Some(hist_path) = &hist_path {
        rl.append_history(hist_path)?;
    }
    Ok(())
}

pub fn hist_file_path() -> Option<String> {
    match env::var("XDG_CACHE_HOME") {
        Ok(xdg_cache) => Some(format!("{xdg_cache}/smarthome.history")),
        Err(_) => Some(format!("{}/.cache/smarthome.history", env::var("HOME").ok()?)),
    }
}
