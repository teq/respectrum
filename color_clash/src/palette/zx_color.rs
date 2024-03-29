use druid::Data;

const PALETTE: [([u8; 3], [u8; 3]); 8] = [
    ([0x00, 0x00, 0x00], [0x00, 0x00, 0x00]),
    ([0x00, 0x00, 0xee], [0x00, 0x00, 0xff]),
    ([0xee, 0x00, 0x00], [0xff, 0x00, 0x00]),
    ([0xee, 0x00, 0xee], [0xff, 0x00, 0xff]),
    ([0x00, 0xee, 0x00], [0x00, 0xff, 0x00]),
    ([0x00, 0xee, 0xee], [0x00, 0xff, 0xff]),
    ([0xee, 0xee, 0x00], [0xff, 0xff, 0x00]),
    ([0xee, 0xee, 0xee], [0xff, 0xff, 0xff]),
];

#[derive(Debug, Default, Clone, Data)]
pub struct ZXColor {
    index: usize
}

impl ZXColor {

    pub fn palette() -> Vec<Self> {
        (0..PALETTE.len()).map(|index| Self{index}).collect()
    }

    pub fn new(index: usize) -> Self {
        assert!(index < PALETTE.len());
        Self { index }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn rgb_dim(&self) -> [u8; 3] {
        PALETTE[self.index as usize].0
    }

    pub fn rgb_bright(&self) -> [u8; 3] {
        PALETTE[self.index as usize].1
    }

}

impl<T> From<T> for ZXColor where T: Into<usize> {
    fn from(index: T) -> Self {
        Self::new(index.into())
    }
}

impl PartialEq for ZXColor {
    fn eq(&self, other: &ZXColor) -> bool {
        self.index == other.index
    }
}

impl<T> PartialEq<T> for ZXColor where T: Into<usize> + Copy {
    fn eq(&self, other: &T) -> bool {
        self.index == (*other).into()
    }
}
