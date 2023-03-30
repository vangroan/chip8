//! Entrypoint for CLI
use std::{env, error::Error, fs, io::Write, time::Instant};

use chip8::{
    asm::{Assembler, Lexer, TokenKind},
    constants::*,
    prelude::*,
    IMPL_VERSION,
};
use log::{error, info};

static USAGE: &str = r#"
usage: chip8 CMD [FILE]

commands:
    run     Run the target ROM file
    asm     Compile the target assembly file into a ROM
    dis     Disassemble the the target ROM into readable assembly

examples:
    chip8 run breakout.rom
    chip8 asm breakout.asm
    chip8 dis breakout.rom
"#;

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

    info!("running Assembler");

    let file_bytes = fs::read(filepath.as_ref())?;
    let source_code = String::from_utf8(file_bytes)?;

    let mut lexer = Lexer::new(source_code.as_str());

    println!("offset | len | token       | fragment ");
    loop {
        let token = lexer.next_token();

        match token.kind {
            TK::EOF | TK::Newline => println!(
                "{0:7}:{1: <3} {2: <16?}",
                token.span.index, token.span.size, token.kind
            ),
            _ => {
                let offset = token.span.index;
                let len = token.span.size;
                let kind = format!("{:?}", token.kind); // cannot format debug print {:?} into columns
                let fragment = token.span.fragment(lexer.source_code());
                println!("{offset:7}:{len: <3} {kind: <20} \"{fragment}\"")
            }
        }

        if matches!(token.kind, TokenKind::EOF) {
            break;
        }
    }

    {
        let lexer = Lexer::new(source_code.as_str());
        let asm = Assembler::new(lexer);

        match asm.parse() {
            Ok(bytecode) => {
                let mut outfile = fs::File::create("output.rom")?;
                outfile.write_all(&bytecode)?;
                dump_bytecode(&bytecode)
            }
            Err(err) => {
                error!("assembly error\n{err}");
                // Exit process with error
                return Err(err);
            }
        }
    }

    Ok(())
}

fn dump_bytecode(bytecode: &[u8]) {
    // Instructions are always 2 bytes.
    assert!(bytecode.len() % 2 == 0);

    for (i, instr) in bytecode.chunks(2).enumerate() {
        let offset = MEM_START + i * 2;
        let a = instr[0];
        let b = instr[1];
        println!("0x{offset:04X} {a:02X}{b:02X}");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    match parse_args() {
        Some(Cmd::Run { filepath }) => run_bytecode(filepath)?,
        Some(Cmd::Asm { filepath }) => run_assembler(filepath)?,
        None => {
            print_usage();
            // FreeBSD EX_USAGE (64)
            std::process::exit(64)
        }
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
                "dis" => todo!("disassembling"),
                _ => None,
            }
        }
        None => None,
    }
}

/// Consumes the next argument, and prints the usage text if it doesn't exist.
fn consume_arg(mut args: impl Iterator<Item = String>) -> Option<String> {
    match args.next() {
        Some(arg) => Some(arg),
        None => None,
    }
}

fn print_usage() {
    println!("Chip8 v{IMPL_VERSION}");
    println!("{USAGE}");
}

enum Cmd {
    /// Run file
    Run { filepath: String },
    /// Assemble
    Asm { filepath: String },
}
