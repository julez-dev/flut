use crate::prompt::FlutPrompt;
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
            Signal::Success(buffer) => {
                println!("processed: {buffer}");
            }
        }
    }
}
