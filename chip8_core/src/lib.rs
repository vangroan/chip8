/// Core Chip8 state common to all interpreter types.
use std::fmt::Write as FmtWrite;

pub mod prelude {
    pub use super::{Chip8Cpu, Chip8Vm, Interpreter};
}

/// The lower memory space was historically used for the interpreter itself,
/// but is now used for fonts.
pub const MEM_START: usize = 0x200;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const SCREEN_BUF_COUNT: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

/// Type for storing the 12-bit memory addresses.
pub type Address = u16;

/// Core state for a chip8 interpreter.
pub struct Chip8Cpu {
    /// Program counter pointing to the current position in the bytecode.
    pub pc: usize,
    /// General purpose registers for temporary values.
    ///
    /// Register 16 (VF) is used for either the carry flag or borrow switch depending on opcode.
    pub registers: [u8; 16],
    /// Pointer register used for temporarily storing an address. Since addresses are 12 bits, only the
    /// lowest (rightmost) bits are used.
    pub address: Address,
    /// Stack of return pointers used for jumping when a routine call finishes.
    pub stack: [Address; 12],
    /// Delay timer that counts down to 0.
    pub delay: u16,
    /// Sound timer that counts down to 0. When it has a non-zero value, a beep is played.
    pub sound: u16,
    /// Main memory storage space.
    pub ram: [u8; 4096],
    /// Screen buffer that is drawn too.
    pub screen: [bool; SCREEN_BUF_COUNT],
}

impl Default for Chip8Cpu {
    fn default() -> Self {
        Self {
            pc: 0,
            registers: [0; 16],
            address: 0,
            stack: [0; 12],
            delay: 0,
            sound: 0,
            ram: [0; 4096],
            screen: [false; SCREEN_BUF_COUNT],
        }
    }
}

impl Chip8Cpu {}

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
            cpu: Chip8Cpu::default(),
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

    /// Returns the contents of the memory as a human readable string.
    pub fn dump_ram(&self, count: usize) -> Result<String, std::fmt::Error> {
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

    pub fn dump_display(&self) -> Result<String, std::fmt::Error> {
        let mut buf = String::new();

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                if self.cpu.screen[x + y * SCREEN_WIDTH] {
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
