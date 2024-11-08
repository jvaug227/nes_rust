use bitflags::{bitflags, Flags};
use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::ops::AddAssign;
use std::rc::Rc;

use super::instructions::{create_lookup_table, Instruction};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Flags6502: u8 {
        const C = 0b00000001; // Carry
        const Z = 0b00000010; // Zero
        const I = 0b00000100; // Disable Inturrupt
        const D = 0b00001000; // Decimal Mode
        const B = 0b00010000; // Break
        const U = 0b00100000; // Unused
        const V = 0b01000000; // Overflow
        const N = 0b10000000; // Negative
    }
}

impl fmt::Display for Flags6502 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let a = self
            .iter_names()
            .collect::<Vec<_>>()
            .first()
            .map(|&(name, _flag)| name)
            .unwrap_or("");
        write!(f, "{a}")
    }
}

#[allow(non_snake_case)]
pub mod InstructionOperations {
    pub const NOP: u8 = 0x00;
    pub const LDA: u8 = 0x01;
    pub const LDX: u8 = 0x02;
    pub const LDY: u8 = 0x03;
    pub const ADC: u8 = 0x04;
    pub const AND: u8 = 0x05;
    pub const ASL: u8 = 0x06;
    pub const BCC: u8 = 0x07;
    pub const BCS: u8 = 0x08;
    pub const BEQ: u8 = 0x09;
    pub const BIT: u8 = 0x0A;
    pub const BMI: u8 = 0x0B;
    pub const BNE: u8 = 0x0C;
    pub const BPL: u8 = 0x0D;
    pub const BRK: u8 = 0x0E;
    pub const BVC: u8 = 0x0F;
    pub const BVS: u8 = 0x10;
    pub const CLC: u8 = 0x11;
    pub const CLD: u8 = 0x12;
    pub const CLI: u8 = 0x13;
    pub const CLV: u8 = 0x14;
    pub const CMP: u8 = 0x15;
    pub const CPX: u8 = 0x16;
    pub const CPY: u8 = 0x17;
    pub const DEC: u8 = 0x18;
    pub const DEX: u8 = 0x19;
    pub const DEY: u8 = 0x1A;
    pub const EOR: u8 = 0x1B;
    pub const INC: u8 = 0x1C;
    pub const INX: u8 = 0x1D;
    pub const INY: u8 = 0x1E;
    pub const JMP: u8 = 0x1F;
    pub const JSR: u8 = 0x20;
    pub const LSR: u8 = 0x21;
    pub const ORA: u8 = 0x22;
    pub const PHA: u8 = 0x23;
    pub const PHP: u8 = 0x24;
    pub const PLA: u8 = 0x25;
    pub const PLP: u8 = 0x26;
    pub const ROL: u8 = 0x27;
    pub const ROR: u8 = 0x28;
    pub const RTI: u8 = 0x29;
    pub const RTS: u8 = 0x2A;
    pub const SBC: u8 = 0x2B;
    pub const SEC: u8 = 0x2C;
    pub const SED: u8 = 0x2D;
    pub const SEI: u8 = 0x2E;
    pub const STA: u8 = 0x2F;
    pub const STX: u8 = 0x30;
    pub const STY: u8 = 0x31;
    pub const TAX: u8 = 0x32;
    pub const TAY: u8 = 0x33;
    pub const TSX: u8 = 0x34;
    pub const TXA: u8 = 0x35;
    pub const TXS: u8 = 0x36;
    pub const TYA: u8 = 0x37;

    pub const XXX: u8 = 0x00; // Illegal Opcode
}

#[allow(non_snake_case)]
pub mod InstructionAddressingModes {
    /// Implied/Implicit
    /// Opcode itself defines the information manipulated
    pub const IMP: u8 = 0x00;
    /// Accumulator
    /// Accumulator is target of instruction
    pub const ACC: u8 = 0x01;
    /// Immediate
    /// Data/Constant in next byte is target of instruction
    pub const IMM: u8 = 0x02;
    /// Zero Page
    /// 8-bit Memory address in next byte to first page of memory is target of instruction
    /// 1 cycle for indirect address (read_byte -> read_byte)
    pub const ZP: u8 = 0x03;
    /// Zero Page,X
    /// 8-bit Memory address in nexy byte is added to X register
    /// 1 cycle for indirect address (read_byte -> read_byte)
    /// 1 cycle for addition
    pub const ZP_X: u8 = 0x04;
    /// Zero Page,Y
    /// 8-bit Memory address in nexy byte is added to Y register
    /// 1 cycle for indirect address (read_byte -> read_byte)
    /// 1 cycle for addition
    pub const ZP_Y: u8 = 0x05;
    /// Relative
    /// Branch instructions have target address offset by next byte
    /// 1 cycle for byte read
    pub const REL: u8 = 0x06;
    /// Absolute
    /// 16-bit Address in next 2 bytes is the target of instruction
    /// 2 cycles for indirect 16-bit address (read_byte*2 -> read_byte)
    pub const ABS: u8 = 0x07;
    /// Absolute,X
    /// 16-bit Address in next 2 bytes + X is the target of instruction
    /// 2 cycles for indirect 16-bit address (read_byte*2 -> read_byte)
    /// 1 cycle for addition
    pub const ABS_X: u8 = 0x08;
    /// Absolute,Y
    /// 16-bit Address in next 2 bytes + Y is the target of instruction
    /// 2 cycles for indirect 16-bit address (read_byte*2 -> read_byte)
    /// 1 cycle for addition
    pub const ABS_Y: u8 = 0x09;
    /// Indirect
    /// Destination of 16-bit address in next 2 bytes is target of instruction
    /// 4 cycles for 2x indirect address
    pub const IND: u8 = 0x0A;
    pub const IND_X: u8 = 0x0B;
    pub const IND_Y: u8 = 0x0C;
    pub const ABX: u8 = ABS_X;
    pub const ABY: u8 = ABS_Y;
    pub const ZP0: u8 = ZP;
    pub const ZPX: u8 = ZP_X;
    pub const ZPY: u8 = ZP_Y;
    pub const IDX: u8 = IND_X;
    pub const IDY: u8 = IND_Y;
}

