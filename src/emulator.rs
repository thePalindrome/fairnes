use crate::cpu::cpu::{CPUState, CPU};
use crate::cpu::cpu::CPUState::Halted;
use crate::cpu::instruction::AddressingMode;
use crate::cpu::instruction_implementation::execute_instruction;
use crate::cpu::instruction_table::parse_instruction;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Emulator {
    #[serde(skip)]
    pub last_text: String,

    pub cpu: CPU,

    ram: Vec<u8>,

    // TODO: Replace with generic/NROM mapper handler
    rom: Vec<u8>,

    mdr: u8,

}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            last_text: String::new(),
            cpu: CPU::new(),
            ram: vec![0; 0x800],

            // See above TODO
            rom: Vec::with_capacity(0x8000),

            mdr: 0,
        }
    }
}

impl Emulator {
    pub fn load_cartridge(&mut self, bytes: Vec<u8>) {
        if bytes.len() < 16400 { // If less than one PRG ROM bank and the header, it has to be invalid.
            self.last_text = "ROM too small, corruption occurred.".to_string();
            return;
        }

        if bytes[0] != 0x4E || bytes[1] != 0x45 || bytes[2] != 0x53 || bytes[3] != 0x1A {
            self.last_text = "Missing magic number, not a .nes file?".to_string();
            return;
        }

        // TODO: Actually store the header :P

        // TODO: Check for the trainer padding >.>

        self.rom = Vec::from(&bytes[16..]);
        self.reset();
    }

    pub fn reset(&mut self) {
        let pcl = self.mem_read(0xFFFC);
        let pch = self.mem_read(0xFFFD);
        self.cpu.pc = ((pch as u16) << 8) + pcl as u16;
        self.cpu.state = CPUState::NeedInstruction;
        self.cpu.p.interrupt_disable = true;
        self.cpu.sp = 0xFD; // TODO: Handle the resetting popping 'properly'
        self.last_text = format!("Resetting PC to {:x}", self.cpu.pc);
    }

    pub fn run_frame(&mut self) {
        if self.cpu.state == Halted {
            return;
        }
        let mut cycles_this_frame = 0;

        while cycles_this_frame < 29780 { // Approximately how many CPU cycles run this frame
            cycles_this_frame += 1;
            self.master_cycle();
            if self.cpu.state == Halted {
                break;
            }
        }
    }

    pub(crate) fn master_cycle(&mut self) {
        self.cpu_cycle();
    }

