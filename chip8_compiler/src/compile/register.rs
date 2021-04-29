use std::fmt;

/// Register identifying index.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Register {
    pub id: RegisterId,
    pub usage: Usage,
}

pub type RegisterId = u8;

/// Specifies the lifetime of a register's usage.
///
/// Used by the emitter to decide whether a used
/// register can be freed for future use.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Usage {
    /// Register is used for evaluating an expression.
    Temp,
    /// Register is reserved by a local variable.
    Reserved,
}

impl Register {
    #[inline]
    pub fn is_temp(&self) -> bool {
        self.usage == Usage::Temp
    }
}

impl Default for Register {
    fn default() -> Self {
        Register {
            id: 0,
            usage: Usage::Temp,
        }
    }
}

pub struct RegisterMask(u16);

/// Simple bitmask to track which operand register
/// of a CHIP-8 CPU has been used.
///
/// CHIP-8 is a register machine, so instead of a
/// stack for holding operands, we have registers.
/// Inserting and removing specific register indices
/// replaces push-pop semantics.
///
/// The code emitter will be using [`RegisterMask::find_vacant()`](struct.RegisterMask.html#method-find_vacant)
/// to choose empty registers.
impl RegisterMask {
    /// Only 15 registers are available for arithmetic.
    /// The 16th is a special register used as a flag in
    /// some instructions.
    pub const MAX_MASK: u16 = 0b0111_1111_1111_1111;

    #[inline]
    pub fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub fn contains(&self, register: u8) -> bool {
        ((1 << register) & self.0) > 0
    }

    #[inline]
    pub fn vacant(&self, register: u8) -> bool {
        !self.contains(register)
    }

    /// Assign the given register.
    ///
    /// The register will now be considered occupied
    /// and is no longer available for other instructions
    /// to use.
    #[inline]
    pub fn insert(&mut self, register: u8) {
        // TODO: bounds check
        self.0 |= 1 << register;
    }

    /// Clear the given register, freeing it up to be
    /// used by other instructions.
    #[inline]
    pub fn remove(&mut self, register: u8) {
        // TODO: bounds check
        self.0 &= !(1 << register);
    }

    /// Search the mask for a vacant register.
    ///
    /// Returns the register index on success, returns None
    /// if all registers have been assigned.
    #[inline]
    pub fn find_vacant(&mut self) -> Option<u8> {
        for i in 0..=15 {
            if self.vacant(i) {
                self.insert(i);
                return Some(i);
            }
        }
        None
    }
}

impl Default for RegisterMask {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for RegisterMask {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RegisterMask({:016b})", self.0)
    }
}
