//! Entrypoint for CLI
use std::{env, error::Error, fs, time::Instant};

use chip8::{
    asm::{Assembler, Lexer, TokenKind},
    prelude::*,
};

fn run_bytecode(filepath: impl AsRef<str>) -> Chip8Result<()> {
    println!("Running Bytecode Interpreter");

    let bytecode = fs::read(filepath.as_ref())?;

    let mut vm = Chip8Vm::new(Chip8Conf::default());
    vm.load_bytecode(bytecode.as_slice())?;

    Disassembler::new(bytecode.as_slice()).print_bytecode();

    // println!("{}", vm.dump_ram(include_bytes!("../programs/maze").len())?);

    let start = Instant::now();
    let result = vm.execute();
    let end = Instant::now();

    println!(
        "time taken: {}ms",
        end.duration_since(start).as_nanos() as f64 / 1000000.0
    ); // to millis
    println!("{}", vm.dump_display()?);

    result?;

    Ok(())
}

fn run_assembler(filepath: impl AsRef<str>) -> Chip8Result<()> {
    use TokenKind as TK;

    println!("Running Assembler");

    let file_bytes = fs::read(filepath.as_ref())?;
    let source_code = String::from_utf8(file_bytes)?;

    let mut lexer = Lexer::new(source_code.as_str());

    loop {
        let token = lexer.next_token();

        match token.kind {
            TK::EOF | TK::Newline => println!(
                "{:6}:{} {:?}",
                token.span.index, token.span.size, token.kind
            ),
            _ => println!(
                "{:6}:{} {:?} \"{}\"",
                token.span.index,
                token.span.size,
                token.kind,
                token.span.fragment(lexer.source_code())
            ),
        }

        if matches!(token.kind, TokenKind::EOF) {
            break;
        }
    }

    {
        let lexer = Lexer::new(source_code.as_str());
        let asm = Assembler::new(lexer);

        match asm.parse() {
            Ok(_bytecode) => {}
            Err(err) => eprintln!("{}", err),
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    match parse_args() {
        Some(Cmd::Run { filepath }) => run_bytecode(filepath)?,
        Some(Cmd::Asm { filepath }) => run_assembler(filepath)?,
        None => {}
    }

    Ok(())
}

fn parse_args() -> Option<Cmd> {
    let mut args = env::args().skip(1);
    match args.next() {
        Some(cmd) => {
            // don't format me T.T
            match cmd.as_str() {
                "run" => Some(Cmd::Run {
                    filepath: consume_arg(args)?,
                }),
                "asm" => Some(Cmd::Asm {
                    filepath: consume_arg(args)?,
                }),
                _ => {
                    println!("unknown");
                    print_usage();
                    None
                }
            }
        }
        None => {
            print_usage();
            None
        }
    }
}

/// Consumes the next argument, and prints the usage text if it doesn't exist.
fn consume_arg(mut args: impl Iterator<Item = String>) -> Option<String> {
    match args.next() {
        Some(arg) => Some(arg),
        None => {
            print_usage();
            None
        }
    }
}

fn print_usage() {
    println!("Usage...");
}

enum Cmd {
    /// Run file
    Run { filepath: String },
    /// Assemble
    Asm { filepath: String },
}
