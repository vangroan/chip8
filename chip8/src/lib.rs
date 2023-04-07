pub mod asm;
mod bytecode;
mod clock;
pub mod constants;
mod cpu;
mod devices;
mod disasm;
mod error;
mod vm;

pub use self::{
    asm::assemble,
    cpu::Chip8Cpu,
    devices::KeyCode,
    error::{Chip8Error, Chip8Result},
    vm::Hz,
    vm::{Chip8Conf, Chip8Vm},
};

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
