pub mod asm;
mod bytecode;
mod clock;
pub mod constants;
mod cpu;
mod disasm;
mod error;
mod vm;

pub use self::{asm::assemble, vm::Hz};

/// Version of *this* implementation.
pub const IMPL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod prelude {
    pub use super::{
        cpu::Chip8Cpu,
        disasm::Disassembler,
        error::{Chip8Error, Chip8Result},
        vm::{Chip8Conf, Chip8Vm},
    };
}
