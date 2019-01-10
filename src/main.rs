use rustyline::error::ReadlineError;
use rustyline::Editor;
use gcl::interp::Interp;

fn main() {
    let mut rl = Editor::<()>::new();
    let mut interp = Interp::new();

    loop {
        let readline = rl.readline("% ");
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
