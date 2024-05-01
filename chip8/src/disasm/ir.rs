//! Intermediate representation for disassembling.
//!
//! This is a structured representation of a program's bytecode.
//! It models control flow and explicitly separates code and data.
use crate::constants::Address;
use std::fmt;
use std::fmt::Formatter;

pub struct Instr {
    /// Index in the buffer where the instruction was read from.
    pub index: usize,
    /// Address in memory where the instruction is located.
    pub addr: Address,
    /// The original bytes that were dead from the buffer.
    pub bytes: [u8; 2],
    pub op: Op,
}

impl Instr {
    /// Original bytes encoded into a `u16`.
    #[inline(always)]
    pub fn bytecode(&self) -> u16 {
        ((self.bytes[0] as u16) << 8) | (self.bytes[1] as u16)
    }

    #[inline(always)]
    pub fn repr(&self) -> InstrRepr {
        InstrRepr { instr: self }
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Op {
    /// 0000
    ///
    /// No operation. Not part of the specification,
    /// but dead space needs to be handled by the
    /// disassembler.
    NoOp,
    /// 00E0 (CLS)
    ///
    /// Clear the screen.
    ClearScreen,
    /// 00EE (RET)
    ///
    /// Return from the sub-routine.
    Return,
    /// 1nnn (JP addr)
    ///
    /// Jump to the address in `nnn`.
    JumpAddress {
        address: Address,
    },
    /// 2nnn (CALL addr)
    ///
    /// Call the sub-routine at address `nnn`.
    Call {
        address: Address,
    },
    /// 3xnn (SE Vx, byte)
    ///
    /// Skip the next instruction if register `Vx` equals value `nn`
    Skip_Eq_Byte {
        vx: u8,
        nn: u8,
    },
    /// 4xnn (SNE Vx, byte)
    ///
    /// Skip the next instruction if register `Vx` does not equal value `nn`.
    Skip_NotEq_Byte {
        vx: u8,
        nn: u8,
    },
    /// 5xy0 (SE Vx, Vy)
    ///
    /// Skip the next instruction if register `Vx` does not equal register `Vy`.
    Skip_Eq {
        vx: u8,
        vy: u8,
    },
    /// 6xnn (LD Vx, byte)
    Load_Byte {
        vx: u8,
        nn: u8,
    },
    /// 7xnn (ADD Vx, byte)
    ///
    /// Add byte to the value in register `Vx`, store the result in `Vx`.
    Add_Byte {
        vx: u8,
        nn: u8,
    },

    // ------------------------------------------------------------------------
    // Math
    /// 8xy0 (LD Vx, byte)
    ///
    /// Store the value of register VY in register VX.
    Load_Vx_Vy {
        vx: u8,
        vy: u8,
    },
    /// 8xy1 (OR Vx, Vy)
    ///
    /// Performs bitwise OR on VX and VY, and stores the result in VX.
    Or_Vx_Vy {
        vx: u8,
        vy: u8,
    },
    /// 8xy2 (AND Vx, Vy)
    ///
    /// Performs bitwise AND on VX and VY, and stores the result in VX.
    And_Vx_Vy {
        vx: u8,
        vy: u8,
    },
    /// 8xy3 (XOR Vx, Vy)
    ///
    /// Performs bitwise XOR on VX and VY, and stores the result in VX.
    Xor_Vx_Vy {
        vx: u8,
        vy: u8,
    },
    /// 8xy4 (ADD Vx, Vy)
    ///
    /// ADDs VX to VY, and stores the result in VX.
    /// Overflow is wrapped. If overflowed, set VF to 1, else 0.
    Add_Vx_Vy {
        vx: u8,
        vy: u8,
    },
    /// 8xy5 (SUB Vx, Vy)
    ///
    /// Subtracts VY from VX, and stores the result in VX.
    /// VF is set to 0 when there is a borrow, set to 1 when there isn't.
    Sub_Vx_Vy {
        vx: u8,
        vy: u8,
    },
    /// 8xy6 (SHR Vx)
    ///
    /// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
    /// Shift VX right by 1. VY is unused.
    ShiftRight {
        vx: u8,
    },
    /// 8xy7 (SUBN Vx, Vy)
    ///
    /// Subtracts VX from VY, and stores the result in VX.
    /// VF is set to 0 when there is a borrow, set to 1 when there isn't.
    SubReverse_Vx_Vy {
        vx: u8,
        vy: u8,
    },
    /// 8xyE (SHL Vx)
    ///
    /// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
    /// Shift VX left by 1.
    /// VY is unused.
    ShiftLeft {
        vx: u8,
    },

    /// Annn (LD I, addr)
    ///
    /// Load address into register `I`.
    Load_Address {
        address: u16,
    },
    // Bnnn (JP V0, addr)
    //
    // Jump to location nnn + V0.
    Jump_Vx {
        address: u16,
    },
    /// Cxnn (RND Vx, byte)
    ///
    /// Generate random number.
    Random {
        vx: u8,
        nn: u8,
    },
    /// Dxyn (DRW Vx, Vy, byte)
    ///
    /// Draw sprite to the display buffer.
    Draw {
        vx: u8,
        vy: u8,
        n: u8,
    },

    // ------------------------------------------------------------------------
    // Meta ops
    Data,
    /// Data region that is drawn to the display.
    Sprite,
    Unknown,
}

pub struct InstrRepr<'a> {
    instr: &'a Instr,
}

// TODO: Pass print settings into a function that formats the whole bytecode buffer (not just one instruction)
#[derive(Debug)]
pub struct InstrReprSettings {
    /// Print the index of the instruction in the original bytecode slice.
    pub print_index: bool,
    /// Print the memory address where the instruction will be loaded in
    /// the virtual machine.
    pub print_address: bool,
    /// Print the original bytecode instruction as hex.
    pub print_original_bytecode: bool,
    /// Interpret each instruction as a pseudocode statement.
    /// Printed as a trailing comment.
    pub print_comment: bool,
    /// Interpret addresses as labels.
    pub print_labels: bool,
}

impl<'a> fmt::Display for InstrRepr<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Instr {
            bytes: [a, b], op, ..
        } = self.instr;

        match op {
            Op::NoOp => Ok(()),
            Op::ClearScreen => write!(f, "CLS"),
            Op::Return => write!(f, "RET"),
            // TODO: Replace with label
            Op::JumpAddress { address } => write!(f, "JP 0x{address:03X}"),
            // TODO: Replace with label
            Op::Call { address } => write!(f, "CALL 0x{address:03X}"),
            Op::Skip_Eq_Byte { vx, nn } => write!(f, "SE v{vx}, {nn}"),
            Op::Skip_NotEq_Byte { vx, nn } => write!(f, "SNE v{vx}, {nn}"),
            Op::Skip_Eq { vx, vy } => write!(f, "SE v{vx}, v{vy}"),
            Op::Load_Byte { vx, nn } => write!(f, "LD v{vx}, {nn}"),
            Op::Add_Byte { vx, nn } => write!(f, "ADD v{vx} {nn}"),
            // ------
            Op::Load_Vx_Vy { vx, vy } => write!(f, "LD v{vx}, v{vy}"),
            Op::Or_Vx_Vy { vx, vy } => write!(f, "OR v{vx}, v{vy}"),
            Op::And_Vx_Vy { vx, vy } => write!(f, "AND v{vx}, v{vy}"),
            Op::Xor_Vx_Vy { vx, vy } => write!(f, "XOR v{vx}, v{vy}"),
            Op::Add_Vx_Vy { vx, vy } => write!(f, "ADD v{vx}, v{vy}"),
            Op::Sub_Vx_Vy { vx, vy } => write!(f, "SUB v{vx}, v{vy}"),
            Op::ShiftRight { vx } => write!(f, "SHR v{vx}"),
            Op::SubReverse_Vx_Vy { vx, vy } => write!(f, "SUBN v{vx}, v{vy}"),
            Op::ShiftLeft { vx } => write!(f, "SHL v{vx}"),
            // ------
            Op::Load_Address { address } => write!(f, "LD I, 0x{address:03X}"),
            Op::Jump_Vx { address } => write!(f, "JP 0x{address:03X}"),
            Op::Random { vx, nn } => write!(f, "RND v{vx}, {nn}"),
            Op::Draw { vx, vy, n } => write!(f, "DRW v{vx}, v{vy}, {n}"),

            Op::Data => write!(f, "0b{a:08b} 0b{b:08b}"),
            Op::Unknown => write!(f, "0x{a:02X}{b:02X}"),
            _ => todo!(),
        }
    }
}
