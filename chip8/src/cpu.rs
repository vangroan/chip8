//! CPU and memory state.
use crate::{bytecode::*, constants::*};

/// Core state for a chip8 interpreter.
#[allow(dead_code)]
pub struct Chip8Cpu {
    // ------------------------------------------------------------------------
    // Registers

    /// Program counter pointing to the current position in the bytecode.
    pub(crate) pc: usize,
    /// Stack pointer, indicating the top of the stack.
    pub(crate) sp: usize,
    /// General purpose registers for temporary values.
    ///
    /// Register 16 (VF) is used for either the carry flag or borrow switch depending on opcode.
    pub(crate) registers: [u8; 16],
    /// Pointer register used for temporarily storing an address. Since addresses are 12 bits, only the
    /// lowest (rightmost) bits are used.
    pub(crate) address: Address,
    /// Delay timer that counts down to 0.
    pub(crate) delay: u16,
    /// Sound timer that counts down to 0. When it has a non-zero value, a beep is played.
    pub(crate) sound: u16,

    // ------------------------------------------------------------------------
    // Memory

    /// Main memory storage space.
    pub(crate) ram: Box<[u8; MEM_SIZE]>,
    /// Stack of return pointers used for jumping when a routine call finishes.
    pub(crate) stack: Box<[Address; STACK_SIZE]>,
    /// Screen buffer that is drawn too.
    pub(crate) display: Box<[bool; DISPLAY_BUFFER_SIZE]>,

    // ------------------------------------------------------------------------
    // Control

    // pub(crate) trap: bool,
    // pub(crate) error: Option<&str>,
}

impl Default for Chip8Cpu {
    fn default() -> Self {
        Self {
            pc: 0,
            sp: 0,
            registers: [0; 16],
            address: 0,
            delay: 0,
            sound: 0,

            ram: Box::new([0; MEM_SIZE]),
            stack: Box::new([0; STACK_SIZE]),
            display: Box::new([false; DISPLAY_BUFFER_SIZE]),
        }
    }
}

impl Chip8Cpu {
    pub fn new() -> Self {
        Default::default()
    }

    /// Extract opcode from the current program pointer.
    #[inline(always)]
    pub fn op_code(&self) -> u8 {
        op_code(&*self.ram, self.pc)
    }

    /// Extract operand NNN from the current program counter.
    #[inline(always)]
    pub fn op_nnn(&self) -> u16 {
        op_nnn(&*self.ram, self.pc)
    }

    /// Extract operand NN from the current program counter.
    #[inline(always)]
    pub fn op_nn(&self) -> u8 {
        op_nn(&*self.ram, self.pc)
    }

    /// Extract operands VX and NN from the current program counter.
    #[inline(always)]
    pub fn op_xnn(&self) -> (u8, u8) {
        op_xnn(&*self.ram, self.pc)
    }

    /// Extract operands VX, VY and N from the current program counter.
    #[inline(always)]
    pub fn op_xyn(&self) -> (u8, u8, u8) {
        op_xyn(&*self.ram, self.pc)
    }

    /// Extract operands VX, VY and N from the current program counter.
    #[inline(always)]
    pub fn op_xy(&self) -> (u8, u8) {
        op_xy(&*self.ram, self.pc)
    }

    /// Extract operand N from the current program counter.
    #[inline(always)]
    pub fn op_n(&self) -> u8 {
        op_n(&*self.ram, self.pc)
    }
}
