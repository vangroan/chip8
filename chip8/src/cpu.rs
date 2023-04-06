//! CPU and memory state.
use crate::{bytecode::*, constants::*};

/// Core state for a chip8 interpreter.
#[allow(dead_code)]
pub struct Chip8Cpu {
    // ------------------------------------------------------------------------
    // Registers
    /// Program counter pointing to the current position in the bytecode.
    pub(crate) pc: usize,
    /// Stack pointer, indicating the top of the stack.
    pub(crate) sp: usize,
    /// General purpose registers for temporary values.
    ///
    /// Register 16 (VF) is used for either the carry flag or borrow switch depending on opcode.
    pub(crate) registers: [u8; REGISTER_COUNT],
    /// Pointer register used for temporarily storing an address. Since addresses are 12 bits, only the
    /// lowest (rightmost) bits are used.
    pub(crate) address: Address,
    /// (DT) Delay timer that counts down to 0.
    pub(crate) delay_timer: u8,
    /// (ST) Sound timer that counts down to 0. When it has a non-zero value, a beep is played.
    pub(crate) sound_timer: u8,
    /// Switch tracking whether the buzzer should be on or off.
    pub(crate) buzzer_state: bool,
    /// Indicates that the machine is waiting for a keypress.
    pub(crate) key_wait: bool,
    /// Keyboard input state. Pressed is a 1 bit, released is a 0 bit.
    pub(crate) key_state: u16,

    // ------------------------------------------------------------------------
    // Memory
    /// Main memory storage space.
    pub(crate) ram: Box<[u8; MEM_SIZE]>,
    /// Stack of return pointers used for jumping when a routine call finishes.
    pub(crate) stack: Box<[Address; STACK_SIZE]>,
    /// Screen buffer that is drawn too.
    pub(crate) display: Box<[bool; DISPLAY_BUFFER_SIZE]>,

    // ------------------------------------------------------------------------
    // Control
    /// Interrupt for VM loop.
    pub(crate) trap: bool,
    /// Error message if the VM is in an error state.
    pub(crate) error: Option<&'static str>,
}

impl Default for Chip8Cpu {
    fn default() -> Self {
        Self {
            pc: 0,
            sp: 0,
            registers: [0; 16],
            address: 0,
            delay_timer: 0,
            sound_timer: 0,
            buzzer_state: false,
            key_wait: false,
            key_state: 0,

            ram: Box::new([0; MEM_SIZE]),
            stack: Box::new([0; STACK_SIZE]),
            display: Box::new([false; DISPLAY_BUFFER_SIZE]),

            trap: false,
            error: None,
        }
    }
}

impl Chip8Cpu {
    pub fn new() -> Self {
        Default::default()
    }

    /// Erase the contents of the memory buffers `ram`, `stack` and `display`.
    pub(crate) fn clear_memory(&mut self) {
        self.ram.fill(0);
        self.stack.fill(0);
        self.display.fill(false);
    }

    pub fn interrupt(&mut self) {
        self.trap = true;
    }

    pub fn set_error(&mut self, message: &'static str) {
        self.trap = true;
        self.error = Some(message);
    }

    pub fn error(&self) -> Option<&str> {
        self.error
    }

    pub fn clear_display(&mut self) {
        self.display.fill(false);
    }

    pub fn set_key_state(&mut self, key_id: u8, state: bool) {
        if key_id <= KEY_COUNT {
            if state {
                self.key_state |= 1 << key_id;
            } else {
                self.key_state &= !(1 << key_id);
            }
        }
    }

    pub fn key_state(&self, key_id: u8) -> bool {
        if key_id <= KEY_COUNT {
            self.key_state & (1 << key_id) > 0
        } else {
            false
        }
    }

    /// Check whether any key is pressed down.
    #[inline(always)]
    pub fn any_key(&self) -> bool {
        self.key_state > 0
    }

    /// Retrieve the value of the first key that is pressed down.
    #[inline]
    pub fn first_key(&self) -> Option<u8> {
        if self.any_key() {
            for k in 0..KEY_COUNT {
                if self.key_state(k) {
                    return Some(k);
                }
            }
        }
        None
    }

    /// Clear the keyboard input state, setting all keys to up.
    #[inline(always)]
    pub fn clear_keys(&mut self) {
        self.key_state = 0;
    }

    /// Count down the delay timer.
    #[inline]
    pub fn tick_delay(&mut self) {
        // The checked_sub implementation uses `unlikely!()` which degrades performance.
        let (val, underflow) = self.delay_timer.overflowing_sub(1);
        if !underflow {
            self.delay_timer = val;
        }
    }

    #[inline]
    pub fn tick_sound(&mut self) {
        // The checked_sub implementation uses `unlikely!()` which degrades performance.
        let (val, underflow) = self.sound_timer.overflowing_sub(1);
        if !underflow {
            self.sound_timer = val;
        }
    }

    /// Extract the instruction at the current program counter.
    #[inline(always)]
    pub fn instr(&self) -> [u8; 2] {
        [self.ram[self.pc & 0xFFF], self.ram[(self.pc + 1) & 0xFFF]]
    }

    /// Extract opcode from the current program pointer.
    #[inline(always)]
    pub fn op_code(&self) -> u8 {
        op_code(&*self.ram, self.pc)
    }

    /// Extract operand NNN from the current program counter.
    #[inline(always)]
    pub fn op_nnn(&self) -> u16 {
        op_nnn(&*self.ram, self.pc)
    }

    /// Extract operand NN from the current program counter.
    #[inline(always)]
    pub fn op_nn(&self) -> u8 {
        op_nn(&*self.ram, self.pc)
    }

    /// Extract operands VX and NN from the current program counter.
    #[inline(always)]
    pub fn op_xnn(&self) -> (u8, u8) {
        op_xnn(&*self.ram, self.pc)
    }

    /// Extract operands VX, VY and N from the current program counter.
    #[inline(always)]
    pub fn op_xyn(&self) -> (u8, u8, u8) {
        op_xyn(&*self.ram, self.pc)
    }

    /// Extract operands VX, VY and N from the current program counter.
    #[inline(always)]
    pub fn op_xy(&self) -> (u8, u8) {
        op_xy(&*self.ram, self.pc)
    }

    /// Extract operand VX from the current program counter.
    #[inline(always)]
    pub fn op_x(&self) -> u8 {
        op_x(&*self.ram, self.pc)
    }

    /// Extract operand N from the current program counter.
    #[inline(always)]
    pub fn op_n(&self) -> u8 {
        op_n(&*self.ram, self.pc)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_key_state() {
        let mut cpu = Chip8Cpu::default();

        cpu.set_key_state(0, true);
        assert_eq!(cpu.key_state, 0b00000000_00000001);
        assert!(cpu.key_state(0));
        assert!(!cpu.key_state(1));
        assert!(!cpu.key_state(7));

        cpu.set_key_state(7, true);
        assert_eq!(cpu.key_state, 0b00000000_10000001);
        assert!(cpu.key_state(0));
        assert!(!cpu.key_state(1));
        assert!(cpu.key_state(7));

        cpu.set_key_state(0, false);
        assert_eq!(cpu.key_state, 0b00000000_10000000);
        assert!(!cpu.key_state(0));
        assert!(!cpu.key_state(1));
        assert!(cpu.key_state(7));

        cpu.set_key_state(15, true);
        assert_eq!(cpu.key_state, 0b10000000_10000000);
        assert!(!cpu.key_state(0));
        assert!(!cpu.key_state(1));
        assert!(cpu.key_state(7));
        assert!(cpu.key_state(15));
    }
}
