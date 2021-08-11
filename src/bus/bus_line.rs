use std::cell::Cell;

/// Signal bus line
#[derive(Default)]
pub struct BusLine<T> {

    /// Line state
    state: Cell<Option<T>>,

}

impl<T: Copy + Default> BusLine<T> {

    /// Create new bus line
    pub fn new() -> Self {
        Default::default()
    }

    /// Sample signal line expecting it is driven by someone else
    pub fn sample(&self) -> Option<T> {
        self.state.get()
    }

    /// Drive signal line expecting it is not taken (driven) by others or panic otherwise.
    pub fn drive(&self, value: T) {
        match self.state.get() {
            Some(_) => panic!("Line is already taken"),
            None => self.state.set(Some(value)),
        }
    }

    /// Drive and release later using returned closure
    pub fn drive_and_release<'a>(&'a self, value: T) -> impl FnOnce() -> () + 'a {
        self.drive(value);
        move || self.state.set(None)
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn sample_returns_nothing_if_line_is_not_driven() {
        let line = BusLine::<bool>::new();
        assert_eq!(line.sample(), None);
    }

    #[test]
    fn sample_returns_nothing_if_line_is_driven_and_then_released() {
        let line = BusLine::<bool>::new();
        let release = line.drive_and_release(true);
        release();
        assert_eq!(line.sample(), None);
    }

    #[test]
    fn sample_returns_line_state_if_line_is_driven() {
        let line = BusLine::<bool>::new();
        line.drive(true);
        assert_eq!(line.sample(), Some(true));
    }

    #[test]
    fn drive_sets_a_new_line_state_if_line_already_released() {
        let line = BusLine::<bool>::new();
        let release = line.drive_and_release(true);
        assert_eq!(line.sample(), Some(true));
        release();
        line.drive(false);
        assert_eq!(line.sample(), Some(false));
    }

    #[test]
    #[should_panic(expected = "Line is already taken")]
    fn drive_panics_if_line_is_already_taken() {
        let line = BusLine::<bool>::new();
        line.drive(true);
        line.drive(false);
    }

}
