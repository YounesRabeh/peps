use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: peps <file.peps>");
        process::exit(1);
    }

    let path = &args[1];
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("error: could not read {}: {}", path, error);
            process::exit(1);
        }
    };

    match peps::run_source(&source) {
        Ok(output) => {
            for line in output {
                println!("{}", line);
            }
        }
        Err(error) => {
            for line in error.output {
                println!("{}", line);
            }
            for diagnostic in error.diagnostics {
                eprintln!("{}", diagnostic.format(Some(path)));
            }
            process::exit(1);
        }
    }
}
