//! Bytecode interpreter.
use std::cell::Cell;

use rand::prelude::*;

use crate::{constants::*, cpu::Chip8Cpu, vm::Interpreter};

// Bytecode interpreter.
pub struct BytecodeInterp {
    loop_counter: Cell<usize>,
}

impl BytecodeInterp {
    pub fn new() -> Self {
        Self {
            loop_counter: Cell::new(0),
        }
    }

    // FIXME: Currently we can't break out of the infinite loops that programs use.
    fn guard_infinite(&self, cpu: &mut Chip8Cpu) {
        let counter = self.loop_counter.get();
        self.loop_counter.set(counter + 1);
        if counter > 1000 {
            cpu.trap = true;
        }
    }
}

impl Interpreter for BytecodeInterp {
    fn on_load(&mut self, _cpu: &Chip8Cpu) {
        /* No op */
    }

    fn execute(&self, cpu: &mut Chip8Cpu) {
        let mut rng = thread_rng();
        self.loop_counter.set(0);

        loop {
            if cpu.trap {
                // Interrupt signal is set.
                return;
            }

            // Each instruction is two bytes, with the opcode identity in the first 4-bit nibble.
            let code = cpu.op_code();

            match code {
                // 00E0 (CLS)
                //
                // Clear display
                0x00E0 => {
                    // for px in cpu.display.iter_mut() {
                    //     *px = false;
                    // }
                    cpu.clear_display();
                    cpu.pc += 2;
                }
                // 00EE (RET)
                //
                // Return from a subroutine.
                // Set the program counter to the value at the top of the stack.
                // Subtract 1 from the stack pointer.
                0x00EE => {
                    cpu.pc = cpu.stack[cpu.sp] as usize;
                    cpu.sp -= 1;
                }
                // 1NNN (JP addr)
                //
                // Jump to address.
                0x1 => {
                    op_trace_nnn("JP", cpu);

                    let address: Address = cpu.op_nnn();
                    cpu.pc = address as usize;

                    self.guard_infinite(cpu);
                }
                // 2NNN (CALL addr)
                //
                // Call subroutine at NNN.
                0x2 => {
                    op_trace_nnn("CALL", cpu);

                    cpu.sp += 1;
                    cpu.stack[cpu.sp] = cpu.pc as u16;
                    cpu.pc = cpu.op_nnn() as usize;
                }
                // 3XNN (SE Vx, byte)
                //
                // Skip the next instruction if register VX equals value NN.
                0x3 => {
                    op_trace_xnn("SE", cpu);

                    let (vx, nn) = cpu.op_xnn();
                    if cpu.registers[vx as usize] == nn {
                        cpu.pc += 4;
                    } else {
                        cpu.pc += 2;
                    }
                }
                // 4XNN (SNE Vx, byte)
                //
                // Skip the next instruction if register VX does not equal value NN.
                0x4 => {
                    op_trace_xnn("SNE", cpu);

                    let (vx, nn) = cpu.op_xnn();
                    if cpu.registers[vx as usize] != nn {
                        cpu.pc += 4;
                    } else {
                        cpu.pc += 2;
                    }
                }
                // 5XY0 (SE Vx, Vy)
                //
                // Skip the next instruction if register VX equals value VY.
                0x5 => {
                    op_trace_xy("SE", cpu);

                    let (vx, vy) = cpu.op_xy();
                    if cpu.registers[vx as usize] == cpu.registers[vy as usize] {
                        cpu.pc += 4;
                    } else {
                        cpu.pc += 2;
                    }
                }
                // 6XNN (LD Vx, byte)
                //
                // Set register VX to value NN.
                0x6 => {
                    op_trace_xnn("LD", cpu);

                    let (vx, nn) = cpu.op_xnn();
                    cpu.registers[vx as usize] = nn;
                    cpu.pc += 2;
                }
                // 7XNN (ADD Vx, byte)
                //
                // Add value NN to register VX. Carry flag is not set.
                0x7 => {
                    op_trace_xnn("ADD", cpu);

                    let (vx, nn) = cpu.op_xnn();
                    cpu.registers[vx as usize] += nn;
                    cpu.pc += 2;
                }
                0x8 => match cpu.op_n() {
                    // 8XY0 (LD Vx, Vy)
                    //
                    // Store the value of register VY in register VX.
                    0x0 => {
                        op_trace_xy_op("LD", cpu);

                        let (vx, vy) = cpu.op_xy();
                        cpu.registers[vx as usize] = cpu.registers[vy as usize];
                        cpu.pc += 2;
                    }
                    // 8XY1 (OR Vx, Vy)
                    //
                    // Performs bitwise OR on VX and VY, and stores the result in VX.
                    0x1 => {
                        op_trace_xy_op("OR", cpu);

                        let (vx, vy) = cpu.op_xy();
                        cpu.registers[vx as usize] |= cpu.registers[vy as usize];
                        cpu.pc += 2;
                    }
                    // 8XY2 (AND Vx, Vy)
                    //
                    // Performs bitwise AND on VX and VY, and stores the result in VX.
                    0x2 => {
                        op_trace_xy_op("AND", cpu);

                        let (vx, vy) = cpu.op_xy();
                        cpu.registers[vx as usize] &= cpu.registers[vy as usize];
                        cpu.pc += 2;
                    }
                    // 8XY3 (XOR Vx, Vy)
                    //
                    // Performs bitwise XOR on VX and VY, and stores the result in VX.
                    0x3 => {
                        op_trace_xy_op("XOR", cpu);

                        let (vx, vy) = cpu.op_xy();
                        cpu.registers[vx as usize] ^= cpu.registers[vy as usize];
                        cpu.pc += 2;
                    }
                    // 8XY4 (ADD Vx, Vy)
                    //
                    // ADDs VX to VY, and stores the result in VX.
                    // Overflow is wrapped.
                    // If overflow, set VF to 1, else 0.
                    0x4 => {
                        op_trace_xy_op("ADD", cpu);

                        let (vx, vy) = cpu.op_xy();
                        let (x, y) = (cpu.registers[vx as usize], cpu.registers[vy as usize]);
                        let result = x as usize + y as usize;
                        cpu.registers[vx as usize] = (result & 0xF) as u8; // Overflow wrap
                        cpu.registers[0xF] = if result > 0x255 { 1 } else { 0 };
                        cpu.pc += 2;
                    }
                    // 8XY5 (SUB Vx, Vy)
                    //
                    // Subtracts VY from VX, and stores the result in VX.
                    // VF is set to 0 when there is a borrow, set to 1 when there isn't.
                    0x5 => {
                        op_trace_xy_op("SUB", cpu);

                        let (vx, vy) = cpu.op_xy();
                        let (x, y) = (cpu.registers[vx as usize], cpu.registers[vy as usize]);
                        let result = x as isize - y as isize;
                        cpu.registers[vx as usize] = (result & 0xF) as u8; // Overflow wrap
                        cpu.registers[0xF] = if y > x { 0 } else { 1 };
                        cpu.pc += 2;
                    }
                    // 8XY6 (SHR Vx)
                    //
                    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
                    // Shift VX right by 1.
                    // VY is unused.
                    0x6 => {
                        op_trace_xy_op("SHR", cpu);

                        let (vx, _vy) = cpu.op_xy();
                        let x = cpu.registers[vx as usize];
                        cpu.registers[0xF] = x & 1;
                        cpu.registers[vx as usize] = x >> 1;
                        cpu.pc += 2;
                    }
                    // 8XY7 (SUBN Vx, Vy)
                    //
                    // Subtracts VX from VY, and stores the result in VX.
                    // VF is set to 0 when there is a borrow, set to 1 when there isn't.
                    0x7 => {
                        op_trace_xy_op("SUBN", cpu);

                        let (vx, vy) = cpu.op_xy();
                        let (x, y) = (cpu.registers[vx as usize], cpu.registers[vy as usize]);
                        let result = y as isize - x as isize;
                        cpu.registers[vx as usize] = (result & 0xF) as u8; // Overflow wrap
                        cpu.registers[0xF] = if x > y { 0 } else { 1 };
                        cpu.pc += 2;
                    }
                    // 8XYE (SHL Vx)
                    //
                    // Shift VX left by 1.
                    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
                    // VY is unused.
                    0xE => {
                        op_trace_xy_op("SHL", cpu);

                        let (vx, _vy) = cpu.op_xy();
                        let x = cpu.registers[vx as usize];
                        cpu.registers[0xF] = x & 1;
                        cpu.registers[vx as usize] = x << 1;
                        cpu.pc += 2;
                    }
                    // Unsupported operation.
                    _ => cpu.set_error("unsupported opcode"),
                },
                // 9XY0 (SNE Vx, Vy)
                //
                // Skip next instruction if Vx != Vy.
                // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
                0x9 => todo!("SNE Vx, Vy"),
                // ANNN (LD I, addr)
                //
                // Set address register I to value NNN.
                0xA => {
                    op_trace_nnn("LDI", cpu);

                    cpu.address = cpu.op_nnn();
                    cpu.pc += 2;
                }
                0xB => todo!("JP V0, addr"),
                // CXNN (RND Vx, byte)
                //
                // Generate random number.
                // Set register VX to the result of bitwise AND between a random number and NN.
                0xC => {
                    op_trace_xnn("RND", cpu);

                    let (vx, nn) = cpu.op_xnn();
                    cpu.registers[vx as usize] = nn & rng.gen::<u8>();
                    cpu.pc += 2;
                }
                // DXYN (DRW Vx, Vy, nibble)
                //
                // Draw sprite to the display buffer, at coordinate as per registers VX and VY.
                // Sprite is encoded as 8 pixels wide, N+1 pixels high, stored in bits located in
                // memory pointed to by address register I.
                //
                // If the sprite is drawn outside of the display area, it is wrapped around to the other side.
                //
                // If the drawing operation erases existing pixels in the display buffer, register VF is set to
                // 1, and set to 0 if no display bits are unset. This is used for collision detection.
                0xD => {
                    op_trace_xyn("DRAW", cpu);

                    let (vx, vy, n) = cpu.op_xyn();
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
                            let d = ((x + c) & DISPLAY_WIDTH_MASK)
                                + ((y + r) & DISPLAY_HEIGHT_MASK) * DISPLAY_WIDTH;

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
                0xE => match cpu.op_nn() {
                    0x9E => todo!("SKP Vx"),
                    0xA1 => todo!("SKNP Vx"),
                    // Unsupported operation.
                    _ => cpu.set_error("unsupported opcode"),
                },
                0xF => match cpu.op_nn() {
                    0x07 => todo!("LD Vx, DT"),
                    0x0A => todo!("LD Vx, K"),
                    0x15 => todo!("LD DT, Vx"),
                    0x18 => todo!("LD ST, Vx"),
                    0x1E => todo!("ADD I, Vx"),
                    0x29 => todo!("LD F, Vx"),
                    0x33 => todo!("LD B, Vx"),
                    0x55 => todo!("LD [I], Vx"),
                    0x65 => todo!("LD Vx, [I]"),
                    // Unsupported operation.
                    _ => cpu.set_error("unsupported opcode"),
                },
                // Unsupported operation.
                _ => cpu.set_error("unsupported opcode"),
            }
        }
    }
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace_nnn(name: &str, cpu: &Chip8Cpu) {
    let nnn = cpu.op_nnn();
    println!("{:04X}: {:4} {:03X}", cpu.pc, name, nnn);
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace_xnn(name: &str, cpu: &Chip8Cpu) {
    let (vx, nn) = cpu.op_xnn();
    println!("{:04X}: {:4} V{:02X} {:02X}", cpu.pc, name, vx, nn);
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace_xyn(name: &str, cpu: &Chip8Cpu) {
    let (vx, vy, n) = cpu.op_xyn();
    println!(
        "{:04X}: {:4} V{:02X} V{:02X} {:01X}",
        cpu.pc, name, vx, vy, n
    );
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace_xy(name: &str, cpu: &Chip8Cpu) {
    let (vx, vy) = cpu.op_xy();
    println!("{:04X}: {:4} V{:02X} V{:02X}", cpu.pc, name, vx, vy);
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace_xy_op(name: &str, cpu: &Chip8Cpu) {
    let (vx, vy) = cpu.op_xy();
    let op2 = cpu.op_n();
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
