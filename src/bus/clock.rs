use std::cell::Cell;

/// System clock. Counts t-cycles with half t-cycle precision.
#[derive(Default)]
pub struct Clock {
    /// Half t-cycles count since system start. Even on rising, odd on falling.
    pub htcycles: Cell<u64>,
}

impl Clock {

    /// Half t-cycles count since system start
    pub fn htcycles(&self) -> u64 {
        self.htcycles.get()
    }

    /// T-cycles count since system start
    pub fn tcycles(&self) -> u64 {
        self.htcycles() >> 1
    }

    /// Get offset in half t-cycles to the next Nth t-cycle rising edge
    pub fn rising(&self, n: usize) -> usize {
        (n << 1) - (self.htcycles() & 1) as usize
    }

    /// Get offset in half t-cycles to the next Nth t-cycle falling edge
    pub fn falling(&self, n: usize) -> usize {
        (n << 1) - (!self.htcycles() & 1) as usize
    }

}
