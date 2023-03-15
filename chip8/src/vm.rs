//! Virtual machine.
use std::fmt::Write;

use crate::constants::*;
use crate::cpu::Chip8Cpu;

pub trait Interpreter {
    /// Called after bytecode has been loaded into the VM memory.
    fn on_load(&mut self, cpu: &Chip8Cpu);
    /// Executes the bytecode stored in VM memory.
    fn execute(&self, cpu: &mut Chip8Cpu);
}

pub struct Chip8Vm<T: Interpreter> {
    cpu: Chip8Cpu,
    interp: T,
}

impl<T: Interpreter> Chip8Vm<T> {
    pub fn new(interpreter: T) -> Self {
        Chip8Vm {
            cpu: Chip8Cpu::new(),
            interp: interpreter,
        }
    }

    pub fn load_bytecode(&mut self, bytecode: &[u8]) {
        let count = bytecode.len().min(4096 - MEM_START);
        for i in 0..count {
            self.cpu.ram[MEM_START + i] = bytecode[i];
        }

        // Reset the program counter to prepare for execution.
        self.cpu.pc = MEM_START;

        // Call inner interpreter for implementation specific preparation.
        self.interp.on_load(&self.cpu);
    }

    pub fn execute(&mut self) {
        self.interp.execute(&mut self.cpu);
    }
}

/// Troubleshooting
#[allow(dead_code)]
impl<T: Interpreter> Chip8Vm<T> {
    /// Returns the contents of the memory as a human readable string.
    pub(crate) fn dump_ram(&self, count: usize) -> Result<String, std::fmt::Error> {
        let iter = self
            .cpu
            .ram
            .iter()
            .enumerate()
            .skip(MEM_START)
            .take(count)
            .step_by(2);
        let mut buf = String::new();

        for (i, op) in iter {
            writeln!(buf, "{:04X}: {:02X}{:02X}", i, op, self.cpu.ram[i + 1])?;
        }

        Ok(buf)
    }

    pub(crate) fn dump_display(&self) -> Result<String, std::fmt::Error> {
        let mut buf = String::new();

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                if self.cpu.display[x + y * DISPLAY_WIDTH] {
                    write!(buf, "#")?;
                } else {
                    write!(buf, ".")?;
                }
            }
            write!(buf, "\n")?;
        }

        Ok(buf)
    }
}
