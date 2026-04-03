use serde::{Deserialize, Serialize};
use crate::cpu::flags_register::FlagsRegister;
use crate::cpu::instruction::Instruction;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum CPUState {
    #[default]
    Halted,
    NeedInstruction,
    NeedOperand,
    Ready,
    Pushing{value: u8},
    Pushing16{value: u16},
    Pulling,
    Pulling16,
    PendingRead{address: u16},
    PendingRead16{address: u16},
    PendingWrite{address: u16, value: u8},
    PendingModify{address: u16, original: u8, value: u8},
    Jumping{address: u16, penalty: bool},
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct CPU {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: FlagsRegister,
    pub sp: u8,
    pub state: CPUState,
    pub instruction: Instruction,

    pub lo: u8,
    pub lo_set: bool,

}

impl CPU {
    pub fn new() -> CPU {
        Self {
            ..Default::default()
        }
    }
}