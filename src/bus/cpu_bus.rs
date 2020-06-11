use super::BusLine;

/// Z80 CPU bus
pub struct CpuBus {
    pub addr:  BusLine<u16>,
    pub data:  BusLine<u8>,
    pub m1:    BusLine<bool>,
    pub mreq:  BusLine<bool>,
    pub iorq:  BusLine<bool>,
    pub rd:    BusLine<bool>,
    pub wr:    BusLine<bool>,
    pub rfsh:  BusLine<bool>,
    pub halt:  BusLine<bool>,
    pub wait:  BusLine<bool>,
    pub int:   BusLine<bool>,
    pub nmi:   BusLine<bool>,
    pub reset: BusLine<bool>,
    pub busrq: BusLine<bool>,
    pub busak: BusLine<bool>,
}

impl CpuBus {

    /// Create new CPU bus
    pub fn new() -> CpuBus {
        CpuBus {
            addr:  BusLine::new("addr"),
            data:  BusLine::new("data"),
            m1:    BusLine::new("m1"),
            mreq:  BusLine::new("mreq"),
            iorq:  BusLine::new("iorq"),
            rd:    BusLine::new("rd"),
            wr:    BusLine::new("wr"),
            rfsh:  BusLine::new("rfsh"),
            halt:  BusLine::new("halt"),
            wait:  BusLine::new("wait"),
            int:   BusLine::new("int"),
            nmi:   BusLine::new("nmi"),
            reset: BusLine::new("reset"),
            busrq: BusLine::new("busrq"),
            busak: BusLine::new("busak"),
        }
    }

}