/// Godbolt produces 54 instructions + 16 bytes with 2 jumps (some boolean optimization later and its
/// ~40 instructions with no jumps, but still 16 bytes)
/// This compares to a 256 * 2 byte lookup table
/// bytecode is layed out as [a2 a1 a0 b2 b1 b0 c1 c0]
/// a, b, and c have no direct meaning
/// Needs to be profiled
pub fn bytecode_to_addrmode(x: u8) -> u8 {
    let c0 = (0b00000001 & x) > 0;
    let c1 = (0b00000010 & x) > 0;
    let b0 = (0b00000100 & x) > 0;
    let b1 = (0b00001000 & x) > 0;
    let b2 = (0b00010000 & x) > 0;
    let a0 = (0b00100000 & x) > 0;
    let a1 = (0b01000000 & x) > 0;
    let a2 = (0b10000000 & x) > 0;

    let a = x >> 5;
    let b = (x & 0b00011100) >> 2;
    let c = x & 0b00000011;

    use InstructionAddressingModes as A;
    let general_addrmodes = [
        [A::IMP, A::IND_X],
        [A::ZP, A::ZP],
        [A::IMP, A::IMM],
        [A::ABS, A::ABS],
        [A::REL, A::IND_Y],
        [A::ZP_X, A::ZP_X],
        [A::IMP, A::ABS_Y],
        [A::ABS_X, A::ABS_X],
    ];

    let mut addrmode =
        general_addrmodes[b as usize][c0 as usize];

    let l0 = (!c0 & a2 & (b == 0)) as u8 * 2; // IMP -> IMM
    let l1 = ((c == 2) & (b == 2) & !a2) as u8; // IMP -> ACC
    let l2 = (c1 & ((b == 5) | (b == 7)) & ((a == 4) | (a == 5))) as u8; // zp/abs X -> Y
    let l3 = (x == 0x20) as u8 * 7; // IMP -> ABS
    let l4 = (x  == 0x6C) as u8 * 3; // ABS -> IND

    addrmode += l0 + l1 + l2 + l3 + l4;

    addrmode
}

pub struct Cpu {
    bus: Rc<RefCell<Bus>>,

    // Registers
    pub a: u8, // Accumulator
    pub x: u8,
    pub y: u8,
    pub stkpt: u8, // Stack Pointer
    pub pc: u16,   // Program Counter
    status: Flags6502,

    pub pipeline_status: PipelineStatus,
    pub page_boundary_crossed: bool,

    pub fetched: u8,

    pub addr_data: u16,
    addr_abs: u16,
    addr_rel: u16,

    pub opcode: u8,
    cycles: u32,

    lookup: [Instruction; 256],
}

#[derive(Clone, Copy, Debug)]
pub enum PipelineStatus {
    Addr0,
    Addr1,
    Addr2,
    Addr3,
    Addr4,
    Addr5,
    Exec0,
    Exec1,
    Exec2,
    Exec3,
    Exec4,
    Exec5,
    Store,
}

impl PipelineStatus {
    pub fn advance(&mut self) {
        *self = match self {
            Self::Addr0 => Self::Addr1,
            Self::Addr1 => Self::Addr2,
            Self::Addr2 => Self::Addr3,
            Self::Addr3 => Self::Addr4,
            Self::Addr4 => Self::Addr5,
            Self::Addr5 => Self::Exec0,
            Self::Exec0 => Self::Exec1,
            Self::Exec1 => Self::Exec2,
            Self::Exec2 => Self::Exec3,
            Self::Exec3 => Self::Exec4,
            Self::Exec4 => Self::Exec5,
            Self::Exec5 => Self::Store,
            Self::Store => Self::Addr0,
        }
    }
}

pub fn lo_byte(word: u16) -> u8 {
    word as u8 // gets truncated
}

pub fn hi_byte(word: u16) -> u8 {
    (word >> 8) as u8 // gets truncated
}

fn set_lo_byte(word: &mut u16, byte: u8) {
    *word = (*word & 0xFF00) | u16::from(byte);
}

fn set_hi_byte(word: &mut u16, byte: u8) {
    *word = (*word & 0x00FF) | (u16::from(byte) << 8);
}

#[allow(non_snake_case)]
impl Cpu {
    pub fn new(bus: Rc<RefCell<Bus>>) -> Self {
        Self {
            bus,
            a: 0,
            x: 0,
            y: 0,
            stkpt: 0,
            pc: 0,
            status: Flags6502::empty(),
            pipeline_status: PipelineStatus::Addr0,
            page_boundary_crossed: false,
            fetched: 0,
            addr_data: 0,
            addr_abs: 0,
            addr_rel: 0,
            opcode: 0,
            cycles: 0,
            lookup: create_lookup_table(),
        }
    }

    pub fn cycles_mut(&mut self) -> &mut u32 {
        &mut self.cycles
    }

