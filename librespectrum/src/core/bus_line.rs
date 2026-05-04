use std::cell::Cell;

use super::{Identifiable, Identifier};

/// Signal bus line
pub struct BusLine<T> {

    /// Line name
    name: &'static str,

    /// Line owner and state
    state: Cell<Option<(Identifier, T)>>,

}

impl<T: Copy> BusLine<T> {

    /// Create new bus line
    pub fn new(name: &'static str) -> Self {
        Self { name, state: Cell::new(None) }
    }

    /// Bus line name
    pub fn name(&self) -> &str {
        self.name
    }

    /// Get line state (owner and value)
    pub fn state(&self) -> Option<(Identifier, T)> {
        self.state.get()
    }

    /// Get line owner (if any)
    pub fn owner(&self) -> Option<Identifier> {
        self.state.get().and_then(|(owner, ..)| Some(owner))
    }

    /// Probe signal line
    pub fn probe(&self) -> Option<T> {
        self.state.get().and_then(|(.., value)| Some(value))
    }

    /// Expect signal on the line
    pub fn expect(&self) -> T {
        self.probe().unwrap()
    }

    /// Drive signal line
    pub fn drive<U: Identifiable>(&self, device: &U, value: T) {
        match self.state.get() {
            None => self.state.set(Some((device.id(), value))),
            Some((owner, ..)) if owner == device.id() => self.state.set(Some((owner, value))),
            Some((owner, ..)) => panic!("Device {} tries to drive the line {} owned by {}", device.id(), self.name, owner)
        }
    }

    /// Release signal line
    pub fn release<U: Identifiable>(&self, device: &U) {
        match self.state.get() {
            Some((owner, ..)) if owner == device.id() => self.state.set(None),
            _ => ()
        }
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    fn mkline() -> BusLine::<bool> { BusLine::<bool>::new("Test line") }

    struct TestDevice {
        id: Identifier
    }

    impl Identifiable for TestDevice {
        fn id(&self) -> Identifier { self.id }
    }

    static DEV1: TestDevice = TestDevice { id: 1 };
    static DEV2: TestDevice = TestDevice { id: 2 };

    #[test]
    fn line_drive_and_probe() {
        let line = mkline();
        assert_eq!(line.probe(), None);
        line.drive(&DEV1, false);
        assert_eq!(line.probe(), Some(false));
        line.drive(&DEV1, true);
        assert_eq!(line.probe(), Some(true));
        line.release(&DEV1);
        assert_eq!(line.probe(), None);
        line.drive(&DEV2, true);
        assert_eq!(line.probe(), Some(true));
    }

    #[test]
    #[should_panic]
    fn only_one_device_can_drive_the_line() {
        let line = mkline();
        line.drive(&DEV1, false);
        line.drive(&DEV2, true);
    }

}
