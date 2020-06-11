use std::cell::Cell;

/// Signal bus line
pub struct BusLine<T: Copy> {
    name: &'static str,
    state: Cell<Option<T>>,
}

impl<T: Copy> BusLine<T> {

    /// Create new bus line
    pub fn new(name: &'static str) -> BusLine<T> {
        BusLine { name, state: Cell::new(None) }
    }

    /// Sample signal line expecting it is driven by someone else
    pub fn sample(&self) -> Option<T> {
        self.state.get()
    }

    /// Drive signal line expecting it is not taken (driven) by others or panic otherwise.
    /// Returns a closure which releases a line when called
    pub fn drive<'a>(&'a self, value: T) -> impl FnOnce() -> () + 'a {
        match self.state.get() {
            Some(_) => panic!("Bus line [{}] is already taken", self.name),
            None => self.state.set(Some(value)),
        }
        move || self.state.set(None)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_returns_nothing_if_line_is_not_driven() {
        let line: BusLine<bool> = BusLine::new("test");
        assert_eq!(line.sample(), None);
    }

    #[test]
    fn sample_returns_nothing_if_line_is_driven_and_then_released() {
        let line: BusLine<bool> = BusLine::new("test");
        let release = line.drive(true);
        release();
        assert_eq!(line.sample(), None);
    }

    #[test]
    fn sample_returns_line_state_if_line_is_driven() {
        let line: BusLine<bool> = BusLine::new("test");
        let _ = line.drive(true);
        assert_eq!(line.sample(), Some(true));
    }

    #[test]
    fn drive_sets_a_new_line_state_if_line_already_released() {
        let line: BusLine<bool> = BusLine::new("test");
        let release = line.drive(true);
        assert_eq!(line.sample(), Some(true));
        release();
        let _ = line.drive(false);
        assert_eq!(line.sample(), Some(false));
    }

    #[test]
    #[should_panic(expected = "Bus line [test] is already taken")]
    fn drive_panics_if_line_is_already_taken() {
        let line: BusLine<bool> = BusLine::new("test");
        let _ = line.drive(true);
        let _ = line.drive(false);
    }

}
