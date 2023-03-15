/// Helpers for extracting data from opcodes.

/// Extract opcode from the current program pointer.
// #[inline(always)]
// pub fn op_code(cpu: &Chip8Cpu) -> u8 {
//     (cpu.ram[cpu.pc] & 0b1111_0000) >> 4
// }

// /// Extract NNN data from the current program counter.
// #[inline(always)]
// pub fn op_nnn(cpu: &Chip8Cpu) -> u16 {
//     ((cpu.ram[cpu.pc] as u16 & 0b1111) << 8) | cpu.ram[cpu.pc + 1] as u16
// }

// /// Extract X and NN data from the current program counter.
// #[inline(always)]
// pub fn op_xnn(cpu: &Chip8Cpu) -> (u8, u8) {
//     // Opcode is in upper nibble and needs to be masked out.
//     ((cpu.ram[cpu.pc] & 0b1111), cpu.ram[cpu.pc + 1])
// }

// /// Extract X and NN data from the current program counter.
// #[inline(always)]
// pub fn op_xyn(cpu: &Chip8Cpu) -> (u8, u8, u8) {
//     // Opcode is in upper nibble and needs to be masked out.
//     let op = cpu.ram[cpu.pc] & 0b1111;
//     let data = cpu.ram[cpu.pc + 1];

//     (op, (data & 0b1111_0000) >> 4, data & 0b1111)
// }

// /// Extract VX and VY from the RAM at the current program counter.
// #[inline(always)]
// pub fn op_xy(cpu: &Chip8Cpu) -> (u8, u8) {
//     // Opcode is in upper nibble and needs to be masked out.
//     let op = cpu.ram[cpu.pc] & 0b1111;
//     let data = cpu.ram[cpu.pc + 1];

//     // Lower nibble is unused.
//     (op, (data & 0b1111_0000) >> 4)
// }

// /// Extract the last nibble from RAM at the position just after the program counter.
// #[inline(always)]
// pub fn op_n(cpu: &Chip8Cpu) -> u8 {
//     let data = cpu.ram[cpu.pc + 1];
//     data & 0b1111
// }

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
    bytecode[cursor] & 0b1111_1111
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
