mod bytecode;
mod clock;
pub mod constants;
mod cpu;
mod disasm;
mod error;
mod vm;

pub mod prelude {
    pub use super::{
        cpu::Chip8Cpu,
        disasm::Disassembler,
        error::{Chip8Error, Chip8Result},
        // interp::BytecodeInterp,
        vm::Chip8Vm,
    };
}
