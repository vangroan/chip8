//! CPU Clock.
use std::{thread, time::Instant};

use crate::constants::*;

/// Timer to synchronize thread with the software clock of the virtual CPU.
/// 
/// It is designed to work with the yielding cooperative pattern
/// of the interpreter loop. When the VM yields control back to the
/// caller, time elapses until it is resumed. Once the interpreter
/// is resumed, the elapsed time is taken into account when determining
/// the next cycle.
pub(crate) struct Clock(Instant);

impl Clock {
    /// Creates a new clock with the current time as internal state.
    pub(crate) fn new() -> Self {
        Self(Instant::now())
    }

    /// Set the clock state back to zero.
    pub(crate) fn reset(&mut self) {
        self.0 = Instant::now()
    }

    /// Block the current thread until the next clock cycle.
    pub(crate) fn wait(&mut self) {
        loop {
            let elapsed = self.0.elapsed().as_nanos();
            if elapsed < CLOCK_CYCLE_TIME {
                // Sleep does not have enough resolution, and causes
                // the clock to run at 30 FPS.
                //
                // Spinning a loop causes high CPU usage and fan madness.
                //
                // Yielding in a loop is the best alternative.
                thread::yield_now();
            } else {
                // Reset back to zero, rather than trying to catch up.
                //
                // If the VM was paused for debugging, and a large
                // amount of time has elapsed until it is resumed,
                // it should simply continue at the next cycle running
                // at its usual speed.
                self.reset();
                return;
            }
        }
    }
}
