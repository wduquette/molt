use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    // TODO: Move this to Interp::shell()
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                if !line.trim().is_empty() {
                    println!("Line: {}", line.trim());
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
