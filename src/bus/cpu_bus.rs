use super::BusLine;

bitflags! {
    pub struct Ctls : u8 {
        const NONE = 0;
        const MREQ = 1 << 0;
        const IORQ = 1 << 1;
        const RD   = 1 << 2;
        const WR   = 1 << 3;
    }
}

bitflags! {
    pub struct Outs : u8 {
        const NONE  = 0;
        const M1    = 1 << 0;
        const RFSH  = 1 << 1;
        const HALT  = 1 << 2;
        const BUSAK = 1 << 3;
    }
}

/// Z80 CPU bus
pub struct CpuBus {
    /// Address bus (tri-state outputs)
    pub addr:  BusLine<u16>,
    /// Data bus (tri-state in/outputs)
    pub data:  BusLine<u8>,
    /// Tri-state control outputs: MREQ, IORQ, RD, WR
    pub ctrl:  BusLine<Ctls>,
    /// Control outputs: M1, RFSH, HALT, BUSAK
    pub outs:  BusLine<Outs>,

    pub wait:  BusLine<bool>, // |
    pub int:   BusLine<bool>, // |
    pub nmi:   BusLine<bool>, // | inputs
    pub reset: BusLine<bool>, // |
    pub busrq: BusLine<bool>, // |
}

impl CpuBus {

    /// Create new CPU bus instance
    pub fn new() -> CpuBus {
        CpuBus {
            addr:  BusLine::new("CPU::address_lines"),
            data:  BusLine::new("CPU::data_lines"),
            ctrl:  BusLine::new("CPU::control_lines"),
            outs:  BusLine::new("CPU::output_lines"),

            wait:  BusLine::new("CPU::wait"),
            int:   BusLine::new("CPU::int"),
            nmi:   BusLine::new("CPU::nmi"),
            reset: BusLine::new("CPU::reset"),
            busrq: BusLine::new("CPU::busrq"),
        }
    }

}
