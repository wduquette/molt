use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::interp::Interp;
use crate::types::Status::*;

pub fn shell(interp: &mut Interp, prompt: &str) {
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if !line.is_empty() {
                    match interp.eval(line) {
                        Okay(value) => {
                            rl.add_history_entry(line);
                            println!("{}", value.as_str());
                        }
                        Error(value) => {
                            println!("{}", value.as_str());
                        }
                        _ => {
                            println!("Unexpected eval return.");
                        }
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                break
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("I/O Error: {:?}", err);
                break
            }
        }
    }
}
