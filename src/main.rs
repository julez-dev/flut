use crate::{lexer::Token, prompt::FlutPrompt};
use reedline::{Reedline, Signal};

mod lexer;
mod prompt;

fn main() -> anyhow::Result<()> {
    let mut line_editor = Reedline::create();
    let prompt = FlutPrompt;

    loop {
        let sig = line_editor.read_line(&prompt)?;

        match sig {
            Signal::CtrlC | Signal::CtrlD => {
                break Ok(());
            }
            Signal::Success(buffer) => match lexer::lex(&buffer) {
                Ok(tokens) => _ = tokens,
                Err(err) => {
                    let report = miette::Report::new(err).with_source_code(buffer);
                    println!("{report:?}");
                }
            },
        }
    }
}
