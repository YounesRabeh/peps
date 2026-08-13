//! Command-line entry point for running `.peps` files.

use std::{env, fs, io::Write, process};

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

    let bytecode = match peps::compile_source(&source) {
        Ok(bytecode) => bytecode,
        Err(diagnostics) => {
            for diagnostic in diagnostics {
                eprintln!("{}", diagnostic.format(Some(path)));
            }
            process::exit(1);
        }
    };
    let result =
        peps::vm::execute_with_input_reader(&bytecode, peps::ExecutionLimit::Unlimited, |kind| {
            eprint!("{}> ", kind.name());
            std::io::stderr()
                .flush()
                .map_err(|error| format!("could not display input prompt: {error}"))?;
            let mut line = String::new();
            std::io::stdin()
                .read_line(&mut line)
                .map_err(|error| format!("could not read input: {error}"))?;
            if line.is_empty() {
                return Err(format!("input ended while reading {}", kind.name()));
            }
            Ok(line.trim_end_matches(['\r', '\n']).to_string())
        });

    match result {
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
