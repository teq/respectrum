use super::BusLine;

bitflags! {
    #[derive(Default)]
    pub struct Ctls : u8 {
        const NONE = 0;
        const MREQ = 1 << 0;
        const IORQ = 1 << 1;
        const RD   = 1 << 2;
        const WR   = 1 << 3;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Outs : u8 {
        const NONE  = 0;
        const M1    = 1 << 0;
        const RFSH  = 1 << 1;
        const HALT  = 1 << 2;
        const BUSAK = 1 << 3;
    }
}

/// Z80 CPU bus
#[derive(Default)]
pub struct CpuBus {
    /// Address bus (tri-state outputs)
    pub addr: BusLine<u16>,
    /// Data bus (tri-state in/outputs)
    pub data: BusLine<u8>,
    /// Tri-state control outputs
    pub ctrl: BusLine<Ctls>,
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