    // Bus
    pub fn write(&mut self, addr: u16, byte: u8) {
        self.bus_mut().write(addr, byte);
    }
    pub fn read_byte(&self, addr: u16) -> u8 {
        self.bus().read(addr, false)
    }
    pub fn read_word(&self, addr: u16) -> u16 {
        let lo: u16 = self.read_byte(addr) as u16;
        let hi: u16 = self.read_byte(addr + 1) as u16;
        lo | (hi << 8)
    }
    pub fn bus(&self) -> Ref<Bus> {
        self.bus.borrow()
    }
    pub fn bus_mut(&self) -> RefMut<Bus> {
        self.bus.borrow_mut()
    }

    // Perform one instruction worth of emulation
    pub fn clock(&mut self) -> bool {
        // println!("Clock: {:?}", self.pipeline_status);
        let instruction = self.lookup[self.opcode as usize];
        let instruction_finished = self.execute(instruction.op(), instruction.addrmode());
        if instruction_finished {
            self.opcode = self.read_byte(self.pc);
            self.pc += 1;
        }
        instruction_finished
    }

    /// Returns true if the instruction should immediately begin executing
    /// an instruction  
    /// Returns false if the cpu should wait for a clock cycle
    fn execute_addrmode(&mut self, addrmode: u8) -> bool {
        use InstructionAddressingModes as Addr;

        let opcode = self.lookup[self.opcode as usize].op();
        // Page boundary incurrs a +1 cycle cost
        if self.page_boundary_crossed {
            // Probably close-enough to a realistic re-creation
            let hi_byte = hi_byte(self.addr_data).wrapping_add(1);
            set_hi_byte(&mut self.addr_data, hi_byte);
            // Could also probably do something like, might infact get compiler-optimized to this
            // self.addr_data = self.addr_data.wrapping_add(0x10);
            self.page_boundary_crossed = false;
            return false;
        }
        match self.pipeline_status {
            PipelineStatus::Addr0 => {
                self.addr_data = self.read_byte(self.pc) as u16;
                self.pc += 1;
                false
            },
            PipelineStatus::Addr1 => match addrmode {
                Addr::ACC => { self.fetched = self.a; true },
                Addr::IMP |Addr::IMM | Addr::REL => { self.fetched = self.addr_data as u8; true }, // Rel offset is stored in both fetched and addr_data
                Addr::ZP0 => { self.fetched = self.read_byte(self.addr_data); false },
                Addr::ZPX => { self.addr_data = u8::wrapping_add(lo_byte(self.addr_data), self.x) as u16; false }, // These have no page-boundary since they will only access page 0
                Addr::ZPY => { self.addr_data = u8::wrapping_add(lo_byte(self.addr_data), self.y) as u16; false },
                Addr::ABS | Addr::ABX | Addr::ABY | Addr::IND => {
                    let offset = match addrmode {
                        Addr::ABX => { self.x },
                        Addr::ABY => { self.y },
                        _ => { 0 },
                    };
                    let old_lo_byte = lo_byte(self.addr_data);
                    let new_lo_byte = old_lo_byte.wrapping_add(offset);
                    set_lo_byte(&mut self.addr_data, new_lo_byte);

                    // page-boundary crossed if we wrapped around and
                    // addition creates a lower value than we started with
                    self.page_boundary_crossed = new_lo_byte < old_lo_byte;

                    let b = self.read_byte(self.pc);
                    self.pc += 1;
                    set_hi_byte(&mut self.addr_data, b);
                    false
                },
                Addr::IDX => {
                    self.fetched = lo_byte(self.addr_data).wrapping_add(self.x);
                    let b = self.read_byte(self.fetched as u16);
                    set_lo_byte(&mut self.addr_data, b);
                    false
                },
                Addr::IDY => {
                    self.fetched = self.read_byte(self.addr_data); // put lo byte in fetched
                    self.addr_data = self.addr_data.wrapping_add(1); // store next zp address in addr_data
                    false
                }
                _ => { true }
            },
            PipelineStatus::Addr2 => match addrmode {
                Addr::ZP0 => { true },
                Addr::ZPX | Addr::ZPY | Addr::ABS | Addr::ABX | Addr::ABY => {
                    if opcode == InstructionOperations::JMP {
                        self.pc = self.addr_data;
                        return true;
                    }
                    self.fetched = self.read_byte(self.addr_data);
                    false
                },
                Addr::IND => {
                    let b = self.read_byte(self.addr_data);
                    set_lo_byte(&mut self.pc, b);
                    // emulate indirect page boundary bug
                    let new_lo_byte = lo_byte(self.addr_data).wrapping_add(1);
                    set_lo_byte(&mut self.addr_data, new_lo_byte);
                    false
                },
                Addr::IDX => {
                    self.fetched = self.fetched.wrapping_add(1);
                    let b = self.read_byte(self.fetched as u16);
                    set_hi_byte(&mut self.addr_data, b);
                    // self.fetched = self.read_byte(lo_byte(self.addr_data) as u16);
                    // self.addr_data = self.addr_data.wrapping_add(1);
                    false
                },
                Addr::IDY => {
                    let zpAddr = lo_byte(self.addr_data) as u16; // grab zp address before we replace it
                    let old_lo_byte = self.fetched;
                    let new_lo_byte = self.fetched.wrapping_add(self.y);
                    set_lo_byte(&mut self.addr_data, new_lo_byte);
                    self.page_boundary_crossed = new_lo_byte < old_lo_byte; // page-boundary crossed if we wrapped around and 
                                                                            // addition creates a lower value than we started with
                    let b = self.read_byte(zpAddr); // read zp address for hi byte
                    set_hi_byte(&mut self.addr_data, b);
                    false
                }
                _ => { true }
            },
            PipelineStatus::Addr3 => match addrmode {
                Addr::ZPX | Addr::ZPY | Addr::ABS | Addr::ABY => { true },
                Addr::ABX => {
                    use InstructionOperations as InsOp;
                    !matches!(opcode, InsOp::DEC | InsOp::INC | InsOp::LSR | InsOp::ROL | InsOp::ROR | InsOp::ASL) 
                } // if this is DEC, this cycle is dedicated to fixing page boundary
                Addr::IND => {
                    let b = self.read_byte(self.addr_data);
                    set_hi_byte(&mut self.pc, b);
                    false
                },
                Addr::IDX => {
                    self.fetched = self.read_byte(self.addr_data);
                    // set_hi_byte(&mut self.addr_data, b);
                    // set_lo_byte(&mut self.addr_data, self.fetched);
                    false
                },
                Addr::IDY => {
                    self.fetched = self.read_byte(self.addr_data);
                    false
                }
                _ => { true }
            },
            PipelineStatus::Addr4 => match addrmode {
                Addr::ABX => { true },
                Addr::IND => {
                    true
                },
                Addr::IDX => {
                    // let b = self.read_byte(self.addr_data);
                    // self.addr_data = b as u16;
                    true
                },
                Addr::IDY => {
                    true
                },
                _ => { true }
            },
            _ => { // assume IND,X addrmode
                    self.fetched = self.addr_data as u8; true
            }
        }
    }

