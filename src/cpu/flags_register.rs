use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FlagsRegister {
    pub carry: bool,
    pub zero: bool,
    pub interrupt_disable: bool,
    pub decimal: bool,
    pub overflow: bool,
    pub negative: bool,
}

impl FlagsRegister {
    pub fn set(&mut self, flag: u8) {
        self.carry = flag & 0x01 > 0;
        self.zero = flag & 0x02 > 0;
        self.interrupt_disable = flag & 0x04 > 0;
        self.decimal = flag & 0x08 > 0;
        self.overflow = flag & 0x40 > 0;
        self.negative = flag & 0x80 > 0;
    }

    pub fn get(&self) -> u8 {
        return if self.carry {0x01u8} else {0u8} |
        if self.zero {0x02u8} else {0u8} |
        if self.interrupt_disable {0x04u8} else {0u8} |
                                0x20u8 |
        if self.decimal {0x08u8} else {0u8} |
        if self.overflow {0x40u8} else {0u8} |
        if self.negative {0x80u8} else {0u8}
    }
}