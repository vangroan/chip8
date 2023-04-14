//! Virtual machine.
use std::{
    fmt::{self, Write},
    time::Duration,
};

use rand::prelude::*;

use crate::{
    bytecode::*,
    clock::Clock,
    constants::*,
    cpu::Chip8Cpu,
    devices::KeyCode,
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

        self.reset();

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Flow {
    Ok,
    Error,
    Interrupt,
    Draw,
    Sound,
    /// Wait for a keypress.
    ///
    /// This is triggered by the opcode `Fx0A` (`LD Vx, K`), which stops
    /// execution until a key is pressed, and loads the key value into `Vx`.
    KeyWait,
}

/// VM Configuration Parameters.
#[derive(Default, Clone)]
pub struct Chip8Conf {
    pub clock_frequency: Option<Hz>,
}

/// CPU clock frequency, in hertz (per second)
#[derive(Debug, Default, Clone, Copy)]
pub struct Hz(pub u64);

impl From<Hz> for Duration {
    fn from(freq: Hz) -> Self {
        if freq.0 == 0 {
            Duration::from_nanos(0)
        } else {
            Duration::from_nanos(NANOS_IN_SECOND / freq.0)
        }
    }
}

/// Interpreter
impl Chip8Vm {
    // FIXME: Currently we can't break out of the infinite loops that programs use.
    fn guard_infinite(&mut self) -> bool {
        self.loop_counter += 1;
        self.loop_counter > INFINITE_LOOP_LIMIT;
        false
    }

    /// Sets the keyboard key input state.
    ///
    /// If the VM is waiting for keyboard input, the `key_wait` flag will
    /// be cleared so it can be resumed.
    pub fn set_key(&mut self, key: KeyCode, pressed: bool) {
        self.cpu.set_key_state(key as u8, pressed);
        self.cpu.key_wait = false;
    }

    /// Check the clock whether the CPU should be stepped.
    pub fn clock_tick(&mut self) -> bool {
        true
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
                Flow::Error => match self.cpu.error {
                    Some(err) => return Err(Chip8Error::Runtime(err)),
                    None => return Ok(Flow::Error),
                },
                Flow::Interrupt => break,
                _ => {}
            }
        }

        Ok(Flow::Ok)
    }

    fn resume(&mut self) -> Flow {
        self.cpu.trap = false;
        self.cpu.error = None;
        self.step()
    }

    pub fn tick(&mut self) -> Result<Flow, Chip8Error> {
        match self.step() {
            Flow::Error => self
                .cpu
                .error
                .map(Chip8Error::Runtime)
                .map(Result::Err)
                .unwrap_or_else(|| Err(Chip8Error::Runtime("unspecified VM error"))),
            flow => Ok(flow),
        }
    }

    fn step(&mut self) -> Flow {
        let mut rng = thread_rng();

        let mut control_flow = Flow::Ok;

        /*loop*/
        {
            if self.cpu.trap {
                // Interrupt signal is set.
                return Flow::Interrupt;
            }

            #[cfg(feature = "throttle")]
            self.clock.wait();

            // Count down timers
            if self.timer.tick() {
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

            let [a, b] = self.cpu.instr();
            let op = a >> 4; // 0xF000
            let vx = a & 0xF; // 0x0F00
            let vy = b >> 4; // 0x00F0
            let n = b & 0xF; // 0x000F
            let nn = b; // 0x00FF
            let nnn = (((a as u16) & 0xF) << 8) | b as u16; // 0x0FFF

            self.cpu.pc += 2;

            match code {
                // Miscellaneous instructions identified by nn
                0x0 | 0xE | 0xF => control_flow = self.exec_misc(op, vx, nn),
                // 1NNN (JP addr)
                //
                // Jump to address.
                0x1 => {
                    op_trace_nnn("JP", &self.cpu);

                    self.cpu.pc = nnn as usize;

                    // TODO: Remove infinite loop guard
                    if self.guard_infinite() {
                        self.cpu.set_error("infinite loop guard");
                        control_flow = Flow::Error;
                    }
                }
                // 2NNN (CALL addr)
                //
                // Call subroutine at NNN.
                0x2 => {
                    op_trace_nnn("CALL", &self.cpu);

                    self.cpu.sp += 1;
                    self.cpu.stack[self.cpu.sp] = self.cpu.pc as u16;
                    self.cpu.pc = nnn as usize;
                }
                // 3XNN (SE Vx, byte)
                //
                // Skip the next instruction if register VX equals value NN.
                0x3 => {
                    op_trace_xnn("SE", &self.cpu);

                    if self.cpu.registers[vx as usize] == nn {
                        self.cpu.pc += 2;
                    }
                }
                // 4XNN (SNE Vx, byte)
                //
                // Skip the next instruction if register VX does not equal value NN.
                0x4 => {
                    op_trace_xnn("SNE", &self.cpu);

                    if self.cpu.registers[vx as usize] != nn {
                        self.cpu.pc += 2;
                    }
                }
                // 5XY0 (SE Vx, Vy)
                //
                // Skip the next instruction if register VX equals value VY.
                0x5 => {
                    op_trace_xy("SE", &self.cpu);

                    if self.cpu.registers[vx as usize] == self.cpu.registers[vy as usize] {
                        self.cpu.pc += 2;
                    }
                }
                // 6XNN (LD Vx, byte)
                //
                // Set register VX to value NN.
                0x6 => {
                    op_trace_xnn("LD", &self.cpu);

                    self.cpu.registers[vx as usize] = nn;
                }
                // 7XNN (ADD Vx, byte)
                //
                // Add value NN to register VX. Carry flag is not set.
                0x7 => {
                    op_trace_xnn("ADD", &self.cpu);

                    self.cpu.registers[vx as usize] += nn;
                }
                // Arithmetic instructions indentified by n
                0x8 => control_flow = self.exec_math(op, vx, vy, n),
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

                    self.cpu.address = nnn;
                }
                0xB => todo!("JP V0, addr"),
                // CXNN (RND Vx, byte)
                //
                // Generate random number.
                // Set register VX to the result of bitwise AND between a random number and NN.
                0xC => {
                    op_trace_xnn("RND", &self.cpu);

                    self.cpu.registers[vx as usize] = nn & rng.gen::<u8>();
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
                    control_flow = Flow::Draw;
                }
                // Unsupported operation.
                _ => {
                    self.cpu.set_error("unsupported opcode");
                    control_flow = Flow::Error;
                }
            }
        }

        control_flow
    }

    /// Execute an arithmetic instruction
    #[inline]
    #[must_use]
    fn exec_math(&mut self, op: u8, vx: u8, vy: u8, n: u8) -> Flow {
        debug_assert_eq!(op, 0x8);
        let mut control_flow = Flow::Ok;

        match n {
            // 8XY0 (LD Vx, Vy)
            //
            // Store the value of register VY in register VX.
            0x0 => {
                op_trace_xy_op("LD", &self.cpu);

                self.cpu.registers[vx as usize] = self.cpu.registers[vy as usize];
            }
            // 8XY1 (OR Vx, Vy)
            //
            // Performs bitwise OR on VX and VY, and stores the result in VX.
            0x1 => {
                op_trace_xy_op("OR", &self.cpu);

                self.cpu.registers[vx as usize] |= self.cpu.registers[vy as usize];
            }
            // 8XY2 (AND Vx, Vy)
            //
            // Performs bitwise AND on VX and VY, and stores the result in VX.
            0x2 => {
                op_trace_xy_op("AND", &self.cpu);

                self.cpu.registers[vx as usize] &= self.cpu.registers[vy as usize];
            }
            // 8XY3 (XOR Vx, Vy)
            //
            // Performs bitwise XOR on VX and VY, and stores the result in VX.
            0x3 => {
                op_trace_xy_op("XOR", &self.cpu);

                self.cpu.registers[vx as usize] ^= self.cpu.registers[vy as usize];
            }
            // 8XY4 (ADD Vx, Vy)
            //
            // ADDs VX to VY, and stores the result in VX.
            // Overflow is wrapped.
            // If overflow, set VF to 1, else 0.
            0x4 => {
                op_trace_xy_op("ADD", &self.cpu);

                let (x, y) = (
                    self.cpu.registers[vx as usize],
                    self.cpu.registers[vy as usize],
                );
                let result = x as usize + y as usize;
                self.cpu.registers[vx as usize] = (result & 0xF) as u8; // Overflow wrap
                self.cpu.registers[0xF] = if result > 0x255 { 1 } else { 0 };
            }
            // 8XY5 (SUB Vx, Vy)
            //
            // Subtracts VY from VX, and stores the result in VX.
            // VF is set to 0 when there is a borrow, set to 1 when there isn't.
            0x5 => {
                op_trace_xy_op("SUB", &self.cpu);

                let (x, y) = (
                    self.cpu.registers[vx as usize],
                    self.cpu.registers[vy as usize],
                );
                let result = x as isize - y as isize;
                self.cpu.registers[vx as usize] = (result & 0xF) as u8; // Overflow wrap
                self.cpu.registers[0xF] = if y > x { 0 } else { 1 };
            }
            // 8XY6 (SHR Vx)
            //
            // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
            // Shift VX right by 1.
            // VY is unused.
            0x6 => {
                op_trace_xy_op("SHR", &self.cpu);

                let x = self.cpu.registers[vx as usize];
                self.cpu.registers[0xF] = x & 1;
                self.cpu.registers[vx as usize] = x >> 1;
            }
            // 8XY7 (SUBN Vx, Vy)
            //
            // Subtracts VX from VY, and stores the result in VX.
            // VF is set to 0 when there is a borrow, set to 1 when there isn't.
            0x7 => {
                op_trace_xy_op("SUBN", &self.cpu);

                let (x, y) = (
                    self.cpu.registers[vx as usize],
                    self.cpu.registers[vy as usize],
                );
                let result = y as isize - x as isize;
                self.cpu.registers[vx as usize] = (result & 0xF) as u8; // Overflow wrap
                self.cpu.registers[0xF] = if x > y { 0 } else { 1 };
            }
            // 8XYE (SHL Vx)
            //
            // Shift VX left by 1.
            // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
            // VY is unused.
            0xE => {
                op_trace_xy_op("SHL", &self.cpu);

                let x = self.cpu.registers[vx as usize];
                self.cpu.registers[0xF] = x & 1;
                self.cpu.registers[vx as usize] = x << 1;
            }
            // ----------------------------------------------------------------
            // Unsupported operation.
            _ => {
                self.cpu.set_error("unsupported math opcode");
                control_flow = Flow::Error;
            }
        }

        control_flow
    }

    /// Execute a miscellaneous instruction
    #[inline]
    #[must_use]
    fn exec_misc(&mut self, op: u8, vx: u8, nn: u8) -> Flow {
        let mut control_flow = Flow::Ok;

        match nn {
            0x0 => { /* No Op */ }
            // ----------------------------------------------------------------
            // 00E0 (CLS)
            //
            // Clear display
            0xE0 => {
                op_trace("CLS", &self.cpu);
                debug_assert_eq!(op, 0x0);

                self.cpu.clear_display();
            }
            // 00EE (RET)
            //
            // Return from a subroutine.
            // Set the program counter to the value at the top of the stack.
            // Subtract 1 from the stack pointer.
            0xEE => {
                op_trace("RET", &self.cpu);
                debug_assert_eq!(op, 0x0);

                self.cpu.pc = self.cpu.stack[self.cpu.sp] as usize;
                self.cpu.sp -= 1;
            }
            // ----------------------------------------------------------------
            // Ex9E (SKP Vx)
            0x9E => {
                op_trace("SKP", &self.cpu);
                debug_assert_eq!(op, 0xE);

                let vx = self.cpu.op_x();
                if self.cpu.key_state(self.cpu.registers[vx as usize & 0xF]) {
                    self.cpu.pc += 2;
                }
            }
            0xA1 => todo!("SKNP Vx"),
            // ----------------------------------------------------------------
            // Fx07 (LD Vx, DT)
            //
            // Set Vx = delay timer value.
            // The value of DT is placed into Vx.
            0x07 => {
                op_trace_xk("LD", &self.cpu, "DT");
                debug_assert_eq!(op, 0xF);

                let vx = self.cpu.op_x();
                self.cpu.registers[vx as usize] = self.cpu.delay_timer;
            }
            // Fx0A (LD Vx, K)
            //
            // Wait for a key press, store the value of the key in Vx.
            // All execution stops until a key is pressed, then the value of that key is stored in Vx.
            0x0A => {
                op_trace_xk("LD", &self.cpu, "K");

                if let Some(k) = self.cpu.first_key() {
                    self.cpu.registers[vx as usize] = k;
                    self.cpu.key_wait = false;
                } else {
                    // rewind the program counter to stall the machine
                    self.cpu.pc -= 2;
                    self.cpu.key_wait = true;
                    control_flow = Flow::KeyWait;
                }
            }
            // Fx15 (LD DT, Vx)
            //
            // Set delay timer = Vx.
            // DT is set equal to the value of Vx.
            0x15 => {
                op_trace_kx("LD", &self.cpu, "DT");
                debug_assert_eq!(op, 0xF);

                let vx = self.cpu.op_x();
                self.cpu.delay_timer = self.cpu.registers[vx as usize];
            }
            // Fx18 (LD ST, Vx)
            //
            // Set sound timer = Vx.
            // ST is set equal to the value of Vx.
            0x18 => {
                op_trace_kx("LD", &self.cpu, "ST");
                debug_assert_eq!(op, 0xF);

                let vx = self.cpu.op_x();
                self.cpu.sound_timer = self.cpu.registers[vx as usize];
                self.cpu.buzzer_state = self.cpu.sound_timer > 0;
                control_flow = Flow::Sound;
            }
            0x1E => todo!("ADD I, Vx"),
            0x29 => todo!("LD F, Vx"),
            0x33 => todo!("LD B, Vx"),
            0x55 => todo!("LD [I], Vx"),
            0x65 => todo!("LD Vx, [I]"),
            // ----------------------------------------------------------------
            // Unsupported operation.
            _ => {
                self.cpu.set_error("unsupported misc opcode");
                control_flow = Flow::Error;
            }
        }

        control_flow
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

    pub fn dump_keys(&self) -> Result<String, fmt::Error> {
        let mut buf = String::new();

        if self.cpu.any_key() {
            write!(buf, "keys: ")?;
            for i in 0..KEY_COUNT {
                if self.cpu.key_state(i) {
                    write!(buf, "k{i:x}")?;
                }
            }
        }

        Ok(buf)
    }
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace(name: &str, cpu: &Chip8Cpu) {
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
    let vx = cpu.op_x();
    println!("{:04X}: {:4} V{:02X} {}", cpu.pc, name, vx, k);
}

#[cfg(feature = "op_trace")]
#[inline]
fn op_trace_kx(name: &str, cpu: &Chip8Cpu, k: &str) {
    let vx = cpu.op_x();
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
fn op_trace(_: &str, _: &Chip8Cpu) {}

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

    /// Fx0A (LD Vx, K)
    ///
    /// Wait for a keypress, then store the key value in Vx.
    /// The VM must stall while waiting, and signal the state to the outer executer.
    #[test]
    #[rustfmt::skip]
    fn test_key_wait() {
        let mut vm = Chip8Vm::new(Chip8Conf::default());
        vm.load_bytecode(&[
            0xF1, 0x0A, // LD v1, K
            0x62, 0x42  // LD v2, 0x42  ; sentinal
        ]).unwrap();

        // machine must stall
        assert_eq!(vm.cpu.pc, MEM_START);
        assert_eq!(vm.step(), Flow::KeyWait);
        assert_eq!(vm.cpu.pc, MEM_START);
        assert_eq!(vm.step(), Flow::KeyWait);
        assert_eq!(vm.cpu.pc, MEM_START);
        assert_eq!(vm.step(), Flow::KeyWait);
        assert_eq!(vm.cpu.pc, MEM_START);
        assert_eq!(vm.step(), Flow::KeyWait);
        assert_eq!(vm.cpu.pc, MEM_START);
        assert_eq!(vm.step(), Flow::KeyWait);
        assert_eq!(vm.cpu.pc, MEM_START);
        assert_eq!(vm.step(), Flow::KeyWait);

        // machine has yielded, waiting for any key to be pressed.
        vm.set_key(KeyCode::Key5, true);

        // machine will now advance
        vm.step();
        assert_eq!(vm.cpu.pc, MEM_START + 2);
        assert!(vm.cpu.key_state(0x05));
        assert_eq!(vm.cpu.registers[1], 0x05);

        // Ensure the machine is continuing
        vm.step();
        assert_eq!(vm.cpu.pc, MEM_START + 4);
        // assert!(!vm.cpu.key_state(0x05), "keyboard state was not cleared");
        assert_eq!(vm.cpu.registers[2], 0x42); // sentinal
    }
}
