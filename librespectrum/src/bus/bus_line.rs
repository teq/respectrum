use std::cell::Cell;

use crate::misc::Identifiable;

/// Signal bus line
pub struct BusLine<T> {

    /// Line name
    name: &'static str,

    /// Line owner and state
    state: Cell<Option<(u32, T)>>,

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

    /// Get line owner (if any)
    pub fn owner(&self) -> Option<u32> {
        self.state.get().and_then(|(owner, _)| Some(owner))
    }

    /// Probe signal line
    pub fn probe(&self) -> Option<T> {
        self.state.get().and_then(|(_, value)| Some(value))
    }

    /// Expect signal on the line
    pub fn expect(&self) -> T {
        self.probe().unwrap()
    }

    /// Drive signal line
    pub fn drive<U: Identifiable>(&self, device: &U, value: T) {
        match self.state.get() {
            None => self.state.set(Some((device.id(), value))),
            Some((owner, _)) if owner == device.id() => self.state.set(Some((owner, value))),
            Some((owner, _)) => panic!("Device {} conflicts with {} on a bus line {}", device.id(), owner, self.name)
        }
    }

    /// Release signal line
    pub fn release<U: Identifiable>(&self, device: &U) {
        match self.state.get() {
            Some((owner, _)) if owner == device.id() => self.state.set(None),
            None | Some((_, _)) => panic!("Device {} doesn't own the line {}", device.id(), self.name)
        }
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    fn mkline() -> BusLine::<bool> { BusLine::<bool>::new("Test line") }

    struct TestDevice {
        id: u32
    }

    impl Identifiable for TestDevice {
        fn id(&self) -> u32 { self.id }
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
    #[should_panic(expected="conflict")]
    fn only_one_device_can_drive_the_line() {
        let line = mkline();
        line.drive(&DEV1, false);
        line.drive(&DEV2, true);
    }

}
