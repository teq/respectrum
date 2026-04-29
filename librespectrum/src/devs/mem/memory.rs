use crate::devs::Device;

pub trait Memory: Device {

    /// Check if given address is writable (located in RAM)
    fn writable(&self, addr: u16) -> bool;

    /// Write byte to the memory
    fn write(&self, addr: u16, byte: u8);

    /// Read byte from the memory
    fn read(&self, addr: u16) -> u8;

}

pub enum MemoryBreakpoint {
    Access(u16), // Break on access to given address
    AccessRange(u16, u16), // Break on access to given address range (inclusive)
    Write(u16), // Break on write to given address
    Read(u16), // Break on read from given address
}

