use log::trace;

/// Helpers for extracting data from opcodes.
use crate::constants::*;

#[rustfmt::skip]
pub mod opcodes {
    /// 00E0 (CLS)
    pub const CLS: u8        = 0xE0;
    /// 00EE (RET)
    pub const RET: u8        = 0xEE;
    /// 1NNN (JP addr)
    pub const JP_ADDR: u8    = 0x1;
    /// 2NNN (CALL addr)
    pub const CALL_ADDR: u8  = 0x2;
    /// 3XNN (SE Vx, byte)
    pub const SE_VX_NN: u8   = 0x3;
    /// Load (LD Vx, byte)
    pub const LD_VX_NN: u8   = 0x6;
    /// 7XNN (ADD Vx, byte)
    pub const ADD_VX_NN: u8  = 0x7;
    /// ANNN (LD I, addr)
    pub const LD_I_NNN: u8   = 0xA;
    /// BNNN (JP V0, addr)
    pub const JP_V0_ADDR: u8 = 0xB;
    /// CXNN (RND Vx, byte)
    pub const RND_X_NN: u8   = 0xC;
    /// DXYN (DRW Vx, Vy, byte)
    pub const DRW_X_Y_N: u8  = 0xD;
}

/// Returns true if the program can fit in VM memory.
#[inline]
pub(crate) fn check_program_size(program: &[u8]) -> bool {
    program.len() <= (MEM_SIZE - MEM_START)
}

/// Extract opcode from the buffer at the cursor.
#[inline(always)]
pub fn op_code(bytecode: &[u8], cursor: usize) -> u8 {
    (bytecode[cursor] & 0b1111_0000) >> 4
}

/// Extract operand NNN from the buffer at the cursor.
#[inline(always)]
pub fn op_nnn(bytecode: &[u8], cursor: usize) -> u16 {
    ((bytecode[cursor] as u16 & 0b1111) << 8) | bytecode[cursor + 1] as u16
}

/// Extract operand NN from the buffer at the cursor.
#[inline(always)]
pub fn op_nn(bytecode: &[u8], cursor: usize) -> u8 {
    bytecode[cursor]
}

/// Extract operands VX and NN from the buffer at the cursor.
#[inline(always)]
pub fn op_xnn(bytecode: &[u8], cursor: usize) -> (u8, u8) {
    // Opcode is in upper nibble and needs to be masked out.
    ((bytecode[cursor] & 0b1111), bytecode[cursor + 1])
}

/// Extract operands VX and NN from the buffer at the cursor.
#[inline(always)]
pub fn op_xyn(bytecode: &[u8], cursor: usize) -> (u8, u8, u8) {
    // Opcode is in upper nibble and needs to be masked out.
    let x = bytecode[cursor] & 0b1111;
    let next = bytecode[cursor + 1];
    let y = (next & 0b1111_0000) >> 4;
    let n = next & 0b1111;

    (x, y, n)
}

/// Extract operands VX and VY from the buffer at the cursor.
#[inline(always)]
pub fn op_xy(bytecode: &[u8], cursor: usize) -> (u8, u8) {
    // Opcode is in upper nibble and needs to be masked out.
    let x = bytecode[cursor] & 0b1111;

    // Lower nibble is unused.
    let y = (bytecode[cursor + 1] & 0b1111_0000) >> 4;

    (x, y)
}

/// Extract operands VX, VY and N from the buffer at the cursor.
#[inline(always)]
pub fn op_x(bytecode: &[u8], cursor: usize) -> u8 {
    // Opcode is in upper nibble and needs to be masked out.
    bytecode[cursor] & 0b1111
}

/// Extract operand N from the buffer at the cursor.
#[inline(always)]
pub fn op_n(bytecode: &[u8], cursor: usize) -> u8 {
    let data = bytecode[cursor + 1];
    data & 0b1111
}

// Encode a a bare instruction, which has no arguments.
pub fn encode_bare(opcode: u8) -> [u8; 2] {
    trace!("encode 0x{:03X}", opcode);
    [0, opcode]
}

pub fn encode_xnn(opcode: u8, vx: u8, nn: u8) -> [u8; 2] {
    trace!("encode {:02X} {:02X}, {:02X}", opcode, vx, nn);
    [(opcode << 4) | (vx & 0xF), nn]
}

pub fn encode_xyn(opcode: u8, vx: u8, vy: u8, n: u8) -> [u8; 2] {
    trace!("encode {:02X} {:02X}, {:02X}, {:02X}", opcode, vx, vy, n);
    [(opcode << 4) | (vx & 0xF), (vy << 4) | (n & 0xF)]
}

pub fn encode_nnn(opcode: u8, nnn: u16) -> [u8; 2] {
    trace!("encode {:02X} 0x{:03X}", opcode, nnn);
    let part1 = ((nnn & 0b1111_0000_0000) >> 8) as u8;
    let part2 = (nnn & 0b1111_1111) as u8;
    [(opcode << 4) | part1, part2]
}
