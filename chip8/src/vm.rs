//! Virtual machine.
use std::{fmt::Write, time::Duration};

use rand::prelude::*;

use crate::{
    bytecode::*,
    clock::Clock,
    constants::*,
    cpu::Chip8Cpu,
    error::{Chip8Error, Chip8Result},
};

const INFINITE_LOOP_LIMIT: usize = 1000;

pub struct Chip8Vm {
    cpu: Chip8Cpu,
    clock: Clock,
    timer: Clock,
    loop_counter: usize,
    conf: Chip8Conf,
}

impl Chip8Vm {
    pub fn new(conf: Chip8Conf) -> Self {
        Chip8Vm {
            cpu: Chip8Cpu::new(),
            clock: Clock::new(conf.clock_frequency.unwrap_or_default().into()),
            timer: Clock::from_nanos(DELAY_FREQUENCY),
            loop_counter: 0,
            conf,
        }
    }

    /// Configuration that was used to instantiate the VM.
    pub fn config(&self) -> &Chip8Conf {
        &self.conf
    }

    pub fn load_bytecode(&mut self, bytecode: &[u8]) -> Chip8Result<()> {
        if !check_program_size(bytecode) {
            return Err(Chip8Error::LargeProgram);
        }

        // Start with clean memory to avoid leaking previous program.
        self.cpu.clear_memory();

        // Load program into virtual RAM
        self.cpu.ram[MEM_START..MEM_START + bytecode.len()].copy_from_slice(bytecode);

        // Reset the program counter to prepare for execution.
        self.cpu.pc = MEM_START;

        Ok(())
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum Flow {
    Error,
    Interrupt,
    Draw,
    Delay,
    Jump,
}

/// VM Configuration Parameters.
#[derive(Default)]
pub struct Chip8Conf {
    pub clock_frequency: Option<Hz>,
}

/// CPU clock frequency, in hertz (per second)
#[derive(Debug, Default, Clone, Copy)]
pub struct Hz(pub u64);

impl Into<Duration> for Hz {
    fn into(self) -> Duration {
        if self.0 == 0 {
            Duration::from_nanos(0)
        } else {
            Duration::from_nanos(NANOS_IN_SECOND / self.0)
        }
    }
}

/// Interpreter
impl Chip8Vm {
    // FIXME: Currently we can't break out of the infinite loops that programs use.
    fn guard_infinite(&mut self) -> bool {
        self.loop_counter += 1;
        if self.loop_counter > INFINITE_LOOP_LIMIT {
            self.cpu.trap = true;
            return true;
        }
        return false;
    }

    /// Clear internal state in preparation for a fresh startup.
    fn reset(&mut self) {
        self.loop_counter = 0;
        self.clock.reset();
        self.timer.reset();
    }

    pub fn execute(&mut self) -> Chip8Result<Flow> {
        self.reset();

        loop {
            match self.resume() {
                Flow::Error => {
                    return match self.cpu.error {
                        Some(err) => Err(Chip8Error::Runtime(err)),
                        None => Ok(Flow::Error),
                    }
                }
                Flow::Interrupt => return Ok(Flow::Interrupt),
                Flow::Draw | Flow::Delay | Flow::Jump => continue,
            }
        }
    }

    fn resume(&mut self) -> Flow {
        self.cpu.trap = false;
        self.cpu.error = None;
        self.step()
    }

    fn step(&mut self) -> Flow {
        let mut rng = thread_rng();

        loop {
            if self.cpu.trap {
                // Interrupt signal is set.
                return Flow::Interrupt;
            }

            #[cfg(feature = "throttle")]
            self.clock.wait();

            // Count down timers
            if self.timer.tick() {
                // self.cpu.delay = self.cpu.delay.checked_sub(1).unwrap_or_default();
                // self.cpu.sound = self.cpu.sound.checked_sub(1).unwrap_or_default();
                self.cpu.tick_sound();
                self.cpu.tick_delay();

                // Buzzer should be on while sound timer counts down,
                // then turned off when the timer reaches zero.
                if self.cpu.sound_timer > 0 && !self.cpu.buzzer_state {
                    self.cpu.buzzer_state = true;
                    // self.devices.buzz(true);
                } else if self.cpu.sound_timer == 0 && self.cpu.buzzer_state {
                    self.cpu.buzzer_state = false;
                    // self.deviecs.buzz(false);
                }
            }

            // Each instruction is two bytes, with the opcode identity in the first 4-bit nibble.
            let code = self.cpu.op_code();

            match code {
                // 00E0 (CLS)
                //
                // Clear display
                0x00E0 => {
                    op_trace("CLS");

                    self.cpu.clear_display();
                    self.cpu.pc += 2;
                }
                // 00EE (RET)
                //
                // Return from a subroutine.
                // Set the program counter to the value at the top of the stack.
                // Subtract 1 from the stack pointer.
                0x00EE => {
                    op_trace("RET");

                    self.cpu.pc = self.cpu.stack[self.cpu.sp] as usize;
                    self.cpu.sp -= 1;
                }
                // 1NNN (JP addr)
                //
                // Jump to address.
                0x1 => {
                    op_trace_nnn("JP", &self.cpu);

                    let address: Address = self.cpu.op_nnn();
                    self.cpu.pc = address as usize;

                    if self.guard_infinite() {
                        continue;
                    } else {
                        self.cpu.trap = true;
                        return Flow::Jump;
                    }
                }
                // 2NNN (CALL addr)
                //
                // Call subroutine at NNN.
                0x2 => {
                    op_trace_nnn("CALL", &self.cpu);

                    self.cpu.sp += 1;
                    self.cpu.stack[self.cpu.sp] = self.cpu.pc as u16;
                    self.cpu.pc = self.cpu.op_nnn() as usize;
                }
                // 3XNN (SE Vx, byte)
                //
                // Skip the next instruction if register VX equals value NN.
                0x3 => {
                    op_trace_xnn("SE", &self.cpu);

                    let (vx, nn) = self.cpu.op_xnn();
                    if self.cpu.registers[vx as usize] == nn {
                        self.cpu.pc += 4;
                    } else {
                        self.cpu.pc += 2;
                    }
                }
                // 4XNN (SNE Vx, byte)
                //
                // Skip the next instruction if register VX does not equal value NN.
                0x4 => {
                    op_trace_xnn("SNE", &self.cpu);

                    let (vx, nn) = self.cpu.op_xnn();
                    if self.cpu.registers[vx as usize] != nn {
                        self.cpu.pc += 4;
                    } else {
                        self.cpu.pc += 2;
                    }
                }
                // 5XY0 (SE Vx, Vy)
                //
                // Skip the next instruction if register VX equals value VY.
                0x5 => {
                    op_trace_xy("SE", &self.cpu);

                    let (vx, vy) = self.cpu.op_xy();
                    if self.cpu.registers[vx as usize] == self.cpu.registers[vy as usize] {
                        self.cpu.pc += 4;
                    } else {
                        self.cpu.pc += 2;
                    }
                }
                // 6XNN (LD Vx, byte)
                //
                // Set register VX to value NN.
                0x6 => {
                    op_trace_xnn("LD", &self.cpu);

                    let (vx, nn) = self.cpu.op_xnn();
                    self.cpu.registers[vx as usize] = nn;
                    self.cpu.pc += 2;
                }
                // 7XNN (ADD Vx, byte)
                //
                // Add value NN to register VX. Carry flag is not set.
                0x7 => {
                    op_trace_xnn("ADD", &self.cpu);

                    let (vx, nn) = self.cpu.op_xnn();
                    self.cpu.registers[vx as usize] += nn;
                    self.cpu.pc += 2;
                }
                0x8 => match self.cpu.op_n() {
                    // 8XY0 (LD Vx, Vy)
                    //
                    // Store the value of register VY in register VX.
                    0x0 => {
                        op_trace_xy_op("LD", &self.cpu);

                        let (vx, vy) = self.cpu.op_xy();
                        self.cpu.registers[vx as usize] = self.cpu.registers[vy as usize];
                        self.cpu.pc += 2;
                    }
                    // 8XY1 (OR Vx, Vy)
                    //
                    // Performs bitwise OR on VX and VY, and stores the result in VX.
                    0x1 => {
                        op_trace_xy_op("OR", &self.cpu);

                        let (vx, vy) = self.cpu.op_xy();
                        self.cpu.registers[vx as usize] |= self.cpu.registers[vy as usize];
                        self.cpu.pc += 2;
                    }
                    // 8XY2 (AND Vx, Vy)
                    //
                    // Performs bitwise AND on VX and VY, and stores the result in VX.
                    0x2 => {
                        op_trace_xy_op("AND", &self.cpu);

                        let (vx, vy) = self.cpu.op_xy();
                        self.cpu.registers[vx as usize] &= self.cpu.registers[vy as usize];
                        self.cpu.pc += 2;
                    }
                    // 8XY3 (XOR Vx, Vy)
                    //
                    // Performs bitwise XOR on VX and VY, and stores the result in VX.
                    0x3 => {
                        op_trace_xy_op("XOR", &self.cpu);

                        let (vx, vy) = self.cpu.op_xy();
                        self.cpu.registers[vx as usize] ^= self.cpu.registers[vy as usize];
                        self.cpu.pc += 2;
                    }
                    // 8XY4 (ADD Vx, Vy)
                    //
                    // ADDs VX to VY, and stores the result in VX.
                    // Overflow is wrapped.
                    // If overflow, set VF to 1, else 0.
                    0x4 => {
                        op_trace_xy_op("ADD", &self.cpu);

                        let (vx, vy) = self.cpu.op_xy();
                        let (x, y) = (
                            self.cpu.registers[vx as usize],
                            self.cpu.registers[vy as usize],
                        );
                        let result = x as usize + y as usize;
                        self.cpu.registers[vx as usize] = (result & 0xF) as u8; // Overflow wrap
                        self.cpu.registers[0xF] = if result > 0x255 { 1 } else { 0 };
                        self.cpu.pc += 2;
                    }
                    // 8XY5 (SUB Vx, Vy)
                    //
                    // Subtracts VY from VX, and stores the result in VX.
                    // VF is set to 0 when there is a borrow, set to 1 when there isn't.
                    0x5 => {
                        op_trace_xy_op("SUB", &self.cpu);

                        let (vx, vy) = self.cpu.op_xy();
                        let (x, y) = (
                            self.cpu.registers[vx as usize],
                            self.cpu.registers[vy as usize],
                        );
                        let result = x as isize - y as isize;
                        self.cpu.registers[vx as usize] = (result & 0xF) as u8; // Overflow wrap
                        self.cpu.registers[0xF] = if y > x { 0 } else { 1 };
                        self.cpu.pc += 2;
                    }
                    // 8XY6 (SHR Vx)
                    //
                    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
                    // Shift VX right by 1.
                    // VY is unused.
                    0x6 => {
                        op_trace_xy_op("SHR", &self.cpu);

                        let (vx, _vy) = self.cpu.op_xy();
                        let x = self.cpu.registers[vx as usize];
                        self.cpu.registers[0xF] = x & 1;
                        self.cpu.registers[vx as usize] = x >> 1;
                        self.cpu.pc += 2;
                    }
                    // 8XY7 (SUBN Vx, Vy)
                    //
                    // Subtracts VX from VY, and stores the result in VX.
                    // VF is set to 0 when there is a borrow, set to 1 when there isn't.
                    0x7 => {
                        op_trace_xy_op("SUBN", &self.cpu);

                        let (vx, vy) = self.cpu.op_xy();
                        let (x, y) = (
                            self.cpu.registers[vx as usize],
                            self.cpu.registers[vy as usize],
                        );
                        let result = y as isize - x as isize;
                        self.cpu.registers[vx as usize] = (result & 0xF) as u8; // Overflow wrap
                        self.cpu.registers[0xF] = if x > y { 0 } else { 1 };
                        self.cpu.pc += 2;
                    }
                    // 8XYE (SHL Vx)
                    //
                    // Shift VX left by 1.
                    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
                    // VY is unused.
                    0xE => {
                        op_trace_xy_op("SHL", &self.cpu);

                        let (vx, _vy) = self.cpu.op_xy();
                        let x = self.cpu.registers[vx as usize];
                        self.cpu.registers[0xF] = x & 1;
                        self.cpu.registers[vx as usize] = x << 1;
                        self.cpu.pc += 2;
                    }
                    // Unsupported operation.
                    _ => {
                        self.cpu.set_error("unsupported opcode");
                        return Flow::Error;
                    }
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
                    op_trace_nnn("LDI", &self.cpu);

                    self.cpu.address = self.cpu.op_nnn();
                    self.cpu.pc += 2;
                }
                0xB => todo!("JP V0, addr"),
                // CXNN (RND Vx, byte)
                //
                // Generate random number.
                // Set register VX to the result of bitwise AND between a random number and NN.
                0xC => {
                    op_trace_xnn("RND", &self.cpu);

                    let (vx, nn) = self.cpu.op_xnn();
                    self.cpu.registers[vx as usize] = nn & rng.gen::<u8>();
                    self.cpu.pc += 2;
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
                    op_trace_xyn("DRAW", &self.cpu);

                    let (vx, vy, n) = self.cpu.op_xyn();
                    let (x, y) = (
                        self.cpu.registers[vx as usize] as usize,
                        self.cpu.registers[vy as usize] as usize,
                    );
                    let mut is_erased = false;

                    // Iteration from pointer in address register I to number of rows specified by opcode value N.
                    for (r, row) in self
                        .cpu
                        .ram
                        .iter()
                        .skip(self.cpu.address as usize)
                        .take(n as usize)
                        .enumerate()
                    {
                        // Each row is 8 bits representing the 8 pixels of the sprite.
                        for c in 0..8 {
                            let d = ((x + c) & DISPLAY_WIDTH_MASK)
                                + ((y + r) & DISPLAY_HEIGHT_MASK) * DISPLAY_WIDTH;

                            let old_px = self.cpu.display[d];
                            let new_px = old_px ^ ((row >> (7 - c) & 0x1) == 1);

                            // XOR erases a pixel when both the old and new values are both 1.
                            is_erased |= old_px && new_px;

                            // Write to display buffer
                            self.cpu.display[d] = new_px;
                        }
                    }

                    // If a pixel was erased, then a collision occurred.
                    self.cpu.registers[0xF] = is_erased as u8;
                    self.cpu.pc += 2;

                    self.cpu.trap = true;
                    return Flow::Draw;
                }
                0xE => match self.cpu.op_nn() {
                    0x9E => todo!("SKP Vx"),
                    0xA1 => todo!("SKNP Vx"),
                    // Unsupported operation.
                    _ => {
                        self.cpu.set_error("unsupported opcode");
                        return Flow::Error;
                    }
                },
                0xF => match self.cpu.op_nn() {
                    // Fx07 (LD Vx, DT)
                    //
                    // Set Vx = delay timer value.
                    // The value of DT is placed into Vx.
                    0x07 => {
                        op_trace_xk("LD", &self.cpu, "DT");

                        let vx = self.cpu.op_x();
                        self.cpu.registers[vx as usize] = self.cpu.delay_timer;
                    }
                    0x0A => todo!("LD Vx, K"),
                    // Fx15 (LD DT, Vx)
                    //
                    // Set delay timer = Vx.
                    // DT is set equal to the value of Vx.
                    0x15 => {
                        op_trace_kx("LD", &self.cpu, "DT");

                        let vx = self.cpu.op_x();
                        self.cpu.delay_timer = self.cpu.registers[vx as usize];
                    }
                    // Fx18 (LD ST, Vx)
                    //
                    // Set sound timer = Vx.
                    // ST is set equal to the value of Vx.
                    0x18 => {
                        op_trace_kx("LD", &self.cpu, "ST");

                        let vx = self.cpu.op_x();
                        self.cpu.sound_timer = self.cpu.registers[vx as usize];
                        self.cpu.buzzer_state = self.cpu.sound_timer > 0;

                        // self.devices.buzz(self.cpu.buzzer_state);
                    }
                    0x1E => todo!("ADD I, Vx"),
                    0x29 => todo!("LD F, Vx"),
                    0x33 => todo!("LD B, Vx"),
                    0x55 => todo!("LD [I], Vx"),
                    0x65 => todo!("LD Vx, [I]"),
                    // Unsupported operation.
                    _ => {
                        self.cpu.set_error("unsupported opcode");
                        return Flow::Error;
                    }
                },
                // Unsupported operation.
                _ => {
                    self.cpu.set_error("unsupported opcode");
                    return Flow::Error;
                }
            }
        }
    }
}

/// Troubleshooting
#[allow(dead_code)]
#[doc(hidden)]
impl Chip8Vm {
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

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                if self.cpu.display[x + y * DISPLAY_WIDTH] {
                    write!(buf, "#")?;
                } else {
                    write!(buf, ".")?;
                }
            }
            writeln!(buf)?;
        }

        Ok(buf)
    }
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace(name: &str) {
    println!("{:04X}: {:4}", cpu.pc, name);
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
fn op_trace_xk(name: &str, cpu: &Chip8Cpu, k: &str) {
    let vx = cpu.op_xk();
    println!("{:04X}: {:4} V{:02X} {}", cpu.pc, name, vx, k);
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace_kx(name: &str, cpu: &Chip8Cpu, k: &str) {
    let vx = cpu.op_xk();
    println!("{:04X}: {:4} {} V{:02X}", cpu.pc, name, k, vx);
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
fn op_trace(_: &str) {}

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
fn op_trace_xk(_: &str, _: &Chip8Cpu, _: &str) {}

#[cfg(not(feature = "op_trace"))]
#[inline]
fn op_trace_kx(_: &str, _: &Chip8Cpu, _: &str) {}

#[cfg(not(feature = "op_trace"))]
#[inline]
fn op_trace_xy_op(_: &str, _: &Chip8Cpu) {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_clock_hz() {
        let interval: Duration = Hz(60).into();
        assert_eq!(interval.as_millis(), 16);
    }
}
