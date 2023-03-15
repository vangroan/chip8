//! IO device interface
use crate::constants::*;

/// Hooks to provide IO devices to the virtual machine.
pub trait Devices {
    /// Wait for keyboard input.
    fn input_wait(&self) -> Input;

    /// Checks immediately whether the given key is currently pressed.
    fn is_pressed(&self, key: Input) -> bool;

    /// Blit the display buffer to screen output.
    fn draw(&self, display: &[bool; DISPLAY_BUFFER_SIZE]);

    /// Turn the sound buzzer on or off.
    fn buzz(&self, state: bool);
}

#[derive(Debug)]
#[repr(u8)]
pub enum Input {
    Key0 = 0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
}
