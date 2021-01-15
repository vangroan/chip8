//! Bytecode interpreter.
use crate::utils::*;
use chip8_core::{prelude::*, Address, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use rand::prelude::*;

mod dis;
pub mod utils;

pub use dis::Disassembler;

pub struct BytecodeInterpreter;

impl Interpreter for BytecodeInterpreter {
    fn on_load(&mut self, _cpu: &Chip8Cpu) {
        /* No op */
    }

    fn execute(&self, cpu: &mut Chip8Cpu) {
        let mut rng = thread_rng();
        let mut loop_count = 0;

        loop {
            // Currently we can't break out of the infinite loops that programs use.
            if loop_count > 1000 {
                return;
            }
            loop_count += 1;

            // Each instruction is two bytes, with the opcode identity in the first 4-bit nibble.
            let code = op_code(cpu);

            match code {
                0x00E0 => {
                    // 00E0
                    // Clear display
                    for px in &mut cpu.display {
                        *px = false;
                    }
                    cpu.pc += 2;
                }
                0x00EE => {
                    // 00EE
                    // Return from a subroutine.
                    // Set the program counter to the value at the top of the stack.
                    // Subtract 1 from the stack pointer.
                    cpu.pc = cpu.stack[cpu.sp] as usize;
                    cpu.sp -= 1;
                }
                0x1 => {
                    // 1NNN
                    // Jump to address.
                    op_trace_nnn("JP", cpu);

                    let address: Address = op_nnn(cpu);
                    cpu.pc = address as usize;
                }
                0x2 => {
                    // 2NNN
                    // Call subroutine at NNN.
                    op_trace_nnn("CALL", cpu);

                    cpu.sp += 1;
                    cpu.stack[cpu.sp] = cpu.pc as u16;
                    cpu.pc = op_nnn(cpu) as usize;
                }
                0x3 => {
                    // 3XNN
                    // Skip the next instruction if register VX equals value NN.
                    op_trace_xnn("SE", cpu);

                    let (vx, nn) = op_xnn(cpu);
                    if cpu.registers[vx as usize] == nn {
                        cpu.pc += 4;
                    } else {
                        cpu.pc += 2;
                    }
                }
                0x4 => {
                    // 4XNN
                    // Skip the next instruction if register VX does not equal value NN.
                    op_trace_xnn("SNE", cpu);

                    let (vx, nn) = op_xnn(cpu);
                    if cpu.registers[vx as usize] != nn {
                        cpu.pc += 4;
                    } else {
                        cpu.pc += 2;
                    }
                }
                0x5 => {
                    // 5XY0
                    // Skip the next instruction if register VX equals value VY.
                    op_trace_xy("SE", cpu);

                    let (vx, vy) = op_xy(cpu);
                    if cpu.registers[vx as usize] == cpu.registers[vy as usize] {
                        cpu.pc += 4;
                    } else {
                        cpu.pc += 2;
                    }
                }
                0x6 => {
                    // 6XNN
                    // Set register VX to value NN.
                    op_trace_xnn("LD", cpu);

                    let (vx, nn) = op_xnn(cpu);
                    cpu.registers[vx as usize] = nn;
                    cpu.pc += 2;
                }
                0x7 => {
                    // 7XNN
                    // Add value NN to register VX. Carry flag is not set.
                    op_trace_xnn("ADD", cpu);

                    let (vx, nn) = op_xnn(cpu);
                    cpu.registers[vx as usize] += nn;
                    cpu.pc += 2;
                }
                0x8 => match op_n(cpu) {
                    0x0 => {
                        // 8XY0
                        // Store the value of register VY in register VX.
                        op_trace_xy_op("LD", cpu);

                        let (vx, vy) = op_xy(cpu);
                        cpu.registers[vx as usize] = cpu.registers[vy as usize];
                        cpu.pc += 2;
                    }
                    0x1 => {
                        // 8XY1
                        // Performs bitwise OR on VX and VY, and stores the result in VX.
                        op_trace_xy_op("OR", cpu);

                        let (vx, vy) = op_xy(cpu);
                        cpu.registers[vx as usize] =
                            cpu.registers[vx as usize] | cpu.registers[vy as usize];
                        cpu.pc += 2;
                    }
                    0x2 => {
                        // 8XY2
                        // Performs bitwise AND on VX and VY, and stores the result in VX.
                        op_trace_xy_op("AND", cpu);

                        let (vx, vy) = op_xy(cpu);
                        cpu.registers[vx as usize] =
                            cpu.registers[vx as usize] & cpu.registers[vy as usize];
                        cpu.pc += 2;
                    }
                    0x3 => {
                        // 8XY3
                        // Performs bitwise XOR on VX and VY, and stores the result in VX.
                        op_trace_xy_op("XOR", cpu);

                        let (vx, vy) = op_xy(cpu);
                        cpu.registers[vx as usize] =
                            cpu.registers[vx as usize] ^ cpu.registers[vy as usize];
                        cpu.pc += 2;
                    }
                    0x4 => {
                        // 8XY4
                        // ADDs VX to VY, and stores the result in VX.
                        // Overflow is wrapped.
                        // If overflow, set VF to 1, else 0.
                        op_trace_xy_op("ADD", cpu);

                        let (vx, vy) = op_xy(cpu);
                        let (x, y) = (cpu.registers[vx as usize], cpu.registers[vy as usize]);
                        let result = x as usize + y as usize;
                        cpu.registers[vx as usize] = (result & 0xF) as u8; // Overflow wrap
                        cpu.registers[0xF] = if result > 0x255 { 1 } else { 0 };
                        cpu.pc += 2;
                    }
                    0x5 => {
                        // 8XY5
                        // Subtracts VY from VX, and stores the result in VX.
                        // VF is set to 0 when there is a borrow, set to 1 when there isn't.
                        op_trace_xy_op("SUB", cpu);

                        let (vx, vy) = op_xy(cpu);
                        let (x, y) = (cpu.registers[vx as usize], cpu.registers[vy as usize]);
                        let result = x as isize - y as isize;
                        cpu.registers[vx as usize] = (result & 0xF) as u8; // Overflow wrap
                        cpu.registers[0xF] = if y > x { 0 } else { 1 };
                        cpu.pc += 2;
                    }
                    _ => {
                        panic!(
                            "Unsupported opcode {:02X}__{:02} at address {:04X}",
                            code,
                            op_n(cpu),
                            cpu.pc
                        );
                    }
                },
                0xA => {
                    // ANNN
                    // Set address register I to value NNN.
                    op_trace_nnn("LDI", cpu);

                    cpu.address = op_nnn(cpu);
                    cpu.pc += 2;
                }
                0xC => {
                    // CXNN
                    // Set register VX to the result of bitwise AND between a random number and NN.
                    op_trace_xnn("RND", cpu);

                    let (vx, nn) = op_xnn(cpu);
                    cpu.registers[vx as usize] = nn & rng.gen::<u8>();
                    cpu.pc += 2;
                }
                0xD => {
                    // DXYN
                    //
                    // Draw sprite in the display buffer, at coordinate as per registers VX and VY.
                    // Sprite is encoded as 8 pixels wide, N+1 pixels high, stored in bits located in
                    // memory pointed to by address register I.
                    //
                    // If the sprite is drawn outside of the display area, it is wrapped around to the other side.
                    //
                    // If the drawing operation erases existing pixels in the display buffer, register VF is set to
                    // 1, and set to 0 if no display bits are unset. This is used for collision detection.
                    op_trace_xyn("DRAW", cpu);

                    let (vx, vy, n) = op_xyn(cpu);
                    let (x, y) = (
                        cpu.registers[vx as usize] as usize,
                        cpu.registers[vy as usize] as usize,
                    );
                    let mut is_erased = false;

                    // Iteration from pointer in address register I to number of rows specified by opcode value N.
                    for (r, row) in cpu
                        .ram
                        .iter()
                        .skip(cpu.address as usize)
                        .take(n as usize)
                        .enumerate()
                    {
                        // Each row is 8 bits representing the 8 pixels of the sprite.
                        for c in 0..8 {
                            let d = ((x + c) % DISPLAY_WIDTH)
                                + ((y + r) % DISPLAY_HEIGHT) * DISPLAY_WIDTH;

                            let old_px = cpu.display[d];
                            let new_px = old_px ^ ((row >> (7 - c) & 0x1) == 1);

                            // XOR erases a pixel when both the old and new values are both 1.
                            is_erased |= old_px && new_px;

                            // Write to display buffer
                            cpu.display[d] = new_px;
                        }
                    }

                    // If a pixel was erased, then a collision occurred.
                    cpu.registers[0xF] = is_erased as u8;
                    cpu.pc += 2;
                }

                _ => {
                    panic!("Unsupported opcode {:02X} at address {:04X}", code, cpu.pc)
                    // return;
                }
            }
        }
    }
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace_nnn(name: &str, cpu: &Chip8Cpu) {
    let nnn = op_nnn(cpu);
    println!("{:04X}: {:4} {:03X}", cpu.pc, name, nnn);
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace_xnn(name: &str, cpu: &Chip8Cpu) {
    let (vx, nn) = op_xnn(cpu);
    println!("{:04X}: {:4} V{:02X} {:02X}", cpu.pc, name, vx, nn);
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace_xyn(name: &str, cpu: &Chip8Cpu) {
    let (vx, vy, n) = op_xyn(cpu);
    println!(
        "{:04X}: {:4} V{:02X} V{:02X} {:01X}",
        cpu.pc, name, vx, vy, n
    );
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace_xy(name: &str, cpu: &Chip8Cpu) {
    let (vx, vy) = op_xy(cpu);
    println!("{:04X}: {:4} V{:02X} V{:02X}", cpu.pc, name, vx, vy);
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace_xy_op(name: &str, cpu: &Chip8Cpu) {
    let (vx, vy) = op_xy(cpu);
    let op2 = op_n(cpu);
    println!(
        "{:04X}: {:4} V{:02X} V{:02X} {:02X}",
        cpu.pc, name, vx, vy, op2
    );
}

#[cfg(not(feature = "op_trace"))]
#[inline]
fn op_trace_nnn(_: &str, _: &Chip8Cpu) {}

#[cfg(not(feature = "op_trace"))]
#[inline]
fn op_trace_xnn(_: &str, _: &Chip8Cpu) {}

#[cfg(not(feature = "op_trace"))]
#[inline]
fn op_trace_xyn(_: &str, _: &Chip8Cpu) {}

#[cfg(not(feature = "op_trace"))]
#[inline]
fn op_trace_xy(_: &str, _: &Chip8Cpu) {}

#[cfg(not(feature = "op_trace"))]
#[inline]
fn op_trace_xy_op(_: &str, _: &Chip8Cpu) {}