    /// TODO: Find out why ABS_X or ABS_Y or IND_Y in ASL, DEC, INC, LSR, ROL,
    /// ROR, STA requires an extra cycle compared to ABS, according to [Obelisk](https://www.nesdev.org/obelisk-6502-guide/reference.html)
    /// E.g. zp,x, abs, abs,x and abs,y usually are 4 cycles in most instructions with (ind, x) being 6
    /// and (ind),y being 5. In STA, abs,x and abs,y are 5 while (ind),y is 6.
    /// Edit: looks to be caused by write-instructions always fixing the PCH, even if it doesn't
    /// need it
    fn execute_instruction(&mut self, instruction: u8) -> bool {
        use InstructionOperations as InsOp;
        use PipelineStatus as PS;
        let opcode_addr_mode = self.lookup[self.opcode as usize].addrmode();
        match (instruction, self.pipeline_status) {
            (InsOp::NOP, _) => {
                // NESDEV unofficial opcodes
                // match self.opcode {
                //     0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                //         cycles.add_assign(1);
                //     }
                //     _ => {}
                // }
                true
            }
            // Add with carry
            (InsOp::ADC, PS::Exec0) => {
                let temp = self.a as u16 + self.fetched as u16 + self.get_flag(Flags6502::C) as u16;

                self.check_nzc_flags(temp);
                self.set_flag(
                    Flags6502::V,
                    ((!(self.a as u16 ^ self.fetched as u16) & (self.a as u16 ^ temp)) & 0x0080)
                        > 0,
                );
                self.a = (temp & 0x00FF) as u8;
                true
            }
            // Logical And (&)
            (InsOp::AND, PS::Exec0) => {
                self.a &= self.fetched;
                self.check_nz_flags(self.a);
                true
            }
            // Arithmatic Shift Left ( A|M << 1 OR A|M * 2)
            (InsOp::ASL, PS::Exec0) => {
                let temp = self.fetched << 1;
                self.check_nz_flags(temp);
                self.set_flag(Flags6502::C, (self.fetched & 0x80) > 0);

                if opcode_addr_mode == InstructionAddressingModes::ACC {
                    self.a = temp;
                    true
                } else {
                    self.fetched = temp;
                    false
                }
            },
            (InsOp::ASL, PS::Exec1) => {
                self.write(self.addr_data, self.fetched);
                false
            },
            (InsOp::ASL, PS::Exec2) => {
                true
            }

            // Branch if Carry Clear
            (InsOp::BCC, PS::Exec0) => {
                self.branch(!self.get_flag(Flags6502::C))
            }
            // Branch is Carry Set
            (InsOp::BCS, PS::Exec0) => {
                self.branch(self.get_flag(Flags6502::C))
            }
            // Branch if Equal (Branch if Zero Set)
            (InsOp::BEQ, PS::Exec0) => {
                self.branch(self.get_flag(Flags6502::Z))
            }
            // Branch if Minus (Branch if Negative Flag Set)
            (InsOp::BMI, PS::Exec0) => {
                self.branch(self.get_flag(Flags6502::N))
            }
            // Branch if Not Equal (Branch if Zero Clear)
            (InsOp::BNE, PS::Exec0) => {
                self.branch(!self.get_flag(Flags6502::Z))
            }
            // Branch is Positive (Branch if Negative Clear)
            (InsOp::BPL, PS::Exec0) => {
                self.branch(!self.get_flag(Flags6502::N))
            }
            // Branch if Overflow Clear
            (InsOp::BVC, PS::Exec0) => {
                self.branch(!self.get_flag(Flags6502::V))
            }
            // Branch if Overflow Set
            (InsOp::BVS, PS::Exec0) => {
                self.branch(self.get_flag(Flags6502::V))
            }
            (InsOp::BCC | InsOp::BCS | InsOp::BEQ | InsOp::BMI | InsOp::BNE | InsOp::BPL | InsOp::BVC | InsOp::BVS, PS::Exec1) => {
                // const sign_extended: u16 = b as i8 as i16 as u16;
                // const high_offset: u8 = (sign_extended >> 8) as u8;
                let sign_extended = hi_byte(self.fetched as i8 as i16 as u16);
                // If the high byte is incorrect due to passing a page boundary or subtraction
                // Insert fix-PC_H microcode
                if self.page_boundary_crossed || sign_extended > 0 {
                    let old_hi_byte = hi_byte(self.pc);
                    let new_hi_byte = old_hi_byte.wrapping_add(sign_extended).wrapping_add(self.page_boundary_crossed as u8);
                    set_hi_byte(&mut self.pc, new_hi_byte);
                    self.page_boundary_crossed = false;
                    false
                } else {
                    // we are done, fetch next instruction
                    true
                }
            }
            (InsOp::BCC | InsOp::BCS | InsOp::BEQ | InsOp::BMI | InsOp::BNE | InsOp::BPL | InsOp::BVC | InsOp::BVS, PS::Exec2) => {
                true
            }
            // Bit Test
            (InsOp::BIT, PS::Exec0) => {
                let temp = self.a & self.fetched;
                self.check_z_flag(temp);
                self.set_flag(Flags6502::N, (self.fetched & (1 << 7)) > 0);
                self.set_flag(Flags6502::V, (self.fetched & (1 << 6)) > 0);
                true
            }
            // BRK - Force Inturrupt
            (InsOp::BRK, PS::Exec0) => {
                let pc_hi_bits = hi_byte(self.pc);
                self.write(0x0100 + (self.stkpt as u16), pc_hi_bits);
                self.stkpt = self.stkpt.wrapping_sub(1);

                false
            },
            (InsOp::BRK, PS::Exec1) => {
                let pc_low_bits = lo_byte(self.pc);
                self.write(0x0100 + self.stkpt as u16, pc_low_bits);
                self.stkpt = self.stkpt.wrapping_sub(1);
                false
            },
            (InsOp::BRK, PS::Exec2) => {
                let p_s = (self.status | Flags6502::B | Flags6502::U).bits();
                self.write(0x0100 + self.stkpt as u16, p_s );
                self.stkpt = self.stkpt.wrapping_sub(1);
                self.set_flag(Flags6502::I, true);
                false
            },
            (InsOp::BRK, PS::Exec3) => {
                // write pc low
                let b = self.read_byte(0xFFFE);
                set_lo_byte(&mut self.pc, b);
                false
            },
            (InsOp::BRK, PS::Exec4) => {
                // write pc high
                let b = self.read_byte(0xFFFF);
                set_hi_byte(&mut self.pc, b);
                false
            },
            (InsOp::BRK, PS::Exec5) => {
                true
            },
            // Clear Carry Flag
            (InsOp::CLC, PS::Exec0) => {
                self.set_flag(Flags6502::C, false);
                true
            }
            // Clear Decimal Mode
            (InsOp::CLD, PS::Exec0) => {
                self.set_flag(Flags6502::D, false);
                true
            }
            // Clear Inturrupt Disable
            (InsOp::CLI, PS::Exec0) => {
                self.set_flag(Flags6502::I, false);
                true
            }
            // Clear Overflow Flag
            (InsOp::CLV, PS::Exec0) => {
                self.set_flag(Flags6502::V, false);
                true
            }
            // Compare
            (InsOp::CMP, PS::Exec0) => {
                self.compare(self.a, self.fetched);
                true
            }
            // Compare X Register
            (InsOp::CPX, PS::Exec0) => {
                self.compare(self.x, self.fetched);
                true
            }
            // Compare Y Register
            (InsOp::CPY, PS::Exec0) => {
                self.compare(self.y, self.fetched);
                true
            }
            // Decrement value at address
            (InsOp::DEC, PS::Exec0) => {
                let temp = self.decrement(self.fetched);
                self.check_nz_flags(temp);
                self.fetched = temp;
                false
            },
            (InsOp::DEC, PS::Exec1) => {
                self.write(self.addr_data, self.fetched);
                false
            },
            (InsOp::DEC, PS::Exec2) => {
                true
            }
            // Decrement X register
            (InsOp::DEX, PS::Exec0) => {
                self.x = self.decrement(self.x);
                self.check_nz_flags(self.x);
                true
            }

            // Decrement Y register
            (InsOp::DEY, PS::Exec0) => {
                self.y = self.decrement(self.y);
                self.check_nz_flags(self.y);
                true
            }
            // Exclusive OR
            (InsOp::EOR, PS::Exec0) => {
                self.a = self.a ^ self.fetched;
                self.check_nz_flags(self.a);
                true
            }
            // Increment memory at address
            (InsOp::INC, PS::Exec0) => {
                let temp = self.increment(self.fetched);
                self.check_nz_flags(temp);
                if opcode_addr_mode == InstructionAddressingModes::ACC{ self.a = temp; true }
                else { self.fetched = temp; false }
            },
            (InsOp::INC, PS::Exec1) => {
                self.write(self.addr_data, self.fetched);
                false
            }
            (InsOp::INC, PS::Exec2) => {
                true
            }
            // Increment X register
            (InsOp::INX, PS::Exec0) => {
                self.x = self.increment(self.x);
                self.check_nz_flags(self.x);
                true
            }
            // Increment Y register
            (InsOp::INY, PS::Exec0) => {
                self.y = self.increment(self.y);
                self.check_nz_flags(self.y);
                true
            }
            // Jump to address
            (InsOp::JMP, PS::Exec0) => {
                //self.pc = self.addr_data; // already taken care of in addrmode execution
                true
            }
            // Jump to subroutine
            // ~~TODO: Check cycle correctness~~
            // INFO: JSR is apparently done very weirdly:
            // 1. Read lo byte of data
            // 2. Read stkptr for some reason
            // 3. Push high byte of pc
            // 4. Push low byte of pc
            // 5. Read hi byte of data
            // 6. Read next instruction
            // I read the new pc, push the old pc,
            // then set pc to the new pc
            (InsOp::JSR, PS::Exec0) => {
                self.pc -= 1;

                self.write(
                    0x0100 + self.stkpt as u16,
                    hi_byte(self.pc),
                );
                self.stkpt = self.stkpt.wrapping_sub(1);
                false
            },
            (InsOp::JSR, PS::Exec1) => {
                self.write(0x0100 + self.stkpt as u16, lo_byte(self.pc));
                self.stkpt = self.stkpt.wrapping_sub(1);
                self.pc = self.addr_data;
                false
            }
            (InsOp::JSR, PS::Exec2) => {
                true
            }

            // Load Accumulator
            (InsOp::LDA, PS::Exec0) => {
                self.a = self.fetched;
                self.check_nz_flags(self.a);
                true
            }
            // Load X register
            (InsOp::LDX, PS::Exec0) => {
                self.x = self.fetched;
                self.check_nz_flags(self.x);
                true
            }
            // Load Y register
            (InsOp::LDY, PS::Exec0) => {
                self.y = self.fetched;
                self.check_nz_flags(self.y);
                true
            }
            // Logical Shift Right
            (InsOp::LSR, PS::Exec0) => {
                self.set_flag(Flags6502::C, self.fetched & 0x0001 > 0);
                let temp = self.fetched >> 1;
                self.check_nz_flags(temp);

                if opcode_addr_mode == InstructionAddressingModes::ACC {
                    self.a = temp;
                    true
                } else {
                    self.fetched = temp;
                    false
                }
            },
            (InsOp::LSR, PS::Exec1) => {
                self.write(self.addr_data, self.fetched);
                true
            }

            // Logical Inclusive OR
            (InsOp::ORA, PS::Exec0) => {
                self.a = self.a | self.fetched;
                self.check_nz_flags(self.a);
                true
            }
            // Push Accumulator (to stack)
            (InsOp::PHA, PS::Exec0) => {
                self.write(0x0100 + self.stkpt as u16, self.a);
                self.stkpt = self.stkpt.wrapping_sub(1);
                true
            }
            // Push Processor Stack
            (InsOp::PHP, PS::Exec0) => {
                // B and U are pushed as set
                self.write(0x0100 + self.stkpt as u16, (self.get_status() | Flags6502::B | Flags6502::U).bits()); 
                self.stkpt = self.stkpt.wrapping_sub(1);
                true
            }
            // Pull Accumulator
            (InsOp::PLA, PS::Exec0) => {
                self.stkpt = self.stkpt.wrapping_add(1);
                self.a = self.read_byte(0x0100 + self.stkpt as u16);
                self.check_nz_flags(self.a);
                true
            }
            // Pull Processor Stack
            (InsOp::PLP, PS::Exec0) => {
                self.stkpt = self.stkpt.wrapping_add(1);
                self.status = Flags6502::from_bits_retain(
                        self.read_byte(0x0100 + self.stkpt as u16) & 0b11001111
                        | self.status.bits() & 0b00110000
                );
                true
            }
            // Rotate Left
            (InsOp::ROL, PS::Exec0) => {
                let temp = ((self.fetched) << 1) | self.get_flag(Flags6502::C) as u8;
                self.set_flag(Flags6502::C, (self.fetched & 0x80) > 0);
                self.check_nz_flags(temp);
                if opcode_addr_mode == InstructionAddressingModes::ACC {
                    self.a = temp;
                    true
                } else {
                    self.fetched = temp;
                    false
                }
            }
            (InsOp::ROL, PS::Exec1) => {
                self.write(self.addr_data, self.fetched);
                false
            }
            (InsOp::ROL, PS::Exec2) => {
                true
            }
            // Rotate Right
            (InsOp::ROR, PS::Exec0) => {
                let temp = ((self.get_flag(Flags6502::C) as u8) << 7) | self.fetched >> 1;
                self.set_flag(Flags6502::C, self.fetched & 0x01 > 0);
                self.check_nz_flags(temp);

                if opcode_addr_mode == InstructionAddressingModes::ACC {
                    self.a = temp;
                    true
                } else {
                    self.fetched = temp;
                    false
                }
            }
            (InsOp::ROR, PS::Exec1) => {
                self.write(self.addr_data, self.fetched);
                false
            }
            (InsOp::ROR, PS::Exec2) => {
                true
            }
            // Return from Inturrupt
            (InsOp::RTI, PS::Exec0) => {
                self.stkpt = self.stkpt.wrapping_add(1);
                let _ = self.read_byte(0x0100 + self.stkpt as u16); // dummy read
                false
            },
            (InsOp::RTI,PS::Exec1) => {
                let b = self.read_byte(0x0100 + self.stkpt as u16);
                let b = (b & 0b11001111) | (self.status.bits() & 0b00110000);
                self.stkpt = self.stkpt.wrapping_add(1);
                self.status = Flags6502::from_bits_retain(b);
                // self.status &= !Flags6502::B;
                // self.status &= !Flags6502::U;
                false
            }
            (InsOp::RTI, PS::Exec2) => {
                let b = self.read_byte(0x0100 + self.stkpt as u16);
                self.stkpt = self.stkpt.wrapping_add(1);
                set_lo_byte(&mut self.pc, b);
                false
            },
            (InsOp::RTI, PS::Exec3) => {
                let b = self.read_byte(0x0100 + self.stkpt as u16);
                set_hi_byte(&mut self.pc, b);
                false
            }
            (InsOp::RTI, PS::Exec4) => {
                true
            }
            // Return from subroutine
            (InsOp::RTS, PS::Exec0) => {
                self.stkpt = self.stkpt.wrapping_add(1);
                let _ = self.read_byte(0x0100 + self.stkpt as u16); // dummy read
                false
            },
            (InsOp::RTS, PS::Exec1) => {
                let b = self.read_byte(0x0100 + self.stkpt as u16);
                self.stkpt = self.stkpt.wrapping_add(1);
                set_lo_byte(&mut self.pc, b);
                false
            },
            (InsOp::RTS, PS::Exec2) => {
                let b = self.read_byte(0x0100 + self.stkpt as u16);
                set_hi_byte(&mut self.pc, b);
                false
            }
            (InsOp::RTS, PS::Exec3) => {
                true
            }

            // Subtract with carry
            // TODO: Confirm subtraction works
            (InsOp::SBC, PS::Exec0) => {
                let value = (!self.fetched) as u16;
                let temp = self.a as u16 + value + self.get_flag(Flags6502::C) as u16;
                self.set_flag(Flags6502::C, temp & 0xFF00 > 0);
                self.check_nz_flags(temp as u8);
                self.set_flag(
                    Flags6502::V,
                    ((temp ^ self.a as u16) & (temp ^ value) & 0x0080) > 0,
                );
                self.a = lo_byte(temp);
                true
            }
            // Set carry flag
            (InsOp::SEC, PS::Exec0) => {
                self.set_flag(Flags6502::C, true);
                true
            }
            // Set decimal flag
            (InsOp::SED, PS::Exec0) => {
                self.set_flag(Flags6502::D, true);
                true
            }
            // Set inturrupt disable
            (InsOp::SEI, PS::Exec0) => {
                self.set_flag(Flags6502::I, true);
                true
            }
            // Store Accumulator
            (InsOp::STA, PS::Exec0) => {
                self.write(self.addr_data, self.a);
                false
            }
            (InsOp::STA, PS::Exec1) => {
                true
            }
            // Store X register
            (InsOp::STX, PS::Exec0) => {
                self.write(self.addr_data, self.x);
                false
            }
            (InsOp::STX, PS::Exec1) => {
                true
            }
            // Store Y register
            (InsOp::STY, PS::Exec0) => {
                self.write(self.addr_data, self.y);
                false
            }
            (InsOp::STY, PS::Exec1) => {
                true
            }
            // Transfer Accumulator to X
            (InsOp::TAX, PS::Exec0) => {
                self.x = self.a;
                self.check_nz_flags(self.x);
                true
            }
            // Transfer Accumulator to Y
            (InsOp::TAY, PS::Exec0) => {
                self.y = self.a;
                self.check_nz_flags(self.y);
                true
            }
            // Transfer Stack Pointer to X
            (InsOp::TSX, PS::Exec0) => {
                self.x = self.stkpt;
                self.check_nz_flags(self.x);
                true
            }
            // Transfer Stack Pointer to A
            (InsOp::TXA, PS::Exec0) => {
                self.a = self.x;
                self.check_nz_flags(self.a);
                true
            }
            // Transfer X to Stack Pointer
            (InsOp::TXS, PS::Exec0) => {
                self.stkpt = self.x;
                true
            }
            // Transfer Y to Accumulator
            (InsOp::TYA, PS::Exec0) => {
                self.a = self.y;
                self.check_nz_flags(self.a);
                true
            }
            _ => { false } // Illegal Instruction
        }
    }

