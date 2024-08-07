
/// Merge high and low u8 bytes to a u16 word
#[macro_export]
macro_rules! mkword {
    ($high: expr, $low: expr) => { (($high as u16) << 8) | $low as u16 }
}

/// Split u16 word to high and low u8 bytes
#[macro_export]
macro_rules! spword {
    ($word: expr) => {
        {
            let word = $word as u16;
            ((word >> 8) as u8, (word & 0xff) as u8)
        }
    }
}

/// Yield results from generator from within another generator.
/// Analogous to python's `yield from` syntax
#[macro_export]
macro_rules! yield_from {
    ($input: expr) => {
        {
            let mut task = $input;
            loop {
                match std::pin::Pin::new(&mut task).resume(()) {
                    std::ops::CoroutineState::Yielded(some) => yield some,
                    std::ops::CoroutineState::Complete(result) => break result
                }
            }
        }
    }
}
