use std::cell::Cell;

use crate::misc::Identifiable;

/// Signal bus line
pub struct BusLine<T> {

    /// Line name
    name: &'static str,

    /// Line state and its current owner
    state: Cell<Option<(T, &'static str)>>,

}

impl<T: Copy> BusLine<T> {

    /// Create new bus line
    pub fn new(name: &'static str) -> Self {
        Self { name, state: Default::default() }
    }

    /// Bus line name
    pub fn name(&self) -> &str {
        self.name
    }

    /// Sample signal line
    pub fn sample(&self) -> Option<T> {
        match self.state.get() {
            Some((val, _))  => Some(val),
            None => None
        }
    }

    /// Drive signal line expecting it is not taken (driven) by others or panic otherwise.
    pub fn drive(&self, device: &dyn Identifiable, value: T) {
        match self.state.get() {
            Some((_, taken_by)) => panic!(
                "Device {} tries to drive {} while it's taken by {}",
                device.id(), self.name, taken_by
            ),
            None => self.state.set(Some((value, device.id()))),
        }
    }

    /// Drive and release later using returned closure
    pub fn drive_and_release<'a>(&'a self, device: &dyn Identifiable, value: T) -> impl FnOnce() -> () + 'a {
        self.drive(device, value);
        move || self.state.set(None)
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    fn mkline() -> BusLine::<bool> { BusLine::<bool>::new("Test line") }

    struct TestDevice {
        pub name: &'static str
    }

    impl Identifiable for TestDevice {
        fn id(&self) -> &'static str {
            self.name
        }
    }

    static dev: TestDevice = TestDevice { name: "Test device" };

    #[test]
    fn sample_returns_nothing_if_line_is_not_driven() {
        let line = mkline();
        assert_eq!(line.sample(), None);
    }

    #[test]
    fn sample_returns_nothing_if_line_is_driven_and_then_released() {
        let line = mkline();
        let release = line.drive_and_release(&dev, true);
        release();
        assert_eq!(line.sample(), None);
    }

    #[test]
    fn sample_returns_line_state_if_line_is_driven() {
        let line = mkline();
        line.drive(&dev, true);
        assert_eq!(line.sample(), Some(true));
    }

    #[test]
    fn drive_sets_a_new_line_state_if_line_already_released() {
        let line = mkline();
        let release = line.drive_and_release(&dev, true);
        assert_eq!(line.sample(), Some(true));
        release();
        line.drive(&dev, false);
        assert_eq!(line.sample(), Some(false));
    }

    #[test]
    #[should_panic]
    fn drive_panics_if_line_is_already_taken() {
        let line = mkline();
        line.drive(&dev, true);
        line.drive(&dev, false);
    }

}
