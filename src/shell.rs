use crate::interp::Interp;
use crate::types::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub fn shell(interp: &mut Interp, prompt: &str) {
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if !line.is_empty() {
                    match interp.eval(line) {
                        Ok(value) => {
                            rl.add_history_entry(line);
                            
                            // Don't output empty values.
                            if !value.is_empty() {
                                println!("{}", value);
                            }
                        }
                        Err(ResultCode::Error(msg)) => {
                            println!("{}", msg);
                        }
                        _ => {
                            println!("Unexpected eval return.");
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                break;
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("I/O Error: {:?}", err);
                break;
            }
        }
    }
}
