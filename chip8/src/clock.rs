//! CPU Clock.
use std::{
    thread,
    time::{Duration, Instant},
};

/// Timer to synchronize thread with the software clock of the virtual CPU.
///
/// It is designed to work with the yielding cooperative pattern
/// of the interpreter loop. When the VM yields control back to the
/// caller, time elapses until it is resumed. Once the interpreter
/// is resumed, the elapsed time is taken into account when determining
/// the next cycle.
#[allow(dead_code)]
pub(crate) struct Clock {
    /// Expected duration of one clock cycle, in nanoseconds.
    /// Stored as integer to avoid calculating nanos on each frame.
    interval: u128,
    /// Last time measuement
    last: Instant,
}

#[allow(dead_code)]
impl Clock {
    /// Creates a new clock with the current time as internal state.
    pub(crate) fn new(interval: Duration) -> Self {
        Self {
            interval: interval.as_nanos(),
            last: Instant::now(),
        }
    }

    pub(crate) fn from_nanos(nano_seconds: u64) -> Self {
        Self {
            interval: nano_seconds as u128,
            last: Instant::now(),
        }
    }

    /// Set the clock state back to zero.
    pub(crate) fn reset(&mut self) {
        self.last = Instant::now()
    }

    /// Block the current thread until the next clock cycle.
    pub(crate) fn wait(&mut self) {
        loop {
            let elapsed = self.last.elapsed().as_nanos();
            // if elapsed < self.interval {
            if elapsed < self.interval {
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

    /// Returns true when the next clock cycle has been reached.
    pub(crate) fn tick(&mut self) -> bool {
        let elapsed = self.last.elapsed().as_nanos();
        if elapsed < self.interval {
            false
        } else {
            // Reset back to zero, rather than trying to catch up.
            self.reset();
            true
        }
    }
}
