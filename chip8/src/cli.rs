//! Entrypoint for CLI
mod bytecode;
pub mod constants;
mod cpu;
mod disasm;
mod interp;
mod vm;

use std::{error::Error, time::Instant};

use self::{disasm::Disassembler, interp::BytecodeInterp, vm::Chip8Vm};

const BYTECODE: &[u8] = include_bytes!("../programs/maze");

fn run_bytecode() -> Result<(), Box<dyn Error>> {
    println!("Running Bytecode Interpreter");

    let interpreter = BytecodeInterp;
    let mut vm = Chip8Vm::new(interpreter);
    vm.load_bytecode(BYTECODE);

    Disassembler::new(BYTECODE).print_bytecode();

    // println!("{}", vm.dump_ram(include_bytes!("../programs/maze").len())?);

    let start = Instant::now();
    vm.execute();
    let end = Instant::now();

    println!(
        "time taken: {}ms",
        end.duration_since(start).as_nanos() as f64 / 1000000.0
    ); // to millis
    println!("{}", vm.dump_display()?);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    run_bytecode()?;

    Ok(())
}
