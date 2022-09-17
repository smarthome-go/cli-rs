use std::borrow::Cow::{self, Borrowed, Owned};

use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::Result;
use rustyline::{Cmd, CompletionType, Config, EditMode, Editor, KeyEvent};
use rustyline_derive::{Completer, Helper, Hinter, Validator};
use smarthome_sdk_rs::Client;

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
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

pub async fn start(client: &Client) -> Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .build();

    let h = ReplHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        colored_prompt: "".to_owned(),
        validator: MatchingBracketValidator::new(),
    };

    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(h));
    rl.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);

    if rl.load_history("history.txt").is_err() {
        println!("Created history file");
    }

    loop {
        let username = client.username.clone().unwrap_or_else(|| "e".to_string());
        let hostname = client
            .smarthome_url
            .host()
            .expect("Client can only exist with a valid URL");
        let prompt = format!("{}@{}> ", username, hostname);

        rl.helper_mut().expect("No helper").colored_prompt = format!(
            "\x1b[1;32m{}\x1b[0m@\x1b[1;34m{}\x1b[0m> ",
            username, hostname,
        );

        match rl.readline(&prompt) {
            Ok(line) => {
                // Skip empty lines
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(line.as_str());
                match client.exec_homescript_code(&line, vec![], false).await {
                    Ok(res) => {
                        match res.success {
                            true => print!("{}", res.output),
                            false => eprintln!(
                                "{}",
                                res.errors
                                    .iter()
                                    .map(|err| {
                                        format!(
                                            "{} (l. {}| c. {}): {}",
                                            err.error_type,
                                            err.location.line,
                                            err.location.column,
                                            err.message
                                        )
                                    })
                                    .collect::<Vec<String>>()
                                    .join("\n")
                            ),
                        };
                    }
                    Err(err) => {
                        eprintln!("{:?}", err)
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                eprintln!("Interrupted");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.append_history("history.txt")
}
