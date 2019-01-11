use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::Interp;

pub fn shell(interp: &mut Interp, prompt: &str) {
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if !line.is_empty() {
                    match interp.eval(line) {
                        Ok(result) => {
                            rl.add_history_entry(line);
                            println!("{}", result);
                        }
                        Err(msg) => {
                            println!("{}", msg);
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
