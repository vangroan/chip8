use chip8_bytecode::{BytecodeInterpreter, Disassembler};
use chip8_core::prelude::*;
use chip8_tree::StaticSimulator;
use std::{error::Error, time::Instant};

const BYTECODE: &[u8] = include_bytes!("../programs/maze");

fn run_bytecode() -> Result<(), Box<dyn Error>> {
    println!("Running Bytecode Interpreter");

    let interpreter = BytecodeInterpreter;
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

fn run_tree() {
    println!("Running Tree Interpreter");

    let interpreter = StaticSimulator::new();
    let mut vm = Chip8Vm::new(interpreter);
    vm.load_bytecode(include_bytes!("../programs/maze"));
}

fn main() -> Result<(), Box<dyn Error>> {
    run_bytecode()?;
    run_tree();

    Ok(())
}
