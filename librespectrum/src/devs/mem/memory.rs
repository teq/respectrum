use crate::devs::Device;

pub trait Memory: Device {

    /// Check if given address is writable (located in RAM)
    fn writable(&self, addr: u16) -> bool;

    /// Write byte to the memory
    fn write(&self, addr: u16, byte: u8);

    /// Read byte from the memory
    fn read(&self, addr: u16) -> u8;

}