    fn execute(&mut self, instruction: u8, addrmode: u8) -> bool {

        if !matches!(self.pipeline_status, PipelineStatus::Exec0 | PipelineStatus::Exec1 | PipelineStatus::Exec2 | PipelineStatus::Exec3 | PipelineStatus::Exec4 | PipelineStatus::Exec5 | PipelineStatus::Store) {
            let finished_addressing = self.execute_addrmode(addrmode);
            if !finished_addressing { if !self.page_boundary_crossed { self.pipeline_status.advance(); } return false; }
            else { self.pipeline_status = PipelineStatus::Exec0; }
        }
        let finished_executing = self.execute_instruction(instruction);
        if !finished_executing { self.pipeline_status.advance(); }
        else { self.pipeline_status = PipelineStatus::Addr0; }
        finished_executing
    }

    // Forces the 6502 into a known state. This is hard-wired inside the CPU. The
    // registers are set to 0x00, the status register is cleared except for unused
    // bit which remains at 1. An absolute address is read from location 0xFFFC
    // which contains a second address that the program counter is set to. This
    // allows the programmer to jump to a known and programmable location in the
    // memory to start executing from. Typically the programmer would set the value
    // at location 0xFFFC at compile time.
    pub fn reset(&mut self) {
        self.addr_data = 0xFFFC;

        self.pc = self.read_word(self.addr_data);

        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.stkpt = 0xFD;
        self.status = Flags6502::U;

        self.fetched = 0;
    }

