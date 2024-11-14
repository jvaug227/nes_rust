use bitflags::bitflags;
use std::fmt;

use super::instructions::lookup;

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

    // Stable opcodes
    pub const SLO: u8 = 0x38;
    pub const RLA: u8 = 0x39;
    pub const SRE: u8 = 0x3A;
    pub const RRA: u8 = 0x3B;
    pub const SAX: u8 = 0x3C;
    pub const LAX: u8 = 0x3D;
    pub const DCP: u8 = 0x3E;
    pub const ISC: u8 = 0x3F;
    pub const ANC: u8 = 0x40;
    pub const ALR: u8 = 0x41;
    pub const ARR: u8 = 0x42;
    pub const SBX: u8 = 0x43;
    // SBC duplicate

    // slightly unstable opcodes
    pub const SHA: u8 = 0x44;
    pub const SHY: u8 = 0x45;
    pub const SHX: u8 = 0x46;
    pub const TAS: u8 = 0x47;
    pub const LAS: u8 = 0x48;

    // unstable opcodes
    // Lax imm
    pub const ANE: u8 = 0x49;
    pub const ANX: u8 = 0x50;

    pub const JAM: u8 = 0x51;
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
// pub fn bytecode_to_addrmode(x: u8) -> u8 {
//     let c0 = (0b00000001 & x) > 0;
//     let c1 = (0b00000010 & x) > 0;
//     let b0 = (0b00000100 & x) > 0;
//     let b1 = (0b00001000 & x) > 0;
//     let b2 = (0b00010000 & x) > 0;
//     let a0 = (0b00100000 & x) > 0;
//     let a1 = (0b01000000 & x) > 0;
//     let a2 = (0b10000000 & x) > 0;
//
//     let a = x >> 5;
//     let b = (x & 0b00011100) >> 2;
//     let c = x & 0b00000011;
//
//     use InstructionAddressingModes as A;
//     let general_addrmodes = [
//         [A::IMP, A::IND_X],
//         [A::ZP, A::ZP],
//         [A::IMP, A::IMM],
//         [A::ABS, A::ABS],
//         [A::REL, A::IND_Y],
//         [A::ZP_X, A::ZP_X],
//         [A::IMP, A::ABS_Y],
//         [A::ABS_X, A::ABS_X],
//     ];
//
//     let mut addrmode =
//         general_addrmodes[b as usize][c0 as usize];
//
//     let l0 = (!c0 & a2 & (b == 0)) as u8 * 2; // IMP -> IMM
//     let l1 = ((c == 2) & (b == 2) & !a2) as u8; // IMP -> ACC
//     let l2 = (c1 & ((b == 5) | (b == 7)) & ((a == 4) | (a == 5))) as u8; // zp/abs X -> Y
//     let l3 = (x == 0x20) as u8 * 7; // IMP -> ABS
//     let l4 = (x  == 0x6C) as u8 * 3; // ABS -> IND
//
//     addrmode += l0 + l1 + l2 + l3 + l4;
//
//     addrmode
// }

#[derive(Copy, Clone)]
pub struct Cpu {
    // Registers
    pub a: u8, // Accumulator
    pub x: u8,
    pub y: u8,
    pub stkpt: u8, // Stack Pointer
    pub pc: u16,   // Program Counter
    status: Flags6502,

    pub pipeline_status: PipelineStatus,
    pub page_boundary_crossed: bool,
    pub did_page_break_this_instruction: bool,
    pub internal_carry: bool,

    pub fetched: u8,
    pub temp: u8,

    pub addr_data: u16,

