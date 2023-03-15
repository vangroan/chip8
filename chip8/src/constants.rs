//! Constant values of the Chip-8 architecture.

/// Number of general purpose registers.
pub const REGISTER_COUNT: usize = 0x10; // 16

/// The lower memory space was historically used for the interpreter itself,
/// but is now used for fonts.
pub const MEM_START: usize = 0x200; // 512
pub const MEM_SIZE: usize = 0x1000; // 4096

/// Levels of nesting allowed in the call stack.
///
/// The original RCA 1802 implementation allocated 48 bytes
/// for up to 12 levels of nesting.
///
/// There is no practical reason to have this limitation anymore.
/// Increasing it does not affect the correctness of programs.
///
/// Keeping it a power-of-two allows for efficiently masking
/// the stack pointer.
pub const STACK_SIZE: usize = 0xFF;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_SIZE: [usize; 2] = [DISPLAY_WIDTH, DISPLAY_HEIGHT];
pub const DISPLAY_BUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
pub const DISPLAY_WIDTH_MASK: usize = DISPLAY_WIDTH - 1;
pub const DISPLAY_HEIGHT_MASK: usize = DISPLAY_HEIGHT - 1;

/// Type for storing the 12-bit memory addresses.
pub type Address = u16;