    // Interrupt requests are a complex operation and only happen if the
    // "disable interrupt" flag is 0. IRQs can happen at any time, but
    // you dont want them to be destructive to the operation of the running
    // program. Therefore the current instruction is allowed to finish
    // (which I facilitate by doing the whole thing when cycles == 0) and
    // then the current program counter is stored on the stack. Then the
    // current status register is stored on the stack. When the routine
    // that services the interrupt has finished, the status register
    // and program counter can be restored to how they where before it
    // occurred. This is impemented by the "RTI" instruction. Once the IRQ
    // has happened, in a similar way to a reset, a programmable address
    // is read form hard coded location 0xFFFE, which is subsequently
    // set to the program counter.
    pub fn irq(&mut self, cycles: &mut u32) {
        // If interrupts are allowed
        if !self.get_flag(Flags6502::I) {
            // Push the program counter to the stack. It's 16-bits dont
            // forget so that takes two pushes
            self.write(
                0x0100 + self.stkpt as u16,
                ((self.pc >> 8) & 0x00FF) as u8,
            );
            self.stkpt -= 1;
            self.write(0x0100 + self.stkpt as u16, (self.pc & 0x00FF) as u8);
            self.stkpt -= 1;

            // Then Push the status register to the stack
            self.set_flag(Flags6502::B, false);
            self.set_flag(Flags6502::U, true);
            self.set_flag(Flags6502::I, true);
            self.write(0x0100 + self.stkpt as u16, self.get_status().bits());
            self.stkpt -= 1;

            // Read new program counter location from fixed address
            self.addr_abs = 0xFFFE;
            self.pc = self.read_word(self.addr_abs);

            // IRQs take time
            cycles.add_assign(2);
        }
    }
    // A Non-Maskable Interrupt cannot be ignored. It behaves in exactly the
    // same way as a regular IRQ, but reads the new program counter address
    // from location 0xFFFA.
    pub fn nmi(&mut self, cycles: &mut u32) {
        if !self.get_flag(Flags6502::I) {
            self.write(
                0x0100 + self.stkpt as u16,
                hi_byte(self.pc),
            );
            self.stkpt -= 1;
            self.write(0x0100 + self.stkpt as u16, lo_byte(self.pc));
            self.stkpt -= 1;

            self.set_flag(Flags6502::B, false);
            self.set_flag(Flags6502::U, true);
            self.set_flag(Flags6502::I, true);
            self.write(0x0100 + self.stkpt as u16, self.get_status().bits());
            self.stkpt -= 1;

            self.addr_abs = 0xFFFA;
            self.pc = self.read_word(self.addr_abs);

            cycles.add_assign(3);
        }
    }