    fn cpu_cycle(&mut self) {
        if self.cpu.state != Halted {
            match self.cpu.state {
                CPUState::NeedInstruction => {
                    self.cpu.instruction = parse_instruction(self.mem_read(self.cpu.pc));
                    if self.cpu.instruction.mode != AddressingMode::Implied {
                        self.cpu.state = CPUState::NeedOperand;
                    } else {
                        self.cpu.state = CPUState::Ready;
                    }
                    if self.cpu.instruction.name == "PLA" || self.cpu.instruction.name == "PLP" {
                        self.cpu.state = CPUState::Pulling; // Dirty hack, eww
                    }
                    self.cpu.pc = self.cpu.pc.wrapping_add(1);
                },

                CPUState::NeedOperand => {
                    match self.cpu.instruction.mode {
                        AddressingMode::Immediate | AddressingMode::Relative => {self.cpu.instruction.value = self.mem_read(self.cpu.pc); self.cpu.state = CPUState::Ready},
                        AddressingMode::ZeroPage => {
                            let address = self.mem_read(self.cpu.pc);
                            self.cpu.instruction.address = address as u16;
                            if self.cpu.instruction.does_write {
                                self.cpu.instruction.value = address;
                                self.cpu.state = CPUState::Ready;
                            } else {
                                self.cpu.state = CPUState::PendingRead { address: address as u16 };
                            }},
                        AddressingMode::Absolute => {
                            if self.cpu.lo_set {
                                self.cpu.instruction.value = self.mem_read(self.cpu.pc);
                                let full_address = ((self.cpu.instruction.value as u16) << 8) + self.cpu.lo as u16;
                                self.cpu.instruction.address = full_address;
                                if self.cpu.instruction.does_write || self.cpu.instruction.name == "JMP" {
                                    self.cpu.state = CPUState::Ready;
                                } else {
                                    self.cpu.state = CPUState::PendingRead { address: full_address};
                                }
                                self.cpu.lo_set = false;
                            } else {
                                self.cpu.lo = self.mem_read(self.cpu.pc);
                                self.cpu.lo_set = true;
                            }
                        }
                        _ => {}
                    }
                    self.cpu.pc = self.cpu.pc.wrapping_add(1);
                    if self.cpu.state == CPUState::Ready {
                        execute_instruction(self.cpu.instruction.clone(), &mut self.cpu);

                        if self.cpu.state == CPUState::Ready {
                            self.cpu.state = CPUState::NeedInstruction;
                        }
                    }
                },

                CPUState::PendingRead{ address } => {
                    match self.cpu.instruction.mode {
                        AddressingMode::ZeroPage | AddressingMode::Absolute | AddressingMode::Implied => {
                            self.cpu.instruction.value = self.mem_read(address);
                            self.cpu.state = CPUState::Ready;

                            if self.cpu.instruction.name == "RTI" { // Really gross to put this here, but the cycles are being weird.
                                self.cpu.sp = self.cpu.sp.wrapping_add(1);
                                let pc_lo = self.mem_read(0x100 + self.cpu.sp as u16);
                                self.cpu.sp = self.cpu.sp.wrapping_add(1);
                                let pc_hi = self.mem_read(0x100 + self.cpu.sp as u16);
                                self.cpu.state = CPUState::Jumping {address: ((pc_hi as u16) << 8) + pc_lo as u16, penalty: true}; // bypass penalty... I think.
                            } else if self.cpu.instruction.name == "BRK" {
                                self.cpu.state = CPUState::Jumping {address: ((self.cpu.instruction.value as u16) << 8) + self.cpu.lo as u16, penalty: true};
                            }
                        },


                        _ => {},
                    }
                },

                CPUState::PendingRead16{ address } => {
                    self.cpu.lo = self.mem_read(address);
                    self.cpu.state = CPUState::PendingRead{ address: address.wrapping_add(1) };
                }

                CPUState::PendingWrite {address, value} => {
                    println!("Writing {:X} to {:X}", value, address);
                    self.mem_write(address, value);
                    self.cpu.state = CPUState::NeedInstruction;
                }

                CPUState::PendingModify{address, original, value} => {
                    self.mem_write(address, original);
                    self.cpu.state = CPUState::PendingWrite{address, value};
                }

                CPUState::Ready => {
                    execute_instruction(self.cpu.instruction.clone(), &mut self.cpu);

                    if self.cpu.state == CPUState::Ready {
                        self.cpu.state = CPUState::NeedInstruction;
                    }
                }

                CPUState::Jumping {address, penalty} => {
                    if address & 0xFF00 != self.cpu.pc & 0xFF00 && penalty == false {
                        self.cpu.state = CPUState::Jumping { address, penalty: true };
                    } else {
                        self.cpu.pc = address;
                        self.cpu.state = CPUState::NeedInstruction;
                    }
                }

                CPUState::Pushing {value} => {
                    self.mem_write(0x100u16 + self.cpu.sp as u16, value);
                    self.cpu.sp = self.cpu.sp.wrapping_sub(1);
                    if self.cpu.instruction.name == "BRK" {
                        self.cpu.state = CPUState::Ready;
                    } else {
                        self.cpu.state = CPUState::NeedInstruction;
                    }
                    if self.cpu.instruction.name == "JSR" {
                        let address = (self.cpu.instruction.value as u16) << 8 + self.cpu.lo as u16;
                        self.cpu.pc = address;
                    }
                }

                CPUState::Pushing16 {value} => {
                    self.mem_write(0x100u16 + self.cpu.sp as u16, (value >> 8) as u8);
                    self.cpu.sp = self.cpu.sp.wrapping_sub(1);
                    self.cpu.state = CPUState::Pushing { value: (value & 0xFF) as u8};
                }

                CPUState::Pulling => {
                    self.cpu.sp = self.cpu.sp.wrapping_add(1);
                    self.cpu.state = CPUState::PendingRead {address: self.cpu.sp as u16 + 0x100u16}; // I guess this way uses all 4 cycles
                },

                CPUState::Pulling16 => {
                    self.cpu.sp = self.cpu.sp.wrapping_add(1);
                    self.cpu.lo = self.mem_read(self.cpu.sp as u16 + 0x100u16);
                    self.cpu.state = CPUState::Pulling;
                }

                _ => {}
            }
            // Read instruction
            // Or read operand
            // Or another thing?
        }
    }

    pub fn mem_read(&mut self, address: u16) -> u8 {
        if address <= 0x1FFF {
            self.mdr = self.ram[(address & 0x7FF) as usize];
        }

        if address >= 0x8000 {
            self.mdr = self.rom[(address - 0x8000) as usize];
        }

        self.mdr
    }

    pub fn mem_write(&mut self, address: u16, value: u8) {
        self.mdr = value;
        if address <= 0x1FFF {
            self.ram[(address & 0x7FF) as usize] = value;
        }
    }
}

struct Header {
    prg_rom_pages: u8,
    chr_rom_pages: u8,
}