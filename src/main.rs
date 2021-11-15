extern crate lazy_static;

mod scanner;
mod token;
mod ast;

use scanner::Scanner;
use std::{fs, env, process};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::num::NonZeroUsize;

static HAD_ERROR: AtomicBool = AtomicBool::new(false);

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rlox [file]");
        process::exit(0);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run(input: &str) {
    let mut scanner = Scanner::new(&input);
    let tokens = scanner.scan_tokens();

    for t in tokens {
        println!("{:?}", t);
    }
}

 fn run_file(name: &str) {
    let file = fs::read_to_string(name).unwrap();
    run(&file);

    if had_error() { process::exit(65); }
}

 fn run_prompt() {
    loop {
        print!("> ");

        let mut input = String::new();
        io::stdout().flush().unwrap();

        match io::stdin().read_line(&mut input) {
            Ok(ln) => {
                match ln {
                    1 => break,
                    _ => run(&input),
                }
                set_had_error(false);
            }
            Err(err) => {
                println!("{}", err);
                process::exit(1);
            }
        }
    }
}

pub fn error(line: NonZeroUsize, message: &str) {
    report(line, message, "");
}

fn report(line: NonZeroUsize, message: &str, where_: &str) {
    println!("[line {}] Error{}: {}", line, where_, message);
    set_had_error(true);
}

fn had_error() -> bool {
    HAD_ERROR.load(Ordering::Relaxed)
}

fn set_had_error(b: bool) {
    HAD_ERROR.store(b, Ordering::Relaxed);
}