    // internal helpers
    pub fn complete(&mut self) -> bool {
        self.cycles == 0
    }
    pub fn check_n_flag(&mut self, value: u8) {
        self.set_flag(Flags6502::N, (value & 0x80) > 0);
    }

    pub fn check_z_flag(&mut self, value: u8) {
        self.set_flag(Flags6502::Z, value == 0);
    }

    pub fn check_c_flag(&mut self, value: u16) {
        self.set_flag(Flags6502::C, value > 255);
    }

    pub fn check_nz_flags(&mut self, value: u8) {
        self.check_n_flag(value);
        self.check_z_flag(value);
    }

    pub fn check_nzc_flags(&mut self, value: u16) {
        self.check_n_flag(value as u8);
        self.check_z_flag(value as u8);
        self.check_c_flag(value);
    }

    // Find differences and cut off
    // low-bits; if first high-bit is set,
    // page boundary crossed
    pub fn page_boundary_crossed(addr_old: u16, addr_new: u16) -> u16 {
        ((addr_old ^ addr_new) >> 8) & 0x01 // & 0x01 potentially uneeded
    }

    // Flags
    pub fn get_flag(&self, flag: Flags6502) -> bool {
        self.status.contains(flag)
    }
    pub fn get_status(&self) -> Flags6502 {
        self.status
    }
    pub fn set_flag(&mut self, flag: Flags6502, v: bool) {
        self.status.set(flag, v);
    }
    pub fn set_flags(&mut self, flags: Flags6502) {
        self.status = flags;
    }

