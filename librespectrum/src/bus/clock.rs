use std::cell::Cell;

/// System clock. Counts cycles with half t-cycle precision.
#[derive(Default)]
pub struct Clock {

    /// Half t-cycles count since system start. Even on rising, odd on falling.
    htcycles: Cell<u64>,

}

impl Clock {

    /// Set clock in half t-cycles
    pub fn set(&self, val: u64) {
        self.htcycles.set(val);
    }

    /// Get clock in half t-cycles
    pub fn get(&self) -> u64 {
        self.htcycles.get()
    }

    /// Get offset in half t-cycles to the next Nth t-cycle rising edge
    pub fn rising(&self, n: usize) -> usize {
        (n << 1) - (self.get() & 1) as usize
    }

    /// Get offset in half t-cycles to the next Nth t-cycle falling edge
    pub fn falling(&self, n: usize) -> usize {
        (n << 1) - (!self.get() & 1) as usize
    }

}
