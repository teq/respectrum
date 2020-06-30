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
            addr:  BusLine::new("addr"),
            data:  BusLine::new("data"),
            ctrl:  BusLine::new("ctrl"),
            outs:  BusLine::new("outs"),

            wait:  BusLine::new("wait"),
            int:   BusLine::new("int"),
            nmi:   BusLine::new("nmi"),
            reset: BusLine::new("reset"),
            busrq: BusLine::new("busrq"),
        }
    }

}
