/// Convenience helpers for extracting data from opcodes.
use crate::Chip8Cpu;

/// Extract opcode from the current program pointer.
#[inline(always)]
pub fn op_code(cpu: &Chip8Cpu) -> u8 {
    (cpu.ram[cpu.pc] & 0b1111_0000) >> 4
}

/// Extract NNN data from the current program counter.
#[inline(always)]
pub fn op_nnn(cpu: &Chip8Cpu) -> u16 {
    ((cpu.ram[cpu.pc] as u16 & 0b1111) << 8) | cpu.ram[cpu.pc + 1] as u16
}

/// Extract X and NN data from the current program counter.
#[inline(always)]
pub fn op_xnn(cpu: &Chip8Cpu) -> (u8, u8) {
    // Opcode is in upper nibble and needs to be masked out.
    ((cpu.ram[cpu.pc] & 0b1111), cpu.ram[cpu.pc + 1])
}

/// Extract X and NN data from the current program counter.
#[inline(always)]
pub fn op_xyn(cpu: &Chip8Cpu) -> (u8, u8, u8) {
    // Opcode is in upper nibble and needs to be masked out.
    let op = cpu.ram[cpu.pc] & 0b1111;
    let data = cpu.ram[cpu.pc + 1];

    (op, (data & 0b1111_0000) >> 4, data & 0b1111)
}

/// Extract VX and VY from the RAM at the current program counter.
#[inline(always)]
pub fn op_xy(cpu: &Chip8Cpu) -> (u8, u8) {
    // Opcode is in upper nibble and needs to be masked out.
    let op = cpu.ram[cpu.pc] & 0b1111;
    let data = cpu.ram[cpu.pc + 1];

    // Lower nibble is unused.
    (op, (data & 0b1111_0000) >> 4)
}

/// Extract the last nibble from RAM at the position just after the program counter.
#[inline(always)]
pub fn op_n(cpu: &Chip8Cpu) -> u8 {
    let data = cpu.ram[cpu.pc + 1];
    data & 0b1111
}
