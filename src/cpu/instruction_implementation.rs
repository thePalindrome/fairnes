use std::ops::Shl;
use crate::cpu::cpu::{CPUState, CPU};
use crate::cpu::cpu::CPUState::{PendingModify, PendingWrite};
use crate::cpu::instruction::{AddressingMode, Instruction};

pub fn execute_instruction(instruction: Instruction, cpu: &mut CPU) {
    match instruction.name.as_str() {
        "HTL" => {cpu.state = CPUState::Halted;},
        "ADC" => {let result: u16 = cpu.a as u16 + instruction.value as u16 + if cpu.p.carry { 1 } else { 0 } as u16; cpu.p.overflow = (cpu.a > 127) == (instruction.value > 127) && (cpu.a > 127) != (result & 0xFF > 127); cpu.p.carry = result > 0xFF; cpu.a = result as u8; update_zero_and_negative(cpu.a, cpu) },
        "AND" => {cpu.a = cpu.a & instruction.value; update_zero_and_negative(cpu.a, cpu)},
        "ASL" => {if instruction.mode == AddressingMode::Implied {cpu.p.carry = cpu.a > 127; cpu.a = cpu.a << 1; update_zero_and_negative(cpu.a, cpu)} else {cpu.p.carry = instruction.value > 127; cpu.state = PendingModify {address: instruction.address, original: instruction.value, value: instruction.value.unbounded_shl(1)}; update_zero_and_negative(instruction.value.unbounded_shl(1), cpu) }}
        "BIT" => {cpu.p.zero = (cpu.a & instruction.value) == 0; cpu.p.negative = (instruction.value & 0x80) != 0; cpu.p.overflow = (instruction.value & 0x40) != 0}
        "BPL" => {if !cpu.p.negative {cpu.state = CPUState::Jumping {address: offset_jump(instruction.value, cpu), penalty: false}}},
        "BMI" => {if cpu.p.negative  {cpu.state = CPUState::Jumping {address: offset_jump(instruction.value, cpu), penalty: false}}},
        "BRK" => {if instruction.value == 0 {cpu.pc = cpu.pc.wrapping_add(1); cpu.state = CPUState::Pushing16{value: cpu.pc}; cpu.instruction.value = 1} else if instruction.value == 1 {cpu.state = CPUState::Pushing{value: cpu.p.get() | 0x10}; cpu.instruction.value = 2} else {cpu.state = CPUState::PendingRead16{address: 0xFFFE}}}
        "BVC" => {if !cpu.p.overflow {cpu.state = CPUState::Jumping {address: offset_jump(instruction.value, cpu), penalty: false}}},
        "BVS" => {if cpu.p.overflow  {cpu.state = CPUState::Jumping {address: offset_jump(instruction.value, cpu), penalty: false}}},
        "BCC" => {if !cpu.p.carry {cpu.state = CPUState::Jumping {address: offset_jump(instruction.value, cpu), penalty: false}}},
        "BCS" => {if cpu.p.carry  {cpu.state = CPUState::Jumping {address: offset_jump(instruction.value, cpu), penalty: false}}},
        "BNE" => {if !cpu.p.zero  {cpu.state = CPUState::Jumping {address: offset_jump(instruction.value, cpu), penalty: false}}},
        "BEQ" => {if cpu.p.zero   {cpu.state = CPUState::Jumping {address: offset_jump(instruction.value, cpu), penalty: false}}},
        "CMP" => {cpu.p.carry = instruction.value <= cpu.a; cpu.p.zero = instruction.value == cpu.a; cpu.p.negative = cpu.a.wrapping_sub(instruction.value) > 127}
        "CPX" => {cpu.p.carry = instruction.value <= cpu.x; cpu.p.zero = instruction.value == cpu.x; cpu.p.negative = cpu.x.wrapping_sub(instruction.value) > 127}
        "CPY" => {cpu.p.carry = instruction.value <= cpu.y; cpu.p.zero = instruction.value == cpu.y; cpu.p.negative = cpu.y.wrapping_sub(instruction.value) > 127}
        "CLC" => {cpu.p.carry = false}
        "CLD" => {cpu.p.decimal = false}
        "CLI" => {cpu.p.interrupt_disable = false}
        "CLV" => {cpu.p.overflow = false}
        "DEC" => {let result = instruction.value.wrapping_sub(1); cpu.state = CPUState::PendingModify{ address: instruction.address, original: instruction.value, value: result}; update_zero_and_negative(result, cpu) }
        "DEX" => {cpu.x = cpu.x.wrapping_sub(1); update_zero_and_negative(cpu.x, cpu)},
        "DEY" => {cpu.y = cpu.y.wrapping_sub(1); update_zero_and_negative(cpu.y, cpu)},
        "EOR" => {cpu.a = cpu.a ^ instruction.value; update_zero_and_negative(cpu.a, cpu)},
        "INC" => {let result = instruction.value.wrapping_add(1); cpu.state = CPUState::PendingModify{ address: instruction.address, original: instruction.value, value: result}; update_zero_and_negative(result, cpu) }
        "INX" => {cpu.x = cpu.x.wrapping_add(1); update_zero_and_negative(cpu.x, cpu)},
        "INY" => {cpu.y = cpu.y.wrapping_add(1); update_zero_and_negative(cpu.y, cpu)},
        "JMP" => {cpu.pc = instruction.address},
        "JSR" => {cpu.state = CPUState::Pushing16 { value: cpu.pc.wrapping_sub(1)}}
        "LDA" => {cpu.a = instruction.value; update_zero_and_negative(cpu.a, cpu)}
        "LDX" => {cpu.x = instruction.value; update_zero_and_negative(cpu.x, cpu)}
        "LDY" => {cpu.y = instruction.value; update_zero_and_negative(cpu.y, cpu)}
        "LSR" => {if instruction.mode == AddressingMode::Implied {cpu.p.carry = cpu.a & 1 == 1; cpu.a >>= 1; update_zero_and_negative(cpu.a, cpu)} else {cpu.p.carry = instruction.value & 1 == 1; cpu.state = PendingModify {address: instruction.address, original: instruction.value, value: instruction.value >> 1}; update_zero_and_negative(instruction.value >> 1, cpu) }}
        "NOP" => {}, // NOP is silly :P
        "ORA" => {cpu.a = cpu.a | instruction.value; update_zero_and_negative(cpu.a, cpu)},
        "PHA" => {cpu.state = CPUState::Pushing {value: cpu.a};},
        "PHP" => {cpu.state = CPUState::Pushing {value: cpu.p.get() | 0x30};},
        "PLA" => {cpu.a = instruction.value; update_zero_and_negative(cpu.a, cpu)}
        "PLP" => {cpu.p.set(instruction.value)},
        "ROL" => {let old_carry = if cpu.p.carry {1} else {0}; if instruction.mode == AddressingMode::Implied {cpu.p.carry = cpu.a > 127; cpu.a = (cpu.a << 1).wrapping_add(old_carry); update_zero_and_negative(cpu.a, cpu)} else {cpu.p.carry = instruction.value > 127; cpu.state = PendingModify {address: instruction.address, original: instruction.value, value: (instruction.value << 1) + old_carry}; update_zero_and_negative((instruction.value << 1) + old_carry, cpu) }}
        "ROR" => {let old_carry = if cpu.p.carry {128} else {0}; if instruction.mode == AddressingMode::Implied {cpu.p.carry = cpu.a & 1 == 1; cpu.a = cpu.a >> 1; cpu.a += old_carry; update_zero_and_negative(cpu.a, cpu)} else {cpu.p.carry = instruction.value & 1 == 1; cpu.state = PendingModify {address: instruction.address, original: instruction.value, value: instruction.value.unbounded_shr(1) + old_carry}; update_zero_and_negative(instruction.value.unbounded_shr(1) + old_carry, cpu) }}
        "RTI" => {if instruction.value == 0 {cpu.state = CPUState::Pulling}}
        "RTS" => {if !cpu.lo_set {cpu.state = CPUState::Pulling16; cpu.lo_set = true;} else {cpu.pc = ((instruction.value as u16) << 8) + cpu.lo as u16 + 1; cpu.lo_set = false}} // Gross to use two READY states, but phew
        "SBC" => {let result: i16 = cpu.a as i16 - instruction.value as i16 - if cpu.p.carry {0} else {1}; cpu.p.overflow = ((cpu.a as i16 ^ instruction.value as i16) & (cpu.a as i16 ^ result) & 0x80) != 0; cpu.p.carry = result >= 0; cpu.a = result as u8; update_zero_and_negative(cpu.a, cpu)}
        "SEC" => {cpu.p.carry = true}
        "SED" => {cpu.p.decimal = true}
        "SEI" => {cpu.p.interrupt_disable = true}
        "STA" => { cpu.state = PendingWrite { address: build_address(instruction,cpu), value: cpu.a }}
        "STX" => { cpu.state = PendingWrite { address: build_address(instruction,cpu), value: cpu.x }}
        "STY" => { cpu.state = PendingWrite { address: build_address(instruction,cpu), value: cpu.y }}
        "TAX" => { cpu.x = cpu.a; update_zero_and_negative(cpu.x, cpu)}
        "TAY" => { cpu.y = cpu.a; update_zero_and_negative(cpu.a, cpu)}
        "TSX" => { cpu.x = cpu.sp; update_zero_and_negative(cpu.x, cpu)}
        "TXA" => { cpu.a = cpu.x; update_zero_and_negative(cpu.a, cpu)}
        "TXS" => { cpu.sp = cpu.x}
        "TYA" => { cpu.a = cpu.y; update_zero_and_negative(cpu.a, cpu)}
        _ => {}
    }
}

fn build_address(instruction: Instruction, cpu: &mut CPU) -> u16 {
    if instruction.mode == AddressingMode::Absolute {
        ((instruction.value as u16) << 8) + cpu.lo as u16
    } else if instruction.mode == AddressingMode::AbsoluteX  || instruction.mode == AddressingMode::AbsoluteY
        || instruction.mode == AddressingMode::ZeroPageX || instruction.mode == AddressingMode::ZeroPageY
        || instruction.mode == AddressingMode::XIndirect || instruction.mode == AddressingMode::IndirectY {
        instruction.address
    } else {
        instruction.value as u16
    }
}

fn offset_jump(value: u8, cpu: &mut CPU) -> u16 {
    let offset = if value > 127 { value as i16 - 256 } else {value as i16};
    ((cpu.pc as i32).wrapping_add(offset as i32) as u16).wrapping_add(0) // TODO: This smells like it could break if we branch below 0 or above 0xFFFF
}

fn update_zero_and_negative(value: u8, cpu: &mut CPU) {
    cpu.p.zero = value == 0;
    cpu.p.negative = value > 127;
}