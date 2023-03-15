//! Entrypoint for CLI

use std::{error::Error, time::Instant};

use chip8::prelude::*;

const BYTECODE: &[u8] = include_bytes!("../../programs/maze");

fn run_bytecode() -> Chip8Result<()> {
    println!("Running Bytecode Interpreter");

    let mut vm = Chip8Vm::new();
    vm.load_bytecode(BYTECODE)?;

    Disassembler::new(BYTECODE).print_bytecode();

    // println!("{}", vm.dump_ram(include_bytes!("../programs/maze").len())?);

    let start = Instant::now();
    let result = vm.execute();
    let end = Instant::now();

    println!(
        "time taken: {}ms",
        end.duration_since(start).as_nanos() as f64 / 1000000.0
    ); // to millis
    println!("{}", vm.dump_display()?);

    result
}

fn main() -> Result<(), Box<dyn Error>> {
    run_bytecode()?;

    Ok(())
}
