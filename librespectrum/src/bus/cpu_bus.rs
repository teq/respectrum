use super::BusLine;

bitflags! {
    #[derive(Default)]
    pub struct Ctrl : u8 {
        const NONE = 0;
        const MREQ = 1 << 0;
        const IORQ = 1 << 1;
        const RD   = 1 << 2;
        const WR   = 1 << 3;
        const RFSH = 1 << 4;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Outs : u8 {
        const NONE  = 0;
        const M1    = 1 << 0;
        const HALT  = 1 << 1;
        const BUSAK = 1 << 2;
    }
}

/// Z80 CPU bus
pub struct CpuBus {
    /// Address bus (tri-state outputs)
    pub addr: BusLine<u16>,
    /// Data bus (tri-state in/outputs)
    pub data: BusLine<u8>,
    /// Tri-state control outputs
    pub ctrl: BusLine<Ctrl>,
    /// Control outputs
    pub outs: BusLine<Outs>,
    /// WAIT input
    pub wait: BusLine<bool>,
    /// INT input
    pub int: BusLine<bool>,
    /// NMI input
    pub nmi: BusLine<bool>,
    /// RESET input
    pub reset: BusLine<bool>,
    /// BUSRQ input
    pub busrq: BusLine<bool>,
}

impl Default for CpuBus {
    fn default() -> Self {
        Self {
            addr: BusLine::new("ADDR"),
            data: BusLine::new("DATA"),
            ctrl: BusLine::new("CTRL"),
            outs: BusLine::new("OUTS"),
            wait: BusLine::new("WAIT"),
            int: BusLine::new("INT"),
            nmi: BusLine::new("NMI"),
            reset: BusLine::new("RESET"),
            busrq: BusLine::new("BUSRQ"),
        }
    }
}
