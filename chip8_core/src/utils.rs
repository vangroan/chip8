/// Convenience helpers for extracting data from opcodes.
use crate::Chip8Cpu;

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
    let op = bytecode[cursor] & 0b1111;
    let data = bytecode[cursor + 1];

    (op, (data & 0b1111_0000) >> 4, data & 0b1111)
}

/// Extract operands VX, VY and N from the buffer at the cursor.
#[inline(always)]
pub fn op_xy(bytecode: &[u8], cursor: usize) -> (u8, u8) {
    // Opcode is in upper nibble and needs to be masked out.
    let op = bytecode[cursor] & 0b1111;
    let data = bytecode[cursor + 1];

    // Lower nibble is unused.
    (op, (data & 0b1111_0000) >> 4)
}

/// Extract operand N from the buffer at the cursor.
#[inline(always)]
pub fn op_n(bytecode: &[u8], cursor: usize) -> u8 {
    let data = bytecode[cursor + 1];
    data & 0b1111
}
