/// Helpers for extracting data from opcodes.
use crate::constants::*;

pub mod opcodes {
    /// Load (LD Vx, byte)
    pub const LD_VX_BYTE: u8 = 0x6;
    /// ANNN (LD I, addr)
    pub const LD_NNN_BYTE: u8 = 0xA;
    /// CXNN (RND Vx, byte)
    pub const RND_X_BYTE: u8 = 0xC;
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

pub fn encode_xnn(opcode: u8, vx: u8, nn: u8) -> [u8; 2] {
    println!("encode {:02X} {:02X} {:02X}", opcode, vx, nn);
    [(opcode << 4) | (vx & 0xF), nn]
}

pub fn encode_nnn(opcode: u8, nnn: u16) -> [u8; 2] {
    println!("encode {:02X} 0x{:03X}", opcode, nnn);
    let part1 = ((nnn & 0b1111_0000_0000) >> 8) as u8;
    let part2 = (nnn & 0b1111_1111) as u8;
    [(opcode << 4) | part1, part2]
}
