use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::interp::Interp;

pub fn shell(interp: &mut Interp, prompt: &str) {
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if !line.is_empty() {
                    match interp.eval(line) {
                        Err(()) => {
                            println!("{}", interp.get_result().as_string());
                        }
                        _ => {
                            rl.add_history_entry(line);
                            println!("{}", interp.get_result().as_string());
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
