use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AddressingMode {
    #[default]
    Implied,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    XIndirect,
    IndirectY,
    Indirect,
    Relative,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub opcode: u8,
    pub name: String,
    pub mode: AddressingMode,
    pub value: u8,
    pub address: u16,
    pub does_write: bool
}

impl Instruction {
    pub fn new(opcode: u8, name: &str, mode: AddressingMode, does_write: bool) -> Instruction {
        Instruction {
            opcode, name: name.to_string(), mode, value: 0, address: 0, does_write
        }
    }
}