//! IO device interface
use crate::constants::*;

/// Hooks to provide IO devices to the virtual machine.
pub trait Devices {
    /// Wait for keyboard input.
    fn input_wait(&self) -> KeyCode;

    /// Checks immediately whether the given key is currently pressed.
    fn is_pressed(&self, key: KeyCode) -> bool;

    /// Blit the display buffer to screen output.
    fn draw(&self, display: &[bool; DISPLAY_BUFFER_SIZE]);

    /// Turn the sound buzzer on or off.
    fn buzz(&self, state: bool);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyCode {
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
    KeyF = 0xF,
}

impl KeyCode {
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Key0 => 0,
            Self::Key1 => 1,
            Self::Key2 => 2,
            Self::Key3 => 3,
            Self::Key4 => 4,
            Self::Key5 => 5,
            Self::Key6 => 6,
            Self::Key7 => 7,
            Self::Key8 => 8,
            Self::Key9 => 9,
            Self::KeyA => 10,
            Self::KeyB => 11,
            Self::KeyC => 12,
            Self::KeyD => 13,
            Self::KeyE => 14,
            Self::KeyF => 15,
        }
    }
}

impl std::fmt::Display for KeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let key_id = self.as_u8();
        write!(f, "k{key_id:x}")
    }
}

impl From<KeyCode> for u8 {
    fn from(keycode: KeyCode) -> Self {
        keycode.as_u8()
    }
}

impl TryFrom<u8> for KeyCode {
    type Error = InvalidKeyCode;

    fn try_from(key_id: u8) -> Result<Self, Self::Error> {
        match key_id {
            0 => Ok(Self::Key0),
            1 => Ok(Self::Key1),
            2 => Ok(Self::Key2),
            3 => Ok(Self::Key3),
            4 => Ok(Self::Key4),
            5 => Ok(Self::Key5),
            6 => Ok(Self::Key6),
            7 => Ok(Self::Key7),
            8 => Ok(Self::Key8),
            9 => Ok(Self::Key9),
            10 => Ok(Self::KeyA),
            11 => Ok(Self::KeyB),
            12 => Ok(Self::KeyC),
            13 => Ok(Self::KeyD),
            14 => Ok(Self::KeyE),
            15 => Ok(Self::KeyF),
            _ => Err(InvalidKeyCode),
        }
    }
}

#[derive(Debug)]
pub struct InvalidKeyCode;

impl std::error::Error for InvalidKeyCode {}

impl std::fmt::Display for InvalidKeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "keycode must be in range 0 <= keycode < 16")
    }
}

#[cfg(feature = "serde")]
mod de {
    use std::fmt::Display;

    use num_traits::AsPrimitive;
    use serde::de::{Deserialize, Error, Expected, Unexpected, Visitor};

    use super::*;

    impl Expected for InvalidKeyCode {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            <Self as Display>::fmt(self, f)
        }
    }

    impl<'de> Deserialize<'de> for KeyCode {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            // YAML integer type
            deserializer.deserialize_i64(KeyCodeVisitor)
        }
    }

    struct KeyCodeVisitor;

    impl KeyCodeVisitor {
        #[inline]
        fn check_int<N, E>(val: N) -> Result<(), E>
        where
            N: AsPrimitive<isize>,
            E: Error,
        {
            let n = val.as_();
            if n < 0 || n > u8::MAX as isize {
                Err(E::invalid_value(
                    Unexpected::Signed(n as i64),
                    &InvalidKeyCode,
                ))
            } else {
                Ok(())
            }
        }
    }

    impl<'de> Visitor<'de> for KeyCodeVisitor {
        type Value = KeyCode;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "an 8-bit integer between 0 and 16")
        }

        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Self::check_int(v)?;
            KeyCode::try_from(v as u8)
                .map_err(|err| E::invalid_value(Unexpected::Signed(v as i64), &err))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Self::check_int(v)?;
            KeyCode::try_from(v as u8)
                .map_err(|err| E::invalid_value(Unexpected::Signed(v as i64), &err))
        }
    }
}