    pub opcode: u8,

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
    Exec6,
    IR,
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
            Self::Exec5 => Self::Exec6,
            Self::Exec6 => Self::IR,
            Self::IR => Self::Addr0,
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
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            stkpt: 0,
            pc: 0,
            status: Flags6502::empty(),
            pipeline_status: PipelineStatus::Addr0,
            page_boundary_crossed: false,
            did_page_break_this_instruction: false,
            internal_carry: false,
            fetched: 0,
            temp: 0,
            addr_data: 0,
            opcode: 0,
            cycles: 0,
        }
    }


    // Perform one instruction worth of emulation
    pub fn clock(&mut self, address_bus: &mut u16, data_bus: &mut u8, address_rw: &mut bool, phi: bool) -> bool {
        self.cycles = self.cycles.wrapping_add(phi as usize);
        let instruction = lookup::LOOKUP_TABLE[self.opcode as usize];
        
        self.execute(instruction.op(), instruction.addrmode(), address_bus, data_bus, address_rw, phi)
    }

    fn execute(&mut self, opcode: u8, addrmode: u8, address_bus: &mut u16, data_bus: &mut u8, address_rw: &mut bool, phi: bool) -> bool {

        let mut is_executing_stage = matches!(self.pipeline_status, PipelineStatus::Exec0 | PipelineStatus::Exec1 | PipelineStatus::Exec2 | PipelineStatus::Exec3 | PipelineStatus::Exec4 | PipelineStatus::Exec5 | PipelineStatus::Exec6);
        let mut is_ir_stage = matches!(self.pipeline_status, PipelineStatus::IR);
        let is_addr_stage = !is_executing_stage && !is_ir_stage;

        // Page boundary incurrs a +1 cycle cost
        if self.page_boundary_crossed {
            self.did_page_break_this_instruction = true;
            if !phi {
                *address_bus = self.addr_data;
                *address_rw = true;
                return false;
            } else {
                _ = *data_bus;
                let hi_byte = hi_byte(self.addr_data).wrapping_add(self.internal_carry as u8);
                set_hi_byte(&mut self.addr_data, hi_byte);
                self.page_boundary_crossed = false;
                self.internal_carry = false;
                return false;
            }
        }

        if is_addr_stage {
            let finished_addressing = self.execute_addrmode(opcode, addrmode, address_bus, data_bus, address_rw, phi);
            if finished_addressing {
                self.pipeline_status = PipelineStatus::Exec0;
                is_executing_stage = !phi;
            } else {
                if phi {
                    self.pipeline_status.advance(); 
                }
                return false;
            }
        }

        if is_executing_stage {
            let finished_executing = self.execute_instruction(opcode, addrmode, address_bus, data_bus, address_rw, phi);
            if finished_executing {
                self.pipeline_status = PipelineStatus::IR;
                is_ir_stage = !phi;
            } else if phi {
                self.pipeline_status.advance();
                return false;
            }
        }

        if is_ir_stage && !phi {
            *address_bus = self.pc;
            *address_rw = true;
            return false;
        }

        if is_ir_stage && phi {

            self.opcode = *data_bus;
            self.pc += 1;
            
            self.pipeline_status = PipelineStatus::Addr0;
            self.did_page_break_this_instruction = false;
            return true;
        }

        false
    }

    /// Returns true if the instruction should immediately begin executing
    /// an instruction  
    /// Returns false if the cpu should wait for a clock cycle
    fn execute_addrmode(&mut self, opcode: u8, addrmode: u8, address_bus: &mut u16, data_bus: &mut u8, address_rw: &mut bool, phi: bool) -> bool {
        use InstructionAddressingModes as Addr;
        use InstructionOperations as InsOp;
        let is_rwm = matches!(opcode, InsOp::DEC | InsOp::INC | InsOp::LSR | InsOp::ROL | InsOp::ROR | InsOp::ASL);
        let is_mw = matches!(opcode, InsOp::STA | InsOp::STX | InsOp::STY | InsOp::SAX);
        let do_pagebreak_anyways = matches!(opcode, InsOp::STA) || is_rwm;
        
        let skip_read = is_mw;

        let offset = match addrmode {
            Addr::ABX | Addr::ZPX => { self.x },
            Addr::ABY | Addr::ZPY => { self.y },
            _ => { 0 },
        };
        match (addrmode, self.pipeline_status, phi) {
            (Addr::IMP, PipelineStatus::Addr0, false) => { false },
            (Addr::IMP, PipelineStatus::Addr0, true ) => { true },

            (Addr::ACC, PipelineStatus::Addr0, false) => { self.fetched = self.a; false },
            (Addr::ACC, PipelineStatus::Addr0, true ) => { true },


            (Addr::IMM | Addr::REL, PipelineStatus::Addr0, false) => { *address_bus = self.pc; *address_rw = true; false },
            (Addr::IMM | Addr::REL, PipelineStatus::Addr0, true ) => { self.fetched = *data_bus; self.addr_data = *data_bus as u16; self.pc += 1; false }
            (Addr::IMM | Addr::REL, PipelineStatus::Addr1, false) => { true }, // Rel offset is stored in both fetched and addr_data
            (Addr::IMM | Addr::REL, PipelineStatus::Addr1, true ) => { true }, // shouldn't be reached but nonetheless

            (Addr::ZP0, PipelineStatus::Addr0, false) => { *address_bus = self.pc; *address_rw = true; false },
            (Addr::ZP0, PipelineStatus::Addr0, true ) => { self.fetched = *data_bus; self.addr_data = *data_bus as u16; self.pc += 1; skip_read }
            (Addr::ZP0, PipelineStatus::Addr1, false) => { *address_bus = self.addr_data; *address_rw = true; false }
            (Addr::ZP0, PipelineStatus::Addr1, true ) => { self.fetched = *data_bus; true },
            (Addr::ZP0, PipelineStatus::Addr2, false) => { true },
            (Addr::ZP0, PipelineStatus::Addr2, true ) => { true }, // shouldn't be reached but nonetheless

            // This functionality is common to all instructions with >1 bytes of opcode
            (_, PipelineStatus::Addr0, false) => { *address_bus = self.pc; *address_rw = true; false },
            (_, PipelineStatus::Addr0, true ) => { self.fetched = *data_bus; self.addr_data = *data_bus as u16; self.pc += 1; false }

            (Addr::ZPX, PipelineStatus::Addr1, false) => { self.addr_data = u8::wrapping_add(lo_byte(self.addr_data), self.x) as u16; false }, // These have no page-boundary since they will only access page 0
            (Addr::ZPX, PipelineStatus::Addr1, true ) => { skip_read }, // should dummy-read maybe
            (Addr::ZPX, PipelineStatus::Addr2, false) => { *address_bus = self.addr_data; *address_rw = true; false }
            (Addr::ZPX, PipelineStatus::Addr2, true ) => { self.fetched = *data_bus; false }
            (Addr::ZPX, PipelineStatus::Addr3, false) => { true },
            (Addr::ZPX, PipelineStatus::Addr3, true ) => { true }, // shouldn't be reached but nonetheless

            (Addr::ZPY, PipelineStatus::Addr1, false) => { self.addr_data = u8::wrapping_add(lo_byte(self.addr_data), self.y) as u16; false },
            (Addr::ZPY, PipelineStatus::Addr1, true ) => { skip_read }, // should dummy-read maybe
            (Addr::ZPY, PipelineStatus::Addr2, false) => { *address_bus = self.addr_data; *address_rw = true; false }
            (Addr::ZPY, PipelineStatus::Addr2, true ) => { self.fetched = *data_bus; false }
            (Addr::ZPY, PipelineStatus::Addr3, false) => { true },
            (Addr::ZPY, PipelineStatus::Addr3, true ) => { true }, // shouldn't be reached but nonetheless

            (Addr::ABS, PipelineStatus::Addr1, false) => { *address_bus = self.pc; *address_rw = true; self.pc += 1; false }
            (Addr::ABS, PipelineStatus::Addr1, true ) => { set_hi_byte(&mut self.addr_data, *data_bus); skip_read }
            (Addr::ABS, PipelineStatus::Addr2, false) => { 
                if opcode == InstructionOperations::JMP {
                    self.pc = self.addr_data;
                    return true;
                }
                *address_bus = self.addr_data; 
                *address_rw = true; 
                false 
            }
            (Addr::ABS, PipelineStatus::Addr2, true ) => { self.fetched = *data_bus; false }
            (Addr::ABS, PipelineStatus::Addr3, false) => { true },
            (Addr::ABS, PipelineStatus::Addr3, true ) => { true },

            (Addr::ABX, PipelineStatus::Addr1, false) => { *address_bus = self.pc; *address_rw = true; self.pc += 1; false },
            (Addr::ABX, PipelineStatus::Addr1, true ) => {
                set_hi_byte(&mut self.addr_data, *data_bus);
                let old_lo_byte = lo_byte(self.addr_data);
                let new_lo_byte = old_lo_byte.wrapping_add(offset);
                set_lo_byte(&mut self.addr_data, new_lo_byte);
                self.internal_carry = new_lo_byte < old_lo_byte;
                self.page_boundary_crossed = do_pagebreak_anyways || self.internal_carry;
                skip_read
            }
            (Addr::ABX, PipelineStatus::Addr2, false) => { *address_bus = self.addr_data; *address_rw = true; false }
            (Addr::ABX, PipelineStatus::Addr2, true ) => { self.fetched = *data_bus; false }
            (Addr::ABX, PipelineStatus::Addr3, false) => { true } // if this is DEC, this cycle is dedicated to fixing page boundary
            (Addr::ABX, PipelineStatus::Addr3, true ) => { true }

            (Addr::ABY, PipelineStatus::Addr1, false) => { *address_bus = self.pc; *address_rw = true; self.pc += 1; false },
            (Addr::ABY, PipelineStatus::Addr1, true ) => {
                set_hi_byte(&mut self.addr_data, *data_bus);
                let old_lo_byte = lo_byte(self.addr_data);
                let new_lo_byte = old_lo_byte.wrapping_add(offset);
                set_lo_byte(&mut self.addr_data, new_lo_byte);
                self.internal_carry = new_lo_byte < old_lo_byte;
                self.page_boundary_crossed = do_pagebreak_anyways || self.internal_carry;
                skip_read
            }
            (Addr::ABY, PipelineStatus::Addr2, false) => { *address_bus = self.addr_data; *address_rw = true; false }
            (Addr::ABY, PipelineStatus::Addr2, true ) => { self.fetched = *data_bus; false }
            (Addr::ABY, PipelineStatus::Addr3, false) => { true },
            (Addr::ABY, PipelineStatus::Addr3, true ) => { true },

            (Addr::IND, PipelineStatus::Addr1, false) => { *address_bus = self.pc; *address_rw = true; self.pc += 1; false },
            (Addr::IND, PipelineStatus::Addr1, true ) => { set_hi_byte(&mut self.addr_data, *data_bus); false }
            (Addr::IND, PipelineStatus::Addr2, false) => {
                *address_bus = self.addr_data;
                *address_rw = true;
                // emulate indirect page boundary bug
                let new_lo_byte = lo_byte(self.addr_data).wrapping_add(1);
                set_lo_byte(&mut self.addr_data, new_lo_byte);
                false
            },
            (Addr::IND, PipelineStatus::Addr2, true ) => { set_lo_byte(&mut self.pc, *data_bus); false }
            (Addr::IND, PipelineStatus::Addr3, false) => { *address_bus = self.addr_data; *address_rw = true; false },
            (Addr::IND, PipelineStatus::Addr3, true ) => { set_hi_byte(&mut self.pc, *data_bus); false },

            (Addr::IDX, PipelineStatus::Addr1, false) => {
                self.fetched = lo_byte(self.addr_data).wrapping_add(self.x); // TODO: this might be set after writing to addr
                *address_bus = self.fetched as u16;
                *address_rw = true;
                false
            },
            (Addr::IDX, PipelineStatus::Addr1, true) => {
                _ = *data_bus; // dummy read
                false
            }
            (Addr::IDX, PipelineStatus::Addr2, false) => {
                *address_bus = self.fetched as u16;
                *address_rw = true;
                false
            },
            (Addr::IDX, PipelineStatus::Addr2, true) => {
                set_lo_byte(&mut self.addr_data, *data_bus);
                false
            }
            (Addr::IDX, PipelineStatus::Addr3, false) => {
                self.fetched = self.fetched.wrapping_add(1);
                *address_bus = self.fetched as u16;
                *address_rw = true;
                false
            },
            (Addr::IDX, PipelineStatus::Addr3, true ) => { set_hi_byte(&mut self.addr_data, *data_bus); skip_read },
            (Addr::IDX, PipelineStatus::Addr4, false) => { *address_bus = self.addr_data; *address_rw = true; false },
            (Addr::IDX, PipelineStatus::Addr4, true ) => { self.fetched = *data_bus; false },

            (Addr::IDY, PipelineStatus::Addr1, false) => {
                *address_bus = self.addr_data;
                *address_rw = true;
                self.addr_data = self.addr_data.wrapping_add(1); // TODO: Check if this needs to wrap on ZP
                false
            }
            (Addr::IDY, PipelineStatus::Addr1, true) => {
                self.fetched = *data_bus; // put lo byte in fetched
                false
            }
            (Addr::IDY, PipelineStatus::Addr2, false) => {
                *address_bus = lo_byte(self.addr_data) as u16; // grab zp address before we replace it
                *address_rw = true;
                false
            }
            (Addr::IDY, PipelineStatus::Addr2, true ) => {
                set_hi_byte(&mut self.addr_data, *data_bus);
                let old_lo_byte = self.fetched;
                let new_lo_byte = self.fetched.wrapping_add(self.y);
                set_lo_byte(&mut self.addr_data, new_lo_byte);
                self.internal_carry = new_lo_byte < old_lo_byte; // page-boundary crossed if we wrapped around and addition creates a lower value than we started with
                self.page_boundary_crossed = do_pagebreak_anyways || self.internal_carry; // we always do a page-boundary fix in idy for some reason in STA
                skip_read
            }
            (Addr::IDY, PipelineStatus::Addr3, false) => {
                if do_pagebreak_anyways {
                    // we page breaked and ended up here when we were not supposed to
                    return true;
                }
                *address_bus = self.addr_data;
                *address_rw = true;
                false
            }
            (Addr::IDY, PipelineStatus::Addr3, true ) => { self.fetched = *data_bus; false }

            _ => { true }
        }
    }

    fn execute_instruction(&mut self, opcode: u8, addrmode: u8, address_bus: &mut u16, data_bus: &mut u8, address_rw: &mut bool, phi: bool) -> bool {
        use InstructionOperations as InsOp;
        use PipelineStatus as PS;
        match (opcode, self.pipeline_status, phi) {
            (InsOp::NOP, _, _) => {
                true
            },
            // Add with carry
            (InsOp::ADC, PS::Exec0, false) => {
                // let temp = self.a as u16 + self.fetched as u16 + self.get_flag(Flags6502::C) as u16;
                //
                // self.check_nzc_flags(temp);
                // self.set_flag(
                //     Flags6502::V,
                //     ((!(self.a as u16 ^ self.fetched as u16) & (self.a as u16 ^ temp)) & 0x0080)
                //         > 0,
                // );
                // self.a = (temp & 0x00FF) as u8;
                self.add_carry(self.fetched);
                true
            }
            (InsOp::ADC, PS::Exec0, true ) => { true } // possible unneeded
            // Logical And (&)
            (InsOp::AND, PS::Exec0, false) => {
                self.a &= self.fetched;
                self.check_nz_flags(self.a);
                true
            }
            (InsOp::AND, PS::Exec0, true ) => { true } // possible unneeded
            // Arithmatic Shift Left ( A|M << 1 OR A|M * 2)
            (InsOp::ASL, PS::Exec0, false) => {
                let temp = self.fetched << 1;
                self.check_nz_flags(temp);
                self.set_flag(Flags6502::C, (self.fetched & 0x80) > 0);

                if addrmode == InstructionAddressingModes::ACC {
                    self.a = temp;
                    true
                } else {
                    self.fetched = temp;
                    false
                }
            },
            (InsOp::ASL, PS::Exec0, true) => { false }
            (InsOp::ASL, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false },
            (InsOp::ASL, PS::Exec1, true) => { *data_bus = self.fetched; false }
            (InsOp::ASL, PS::Exec2, _) => { true }

            // Branch if Carry Clear
            (InsOp::BCC, PS::Exec0, false) => {
                self.branch(!self.get_flag(Flags6502::C))
            }
            // Branch is Carry Set
            (InsOp::BCS, PS::Exec0, false) => {
                self.branch(self.get_flag(Flags6502::C))
            }
            // Branch if Equal (Branch if Zero Set)
            (InsOp::BEQ, PS::Exec0, false) => {
                self.branch(self.get_flag(Flags6502::Z))
            }
            // Branch if Minus (Branch if Negative Flag Set)
            (InsOp::BMI, PS::Exec0, false) => {
                self.branch(self.get_flag(Flags6502::N))
            }
            // Branch if Not Equal (Branch if Zero Clear)
            (InsOp::BNE, PS::Exec0, false) => {
                self.branch(!self.get_flag(Flags6502::Z))
            }
            // Branch is Positive (Branch if Negative Clear)
            (InsOp::BPL, PS::Exec0, false) => {
                self.branch(!self.get_flag(Flags6502::N))
            }
            // Branch if Overflow Clear
            (InsOp::BVC, PS::Exec0, false) => {
                self.branch(!self.get_flag(Flags6502::V))
            }
            // Branch if Overflow Set
            (InsOp::BVS, PS::Exec0, false) => {
                self.branch(self.get_flag(Flags6502::V))
            }
            (InsOp::BCC | InsOp::BCS | InsOp::BEQ | InsOp::BMI | InsOp::BNE | InsOp::BPL | InsOp::BVC | InsOp::BVS, PS::Exec0, true) => { false }
            // Intercpting page-break handler will handle this for us
            (InsOp::BCC | InsOp::BCS | InsOp::BEQ | InsOp::BMI | InsOp::BNE | InsOp::BPL | InsOp::BVC | InsOp::BVS, PS::Exec1, false) => {
                // const sign_extended: u16 = b as i8 as i16 as u16;
                // const high_offset: u8 = (sign_extended >> 8) as u8;
                let sign_extended = hi_byte(self.fetched as i8 as i16 as u16);
                let overflowed = self.internal_carry;
                let underflowed = sign_extended > 0;
                // If the high byte is incorrect due to passing a page boundary or subtraction
                // Insert fix-PC_H microcode
                if (overflowed || underflowed) && !(overflowed && underflowed) {
                    let old_hi_byte = hi_byte(self.pc);
                    let new_hi_byte = old_hi_byte.wrapping_add(sign_extended).wrapping_add(self.internal_carry as u8);
                    set_hi_byte(&mut self.pc, new_hi_byte);
                    self.internal_carry = false;
                    false
                } else {
                    // we are done, fetch next instruction
                    true
                }
                // false
            }
            (InsOp::BCC | InsOp::BCS | InsOp::BEQ | InsOp::BMI | InsOp::BNE | InsOp::BPL | InsOp::BVC | InsOp::BVS, PS::Exec1, true) => { true }
            (InsOp::BCC | InsOp::BCS | InsOp::BEQ | InsOp::BMI | InsOp::BNE | InsOp::BPL | InsOp::BVC | InsOp::BVS, PS::Exec2, _) => { true } // possibly unneeded
            // Bit Test
            (InsOp::BIT, PS::Exec0, false) => {
                let temp = self.a & self.fetched;
                self.check_z_flag(temp);
                self.set_flag(Flags6502::N, (self.fetched & (1 << 7)) > 0);
                self.set_flag(Flags6502::V, (self.fetched & (1 << 6)) > 0);
                true
            },
            (InsOp::BIT, PS::Exec0, true) => { true },
            // BRK - Force Inturrupt
            (InsOp::BRK, PS::Exec0, false) => {
                *address_bus = self.addr_data;
                *address_rw = true;
                false
            }
            (InsOp::BRK, PS::Exec0, true) => {
                _ = *data_bus;
                self.pc = self.pc.wrapping_add(1);
                false
            }
            (InsOp::BRK, PS::Exec1, false) => {
                self.fetched = hi_byte(self.pc);
                *address_bus = 0x0100 + (self.stkpt as u16);
                *address_rw = false;
                self.stkpt = self.stkpt.wrapping_sub(1);
                false
            },
            (InsOp::BRK, PS::Exec1, true) => { *data_bus = self.fetched; false }
            (InsOp::BRK, PS::Exec2, false) => {
                self.fetched = lo_byte(self.pc);
                *address_bus = 0x0100 + (self.stkpt as u16);
                *address_rw = false;
                self.stkpt = self.stkpt.wrapping_sub(1);
                false
            },
            (InsOp::BRK, PS::Exec2, true) => { *data_bus = self.fetched; false}
            (InsOp::BRK, PS::Exec3, false) => {
                self.fetched = (self.status | Flags6502::B | Flags6502::U).bits();
                *address_bus = 0x0100 + (self.stkpt as u16);
                *address_rw = false;
                self.stkpt = self.stkpt.wrapping_sub(1);
                self.set_flag(Flags6502::I, true);
                false
            },
            (InsOp::BRK, PS::Exec3, true) => { *data_bus = self.fetched; false }
            (InsOp::BRK, PS::Exec4, false) => { *address_bus = 0xFFFE; *address_rw = true; false },
            (InsOp::BRK, PS::Exec4, true) => { set_lo_byte(&mut self.pc, *data_bus); false }
            (InsOp::BRK, PS::Exec5, false) => { *address_bus = 0xFFFF; *address_rw = true; false },
            (InsOp::BRK, PS::Exec5, true) => { set_hi_byte(&mut self.pc, *data_bus); false },
            (InsOp::BRK, PS::Exec6, _) => { true },
            // Clear Carry Flag
            (InsOp::CLC, PS::Exec0, _) => { self.set_flag(Flags6502::C, false); true }
            // Clear Decimal Mode
            (InsOp::CLD, PS::Exec0, _) => { self.set_flag(Flags6502::D, false); true }
            // Clear Inturrupt Disable
            (InsOp::CLI, PS::Exec0, _) => { self.set_flag(Flags6502::I, false); true }
            // Clear Overflow Flag
            (InsOp::CLV, PS::Exec0, _) => { self.set_flag(Flags6502::V, false); true }
            // Compare
            (InsOp::CMP, PS::Exec0, _) => { self.compare(self.a, self.fetched); true }
            // Compare X Register
            (InsOp::CPX, PS::Exec0, _) => { self.compare(self.x, self.fetched); true }
            // Compare Y Register
            (InsOp::CPY, PS::Exec0, _) => { self.compare(self.y, self.fetched); true }
            // Decrement value at address
            (InsOp::DEC, PS::Exec0, false) => {
                self.fetched = self.decrement(self.fetched);
                false
            },
            (InsOp::DEC, PS::Exec0, true) => { false },
            (InsOp::DEC, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false },
            (InsOp::DEC, PS::Exec1, true) => { *data_bus = self.fetched; false }
            (InsOp::DEC, PS::Exec2, _) => { true }
            // Decrement X register
            (InsOp::DEX, PS::Exec0, false) => { self.x = self.decrement(self.x); true }
            (InsOp::DEX, PS::Exec0, true) => { true } // possibly unneeded

            // Decrement Y register
            (InsOp::DEY, PS::Exec0, false) => { self.y = self.decrement(self.y); true }
            (InsOp::DEY, PS::Exec0, true) => { true } // possibly unneeded
            // Exclusive OR
            (InsOp::EOR, PS::Exec0, false) => { self.a = self.a ^ self.fetched; self.check_nz_flags(self.a); true }
            (InsOp::EOR, PS::Exec0, true) => { true } // possibly unneeded
            // Increment memory at address
            (InsOp::INC, PS::Exec0, false) => { *address_bus = self.addr_data; *address_rw = false; false }, // garbage write
            (InsOp::INC, PS::Exec0, true) => { *data_bus = self.fetched; self.fetched = self.increment(self.fetched); false },
            (InsOp::INC, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false } // real write
            (InsOp::INC, PS::Exec1, true) => { *data_bus = self.fetched; false }
            (InsOp::INC, PS::Exec2, _) => { true }
            // Increment X register
            (InsOp::INX, PS::Exec0, false) => { self.x = self.increment(self.x); true }
            (InsOp::INX, PS::Exec0, true) => { true } // possibly unneeded
            // Increment Y register
            (InsOp::INY, PS::Exec0, false) => { self.y = self.increment(self.y); true }
            (InsOp::INY, PS::Exec0, true) => { true } // possibly unneeded
            // Jump to address
            (InsOp::JMP, PS::Exec0, _) => { true } //self.pc = self.addr_data; // already taken care of in addrmode execution
            // Jump to subroutine
            // INFO: JSR is apparently done very weirdly:
            // 1. Read lo byte of data
            // 2. Read stkptr for some reason
            // 3. Push high byte of pc
            // 4. Push low byte of pc
            // 5. Read hi byte of data
            // 6. Read next instruction
            // I read the new pc, push the old pc,
            // then set pc to the new pc
            (InsOp::JSR, PS::Exec0, false) => {
                self.pc -= 1;
                *address_bus = 0x0100 + self.stkpt as u16;
                *address_rw = false;
                self.fetched = hi_byte(self.pc);
                self.stkpt = self.stkpt.wrapping_sub(1);
                false
            },
            (InsOp::JSR, PS::Exec0, true) => {
                *data_bus = self.fetched;
                false 
            }
            (InsOp::JSR, PS::Exec1, false) => {
                *address_bus = 0x0100 + self.stkpt as u16;
                *address_rw = false;
                self.fetched = lo_byte(self.pc);
                self.stkpt = self.stkpt.wrapping_sub(1);
                self.pc = self.addr_data;
                false
            }
            (InsOp::JSR, PS::Exec1, true) => {
                *data_bus = self.fetched;
                false
            }
            (InsOp::JSR, PS::Exec2, _) => { true }

            // Load Accumulator
            (InsOp::LDA, PS::Exec0, false) => {
                self.a = self.fetched;
                self.check_nz_flags(self.a);
                true
            }
            (InsOp::LDA, PS::Exec0, true) => { true } // possibly unneeded

            // Load X register
            (InsOp::LDX, PS::Exec0, false) => {
                self.x = self.fetched;
                self.check_nz_flags(self.x);
                true
            }
            (InsOp::LDX, PS::Exec0, true) => { true } // possibly unneeded
            // Load Y register
            (InsOp::LDY, PS::Exec0, false) => {
                self.y = self.fetched;
                self.check_nz_flags(self.y);
                true
            }
            (InsOp::LDY, PS::Exec0, true) => { true } // possibly unneeded
            // Logical Shift Right
            (InsOp::LSR, PS::Exec0, false) => {
                self.set_flag(Flags6502::C, self.fetched & 0x0001 > 0);
                let temp = self.fetched >> 1;
                self.check_nz_flags(temp);

                if addrmode == InstructionAddressingModes::ACC {
                    self.a = temp;
                    true
                } else {
                    self.fetched = temp;
                    false
                }
            },
            (InsOp::LSR, PS::Exec0, true) => { false }
            (InsOp::LSR, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false }
            (InsOp::LSR, PS::Exec1, true) => { *data_bus = self.fetched; true }

            // Logical Inclusive OR
            (InsOp::ORA, PS::Exec0, false) => {
                self.a = self.a | self.fetched;
                self.check_nz_flags(self.a);
                true
            }
            (InsOp::ORA, PS::Exec0, true) => { true } // possibly unneeded
            // Push Accumulator (to stack)
            (InsOp::PHA, PS::Exec0, false) => {
                *address_bus = 0x0100 + self.stkpt as u16;
                *address_rw = false;
                self.fetched = self.a;
                self.stkpt = self.stkpt.wrapping_sub(1);
                false
            }
            (InsOp::PHA, PS::Exec0, true) => { *data_bus = self.fetched; true }
            // Push Processor Stack
            (InsOp::PHP, PS::Exec0, false) => {
                *address_bus = 0x0100 + self.stkpt as u16;
                *address_rw = false;
                // B and U are pushed as set
                self.fetched = (self.get_status() | Flags6502::B | Flags6502::U).bits(); 
                self.stkpt = self.stkpt.wrapping_sub(1);
                false
            }
            (InsOp::PHP, PS::Exec0, true) => { *data_bus = self.fetched; true }
            // Pull Accumulator
            (InsOp::PLA, PS::Exec0, false) => { *address_bus = 0x0100 + self.stkpt as u16; *address_rw = true; false }
            (InsOp::PLA, PS::Exec0, true) => { _ = *data_bus; self.check_nz_flags(self.a); self.stkpt = self.stkpt.wrapping_add(1); false }
            (InsOp::PLA, PS::Exec1, false) => {  *address_bus = 0x0100 + self.stkpt as u16; *address_rw = true; false }
            (InsOp::PLA, PS::Exec1, true) => { self.a = *data_bus; self.check_nz_flags(self.a); true }
            // Pull Processor Stack
            (InsOp::PLP, PS::Exec0, false) => { *address_bus = 0x0100 + self.stkpt as u16; *address_rw = true; false }
            (InsOp::PLP, PS::Exec0, true) => { _ = *data_bus; self.check_nz_flags(self.a); self.stkpt = self.stkpt.wrapping_add(1); false }
            (InsOp::PLP, PS::Exec1, false) => { *address_bus = 0x0100 + self.stkpt as u16; *address_rw = true; false }
            (InsOp::PLP, PS::Exec1, true) => {
                self.status = Flags6502::from_bits_retain(
                    *data_bus & 0b11001111
                    | self.status.bits() & 0b00110000
                );
                true
            }
            // Rotate Left
            (InsOp::ROL, PS::Exec0, false) => {
                let temp = ((self.fetched) << 1) | self.get_flag(Flags6502::C) as u8;
                self.set_flag(Flags6502::C, (self.fetched & 0x80) > 0);
                self.check_nz_flags(temp);
                if addrmode == InstructionAddressingModes::ACC {
                    self.a = temp;
                    true
                } else {
                    self.fetched = temp;
                    false
                }
            }
            (InsOp::ROL, PS::Exec0, true) => { false }
            (InsOp::ROL, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false }
            (InsOp::ROL, PS::Exec1, true) => { *data_bus = self.fetched; false }
            (InsOp::ROL, PS::Exec2, _) => { true }
            // Rotate Right
            (InsOp::ROR, PS::Exec0, false) => {
                let temp = ((self.get_flag(Flags6502::C) as u8) << 7) | self.fetched >> 1;
                self.set_flag(Flags6502::C, self.fetched & 0x01 > 0);
                self.check_nz_flags(temp);

                if addrmode == InstructionAddressingModes::ACC {
                    self.a = temp;
                    true
                } else {
                    self.fetched = temp;
                    false
                }
            }
            (InsOp::ROR, PS::Exec0, true) => { false }
            (InsOp::ROR, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false }
            (InsOp::ROR, PS::Exec1, true) => { *data_bus = self.fetched; false }
            (InsOp::ROR, PS::Exec2, _) => { true }
            // Return from Inturrupt
            (InsOp::RTI, PS::Exec0, false) => { *address_bus = 0x0100 + self.stkpt as u16; *address_rw = true; self.stkpt = self.stkpt.wrapping_add(1); false },
            (InsOp::RTI, PS::Exec0, true) => { self.fetched = *data_bus; false } // dummy read
            (InsOp::RTI, PS::Exec1, false) => { *address_bus = 0x0100 + self.stkpt as u16; *address_rw = true; self.stkpt = self.stkpt.wrapping_add(1); false },
            (InsOp::RTI, PS::Exec1, true) => {
                let b = (*data_bus & 0b11001111) | (self.status.bits() & 0b00110000);
                self.status = Flags6502::from_bits_retain(b);
                false
            }
            (InsOp::RTI, PS::Exec2, false) => { *address_bus = 0x0100 + self.stkpt as u16; *address_rw = true; self.stkpt = self.stkpt.wrapping_add(1); false },
            (InsOp::RTI, PS::Exec2, true) => { set_lo_byte(&mut self.pc, *data_bus); false }
            (InsOp::RTI, PS::Exec3, false) => { *address_bus = 0x0100 + self.stkpt as u16; *address_rw = true; false }
            (InsOp::RTI, PS::Exec3, true) => { set_hi_byte(&mut self.pc, *data_bus); false }
            (InsOp::RTI, PS::Exec4, _) => { true }
            // Return from subroutine
            (InsOp::RTS, PS::Exec0, false) => { self.stkpt = self.stkpt.wrapping_add(1); *address_bus = 0x0100 + self.stkpt as u16; *address_rw = true; false },
            (InsOp::RTS, PS::Exec0, true) => { let _ = *data_bus; false } // dummy read
            (InsOp::RTS, PS::Exec1, false) => { *address_bus = 0x0100 + self.stkpt as u16; *address_rw = true; self.stkpt = self.stkpt.wrapping_add(1); false },
            (InsOp::RTS, PS::Exec1, true) => { set_lo_byte(&mut self.pc, *data_bus); false }
            (InsOp::RTS, PS::Exec2, false) => { *address_bus = 0x0100 + self.stkpt as u16; *address_rw = true; false }
            (InsOp::RTS, PS::Exec2, true) => { set_hi_byte(&mut self.pc, *data_bus); false }
            (InsOp::RTS, PS::Exec3, false) => { *address_bus = self.pc; *address_rw = true; false }
            (InsOp::RTS, PS::Exec3, true) => { _ = *data_bus; self.pc += 1; false }
            (InsOp::RTS, PS::Exec4, _) => { true }

            // Subtract with carry
            (InsOp::SBC, PS::Exec0, false) => {
                self.add_carry(!self.fetched);
                true
            }
            (InsOp::SBC, PS::Exec0, true) => { true }
            // Set carry flag
            (InsOp::SEC, PS::Exec0, _) => { self.set_flag(Flags6502::C, true); true }
            // Set decimal flag
            (InsOp::SED, PS::Exec0, _) => { self.set_flag(Flags6502::D, true); true }
            // Set inturrupt disable
            (InsOp::SEI, PS::Exec0, _) => { self.set_flag(Flags6502::I, true); true }
            // Store Accumulator
            (InsOp::STA, PS::Exec0, false) => { *address_bus = self.addr_data; *address_rw = false; self.fetched = self.a; false }
            (InsOp::STA, PS::Exec0, true) => { *data_bus = self.fetched; true }
            (InsOp::STA, PS::Exec1, _) => { true }
            // Store X register
            (InsOp::STX, PS::Exec0, false) => { *address_bus = self.addr_data; *address_rw = false; self.fetched = self.x; false }
            (InsOp::STX, PS::Exec0, true) => { *data_bus = self.fetched; false }
            (InsOp::STX, PS::Exec1, _) => { true }
            // Store Y register
            (InsOp::STY, PS::Exec0, false) => { *address_bus = self.addr_data; *address_rw = false; self.fetched = self.y; false }
            (InsOp::STY, PS::Exec0, true) => { *data_bus = self.fetched; false }
            (InsOp::STY, PS::Exec1, _) => { true }
            // Transfer Accumulator to X
            (InsOp::TAX, PS::Exec0, false) => { self.x = self.a; self.check_nz_flags(self.x); true }
            (InsOp::TAX, PS::Exec0, true) => {  true } // possibly unneeded
            // Transfer Accumulator to Y
            (InsOp::TAY, PS::Exec0, false) => { self.y = self.a; self.check_nz_flags(self.y); true }
            (InsOp::TAY, PS::Exec0, true) => {  true } // possibly unneeded
            // Transfer Stack Pointer to X
            (InsOp::TSX, PS::Exec0, false) => { self.x = self.stkpt; self.check_nz_flags(self.x); true }
            (InsOp::TSX, PS::Exec0, true) => {  true } // possibly unneeded
            // Transfer Stack Pointer to A
            (InsOp::TXA, PS::Exec0, false) => { self.a = self.x; self.check_nz_flags(self.a); true }
            (InsOp::TXA, PS::Exec0, true) => {  true } // possibly unneeded
            // Transfer X to Stack Pointer
            (InsOp::TXS, PS::Exec0, false) => { self.stkpt = self.x; true }
            (InsOp::TXS, PS::Exec0, true) => {  true } // possibly unneeded
            // Transfer Y to Accumulator
            (InsOp::TYA, PS::Exec0, false) => { self.a = self.y; self.check_nz_flags(self.a); true }
            (InsOp::TYA, PS::Exec0, true) => {  true } // possibly unneeded

            // Some of these instructions are a tad confusing since NoMoreSecrets doesn't state whether A is
            // operated with the original fetched value or the altered value. Just going to assume
            // project c74 is correct
            (InsOp::SLO, PS::Exec0, false) => { false }
            (InsOp::SLO, PS::Exec0, true) => { 
                self.temp = self.fetched << 1; 
                self.set_flag(Flags6502::C, (self.fetched & 0x80) > 0); 
                self.check_nz_flags(self.temp); 
                false 
            }
            (InsOp::SLO, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false }
            (InsOp::SLO, PS::Exec1, true) => { *data_bus = self.temp; false }
            (InsOp::SLO, PS::Exec2, false) => {
                self.a = self.a | self.temp;
                self.check_nz_flags(self.a);
                true
            } // this could  potentially be a phi2 operation, but its in phi1 due to the need for IR to also be fetched, same for similar following operatins
            (InsOp::SLO, PS::Exec2, true) => { true }

            (InsOp::RLA, PS::Exec0, false) => { false }
            (InsOp::RLA, PS::Exec0, true ) => {
                self.temp = (self.fetched << 1) | (self.get_flag(Flags6502::C) as u8);
                self.set_flag(Flags6502::C, (self.fetched & 0x80) > 0);
                self.check_nz_flags(self.temp);
                false
            }
            (InsOp::RLA, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false }
            (InsOp::RLA, PS::Exec1, true ) => { *data_bus = self.temp; false }
            (InsOp::RLA, PS::Exec2, false) => {
                self.a = self.a & self.temp;
                self.check_nz_flags(self.a);
                true
            }
            (InsOp::RLA, PS::Exec2, true ) => { true }

            (InsOp::SRE, PS::Exec0, false) => { false }
            (InsOp::SRE, PS::Exec0, true ) => {
                self.temp = self.fetched >> 1;
                self.set_flag(Flags6502::C, (self.fetched & 0x01) > 0);
                self.check_nz_flags(self.temp);
                false
            }
            (InsOp::SRE, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false }
            (InsOp::SRE, PS::Exec1, true ) => { *data_bus = self.temp; false }
            (InsOp::SRE, PS::Exec2, false) => {
                self.a = self.a ^ self.temp;
                self.check_nz_flags(self.a);
                true
            }
            (InsOp::SRE, PS::Exec2, true ) => { true }

            (InsOp::RRA, PS::Exec0, false) => { false }
            (InsOp::RRA, PS::Exec0, true ) => {
                self.temp = (self.fetched >> 1) | ((self.get_flag(Flags6502::C) as u8) << 7);
                self.set_flag(Flags6502::C, (self.fetched & 0x01) > 0);
                self.check_nz_flags(self.temp);
                false
            }
            (InsOp::RRA, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false }
            (InsOp::RRA, PS::Exec1, true ) => { *data_bus = self.temp; false }
            (InsOp::RRA, PS::Exec2, false) => {
                self.add_carry(self.temp);
                true
            }
            (InsOp::RRA, PS::Exec2, true ) => { true }

            (InsOp::SAX, PS::Exec0, false) => { *address_bus = self.addr_data; *address_rw = false; false }
            (InsOp::SAX, PS::Exec0, true ) => { *data_bus = self.a & self.x; true }
            (InsOp::SAX, PS::Exec1, _ ) => { true }

            (InsOp::LAX, PS::Exec0, false) => { self.a = self.fetched; self.x = self.fetched; self.check_nz_flags(self.a); true } // confirm cycle counts
            (InsOp::LAX, PS::Exec0, true ) => { true }

            (InsOp::DCP, PS::Exec0, false) => { false }
            (InsOp::DCP, PS::Exec0, true ) => { self.temp = self.decrement(self.fetched); false }
            (InsOp::DCP, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false }
            (InsOp::DCP, PS::Exec1, true ) => { *data_bus = self.temp; false }
            (InsOp::DCP, PS::Exec2, false) => { self.compare(self.a, self.temp); true } // no idea if cmp should use temp or fetched
            (InsOp::DCP, PS::Exec2, true ) => { true }

            (InsOp::ISC, PS::Exec0, false) => { false }
            (InsOp::ISC, PS::Exec0, true ) => { self.temp = self.increment(self.fetched); false }
            (InsOp::ISC, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false }
            (InsOp::ISC, PS::Exec1, true ) => { *data_bus = self.temp; false }
            (InsOp::ISC, PS::Exec2, false) => { self.add_carry(!self.temp); true }
            (InsOp::ISC, PS::Exec2, true ) => { true }

            (InsOp::ANC, PS::Exec0, false) => {
                self.a = self.a & self.fetched;
                self.check_nz_flags(self.a);
                self.set_flag(Flags6502::C, self.get_flag(Flags6502::N));
                true
            }
            (InsOp::ANC, PS::Exec0, true ) => { true }

            (InsOp::ALR, PS::Exec0, false) => {
                self.temp = self.a & self.fetched;
                self.a = self.temp >> 1;
                self.set_flag(Flags6502::C, (self.temp & 0x01) > 0);
                self.check_nz_flags(self.a);
                true
            }
            (InsOp::ALR, PS::Exec0, true ) => { true }

            (InsOp::ARR, PS::Exec0, false) => {
                let carry_in = self.get_flag(Flags6502::C) as u8;
                self.temp = self.a & self.fetched;
                let b7 = (self.temp & 0x80) >> 1;
                let b6 =  self.temp & 0x40;
                self.set_flag(Flags6502::C, b7 > 0);
                self.set_flag(Flags6502::V, (b7 ^ b6) > 0);
                self.a = (self.temp >> 1) | (carry_in << 7); // confirm C flag is inserted at the right location
                self.check_nz_flags(self.a);
                true
            }
            (InsOp::ARR, PS::Exec0, true ) => { true }

            (InsOp::SBX, PS::Exec0, false) => {
                self.compare(self.a & self.x, self.fetched);
                self.x = self.temp;
                true
            }
            (InsOp::SBX, PS::Exec0, true ) => { true }

            (InsOp::LAS, PS::Exec0, false) => {
                self.temp = self.fetched & self.stkpt;
                self.check_nz_flags(self.temp);
                self.a = self.temp;
                self.x = self.temp;
                self.stkpt = self.temp;
                true
            }
            (InsOp::LAS, PS::Exec0, true ) => { true }

            (InsOp::SHA, PS::Exec0, false) => {
                self.temp = self.a & self.x;
                *address_bus = self.instable_store_address(self.temp, self.addr_data);
                *address_rw = false;
                false
            }
            (InsOp::SHA, PS::Exec0, true ) => { *data_bus = self.instable_store_value(self.temp, self.addr_data); true }
            (InsOp::SHA, PS::Exec1, _) => { true }

            (InsOp::SHX, PS::Exec0, false) => {
                self.temp = self.x;
                *address_bus = self.instable_store_address(self.temp, self.addr_data);
                *address_rw = false;
                false
            }
            (InsOp::SHX, PS::Exec0, true ) => { *data_bus = self.instable_store_value(self.temp, self.addr_data); true }
            (InsOp::SHX, PS::Exec1, _) => { true }

            (InsOp::SHY, PS::Exec0, false) => {
                self.temp = self.y;
                *address_bus = self.instable_store_address(self.temp, self.addr_data);
                *address_rw = false;
                false
            }
            (InsOp::SHY, PS::Exec0, true ) => { *data_bus = self.instable_store_value(self.temp, self.addr_data); true }
            (InsOp::SHY, PS::Exec1, _) => { true }

            (InsOp::TAS, PS::Exec0, false) => { false }
            (InsOp::TAS, PS::Exec0, true ) => { self.stkpt = self.a & self.x; false }
            (InsOp::TAS, PS::Exec1, false) => {
                *address_bus = self.instable_store_address(self.stkpt, self.addr_data);
                *address_rw = false;
                false
            }
            (InsOp::TAS, PS::Exec1, true ) => { *data_bus = self.instable_store_value(self.stkpt, self.addr_data); true }
            (InsOp::TAS, PS::Exec2, _) => { true }

            (InsOp::ANE, PS::Exec0, false) => {
                let magic_number = 0xEE; // this is hardcoded but should be completely random or dependant on the RDY line; its recommended to use 0xEE when RDY enabled and 0xFF otherwise
                self.a = (self.a | magic_number) & self.x & self.fetched;
                self.check_nz_flags(self.a);
                true
            }
            (InsOp::ANX, PS::Exec0, false) => {
                let magic_number = 0xEE; // this is hardcoded but should be completely random or dependant on the RDY line; its recommended to use 0xEE
                self.temp = (self.a | magic_number) & self.fetched;
                self.a = self.temp;
                self.x = self.temp;
                self.check_nz_flags(self.a); // TODO: find is this is correct. This being set after
                                             // calculation is the only way it works with
                                             // SingStepTests, but is indicated by NoMoreSecrets to
                                             // be set before calculations
                true
            }

            (InsOp::JAM, _, _) => { false } // fuck you, gotta RESET now


            _ => { false } // Illegal Instruction
        }
    }

    // Forces the 6502 into a known state. This is hard-wired inside the CPU. The
    // registers are set to 0x00, the status register is cleared except for unused
    // bit which remains at 1. An absolute address is read from location 0xFFFC
    // which contains a second address that the program counter is set to. This
    // allows the programmer to jump to a known and programmable location in the
    // memory to start executing from. Typically the programmer would set the value
    // at location 0xFFFC at compile time.
    pub fn reset(&mut self) {
        // self.addr_data = 0xFFFC;
        //
        // self.pc = self.read_word(self.addr_data);
        //
        // self.a = 0;
        // self.x = 0;
        // self.y = 0;
        // self.stkpt = 0xFD;
        // self.status = Flags6502::U;
        //
        // self.fetched = 0;
        // self.cycles = 7;
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
    pub fn irq(&mut self) {
        // // If interrupts are allowed
        // if !self.get_flag(Flags6502::I) {
        //     // Push the program counter to the stack. It's 16-bits dont
        //     // forget so that takes two pushes
        //     self.write(
        //         0x0100 + self.stkpt as u16,
        //         ((self.pc >> 8) & 0x00FF) as u8,
        //     );
        //     self.stkpt -= 1;
        //     self.write(0x0100 + self.stkpt as u16, (self.pc & 0x00FF) as u8);
        //     self.stkpt -= 1;
        //
        //     // Then Push the status register to the stack
        //     self.set_flag(Flags6502::B, false);
        //     self.set_flag(Flags6502::U, true);
        //     self.set_flag(Flags6502::I, true);
        //     self.write(0x0100 + self.stkpt as u16, self.get_status().bits());
        //     self.stkpt -= 1;
        //
        //     // Read new program counter location from fixed address
        //     self.addr_data = 0xFFFE;
        //     self.pc = self.read_word(self.addr_data);
        //
        //     // IRQs take time
        //     // cycles.add_assign(2);
        // }
    }
    // A Non-Maskable Interrupt cannot be ignored. It behaves in exactly the
    // same way as a regular IRQ, but reads the new program counter address
    // from location 0xFFFA.
    pub fn nmi(&mut self) {
        // if !self.get_flag(Flags6502::I) {
        //     self.write(
        //         0x0100 + self.stkpt as u16,
        //         hi_byte(self.pc),
        //     );
        //     self.stkpt -= 1;
        //     self.write(0x0100 + self.stkpt as u16, lo_byte(self.pc));
        //     self.stkpt -= 1;
        //
        //     self.set_flag(Flags6502::B, false);
        //     self.set_flag(Flags6502::U, true);
        //     self.set_flag(Flags6502::I, true);
        //     self.write(0x0100 + self.stkpt as u16, self.get_status().bits());
        //     self.stkpt -= 1;
        //
        //     self.addr_data = 0xFFFA;
        //     self.pc = self.read_word(self.addr_data);
        //
        //     // cycles.add_assign(3);
        // }
    }

    // internal helpers
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
    fn branch(&mut self, flag: bool) -> bool {
        if flag {
            let old_lo_byte = lo_byte(self.pc);
            let new_lo_byte = old_lo_byte.wrapping_add(self.fetched);
            set_lo_byte(&mut self.pc, new_lo_byte);
            self.internal_carry = new_lo_byte < old_lo_byte; // X + FF = X-1
            // self.page_boundary_crossed = self.internal_carry;
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
        self.temp = a.wrapping_sub(b);
        self.check_nz_flags(self.temp);
        self.set_flag(Flags6502::C, a >= b);
    }
    fn add_carry(&mut self, b: u8) {
        let temp = self.a as u16 + b as u16 + self.get_flag(Flags6502::C) as u16;

        self.check_nzc_flags(temp);
        self.set_flag(
            Flags6502::V,
            ((!(self.a as u16 ^ b as u16) & (self.a as u16 ^ temp)) & 0x0080) > 0,
        );
        self.a = lo_byte(temp);
    }

    fn instable_store_value(&self, value: u8, address: u16) -> u8 {
        let dmad = false;
        let page_breaked = self.did_page_break_this_instruction;
        let hi_byte = hi_byte(address);
        if !dmad { hi_byte.wrapping_add(1 - page_breaked as u8) & value } else { value }
    }
    fn instable_store_address(&self, value: u8, mut address: u16) -> u16 {
        let page_crossed = self.did_page_break_this_instruction;
        let hi_byte = hi_byte(address);
        let hi_byte = if page_crossed { hi_byte/*.wrapping_add(1)*/ & value } else { hi_byte }; // hi byte is already taken care of
        set_hi_byte(&mut address, hi_byte);
        address
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
