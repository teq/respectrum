use std::rc::Rc;

use super::{BusLine, Clock};

pub const MREQ:  u8 = 1 << 0;
pub const IORQ:  u8 = 1 << 1;
pub const RD:    u8 = 1 << 2;
pub const WR:    u8 = 1 << 3;

pub const M1:    u8 = 1 << 0;
pub const RFSH:  u8 = 1 << 1;
pub const HALT:  u8 = 1 << 2;
pub const BUSAK: u8 = 1 << 3;

/// Z80 CPU bus
pub struct CpuBus {
    /// Address bus (tri-state outputs)
    pub addr:  BusLine<u16>,
    /// Data bus (tri-state in/outputs)
    pub data:  BusLine<u8>,
    /// Tri-state control outputs: MREQ, IORQ, RD, WR
    pub ctrl:  BusLine<u8>,
    /// Control outputs: M1, RFSH, HALT, BUSAK
    pub outs:  BusLine<u8>,

    pub wait:  BusLine<bool>, // |
    pub int:   BusLine<bool>, // |
    pub nmi:   BusLine<bool>, // | inputs
    pub reset: BusLine<bool>, // |
    pub busrq: BusLine<bool>, // |

    pub clock: Rc<Clock>,
}

impl CpuBus {

    /// Create new CPU bus instance
    pub fn new(clock: Rc<Clock>) -> CpuBus {
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

            clock,
        }
    }

}