    /// Generalized branch function
    /// If PC_l + r yeilds a new PC_l less than the original, a page boundary was crossed
    /// and the high byte needs to be changed, this is is true whether the offset was negative or
    /// positive
    /// TODO: Confirm branch mechanics
    fn branch(&mut self, flag: bool) -> bool {
        if flag {
            let old_lo_byte = lo_byte(self.pc);
            let new_lo_byte = old_lo_byte.wrapping_add(self.fetched);
            set_lo_byte(&mut self.pc, new_lo_byte);
            self.page_boundary_crossed = new_lo_byte < old_lo_byte; // X + FF = X-1
        }
        !flag // we passed, so actually don't finish instruction
    }
    fn decrement(&mut self, value: u8) -> u8 {
        let temp = value.wrapping_sub(1);
        self.check_nz_flags(temp);
        temp
    }
    fn increment(&mut self, value: u8) -> u8 {
        let temp = value.wrapping_add(1);
        self.check_nz_flags(temp);
        temp
    }
    fn compare(&mut self, a: u8, b: u8) {
        let temp = a.wrapping_sub(b);
        self.check_nz_flags(temp);
        self.set_flag(Flags6502::C, a >= b);
    }
}

pub struct Bus {
    pub ram: Vec<u8>,
}

impl Bus {
    pub fn new() -> Self {
        const RAM_SIZE: usize = 256 * 2048;
        let mut ram: Vec<u8> = Vec::with_capacity(RAM_SIZE);
        ram.resize(RAM_SIZE, 0);
        Self {
            ram, // 65536 262160
        }
    }
    pub fn write(&mut self, addr: u16, byte: u8) {
        let addr = addr as usize;
        // if addr >= 0x0000 && addr <= 0xFFFF {
        self.ram[addr] = byte;
        // }
    }
    pub fn read(&self, addr: u16, _readonly: bool) -> u8 {
        let addr = addr as usize;
        // if addr >= 0x0000 && addr <= 0xFFFF {
        return self.ram[addr];
        // }
        // 0
    }
}
