use bitflags::bitflags;
use std::fmt;

use crate::cpu::instructions::stringify_ins_from_log;
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

#[derive(Copy, Clone, Default)]
pub struct CpuLog {
    pub count_bytes: u8,
    pub byte0: u8,
    pub byte1: u8,
    pub byte2: u8,
    pub start_address: u16,
    pub opcode: u8,
    pub addrmode: u8,
    pub start_cycle: usize,
}

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

    pub fetched: u8,

    pub addr_data: u16,

    pub opcode: u8,
    pub cycles: usize,

    pub cpu_log: CpuLog,

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
            Self::Exec5 => Self::IR,
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
            fetched: 0,
            addr_data: 0,
            opcode: 0,
            cycles: 0,
            cpu_log: CpuLog::default(),
        }
    }

    pub fn cycles(&mut self) -> usize {
        self.cycles
    }

    fn log_byte0(&mut self, b: u8) {
        self.cpu_log.count_bytes = 1;
        self.cpu_log.byte0 = b;
    }
    fn log_byte1(&mut self, b: u8) {
        self.cpu_log.count_bytes = 2;
        self.cpu_log.byte1 = b;
    }
    fn log_byte2(&mut self, b: u8) {
        self.cpu_log.count_bytes = 3;
        self.cpu_log.byte2 = b;
    }

    fn print_log(&self) {
        let pc: u16 = self.cpu_log.start_address;
        let byte0: u8 = self.cpu_log.byte0;
        let byte1: u8 = self.cpu_log.byte1;
        let byte2: u8 = self.cpu_log.byte2;
        let ins_str: String = stringify_ins_from_log(&self.cpu_log);
        let a: u8 = self.a;
        let x: u8 = self.x;
        let y: u8 = self.y;
        let p: u8 = self.get_status().bits();
        let sp: u8 = self.stkpt;
        let cycles: usize = self.cpu_log.start_cycle;
        let ppucycles = cycles * 3;
        let ppu: usize = ppucycles / 340;
        let ppu2: usize = ppucycles % 340;
        print!("{pc:0>4X}  {byte0:0>2X}");
        if self.cpu_log.count_bytes >= 2 {
            print!(" {byte1:0>2X}");
        } else {
            print!("   ");
        }
        if self.cpu_log.count_bytes >= 3 {
            print!(" {byte2:0>2X}");
        } else {
            print!("   ");
        }
        println!(" {ins_str: <32}A:{a:0>2X} X:{x:0>2X} Y:{y:0>2X} P:{p:0>2X} SP:{sp:0>2X} PPU:{ppu: >3},{ppu2: >3} CYC:{cycles}");
    }

    // Perform one instruction worth of emulation
    pub fn clock(&mut self, address_bus: &mut u16, data_bus: &mut u8, address_rw: &mut bool, phi: bool) -> bool {
        // println!("Clock: {:?}", self.pipeline_status);
        self.cycles = self.cycles.wrapping_add(phi as usize);
        let instruction = lookup::LOOKUP_TABLE[self.opcode as usize];
        
        self.execute(instruction.op(), instruction.addrmode(), address_bus, data_bus, address_rw, phi)
    }

    fn execute(&mut self, opcode: u8, addrmode: u8, address_bus: &mut u16, data_bus: &mut u8, address_rw: &mut bool, phi: bool) -> bool {

        let mut is_executing_stage = matches!(self.pipeline_status, PipelineStatus::Exec0 | PipelineStatus::Exec1 | PipelineStatus::Exec2 | PipelineStatus::Exec3 | PipelineStatus::Exec4 | PipelineStatus::Exec5);
        let mut is_ir_stage = matches!(self.pipeline_status, PipelineStatus::IR);
        let is_addr_stage = !is_executing_stage && !is_ir_stage;

        if is_addr_stage {
            let finished_addressing = self.execute_addrmode(opcode, addrmode, address_bus, data_bus, address_rw, phi);
            if finished_addressing {
                self.pipeline_status = PipelineStatus::Exec0;
                is_executing_stage = !phi;
            } else {
                if !self.page_boundary_crossed && phi {
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
            self.print_log();

            self.opcode = *data_bus;
            self.cpu_log.start_address = self.pc;
            self.pc += 1;

            self.log_byte0(self.opcode);
            
            let instruction = lookup::LOOKUP_TABLE[self.opcode as usize];
            self.cpu_log.opcode = instruction.op();
            self.cpu_log.addrmode = instruction.addrmode();
            self.cpu_log.start_cycle = self.cycles;
            self.pipeline_status = PipelineStatus::Addr0;
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
        let is_mw = matches!(opcode, InsOp::STA | InsOp::STX | InsOp::STY);
        
        let skip_read = is_mw;

        // Page boundary incurrs a +1 cycle cost
        if self.page_boundary_crossed {
            // Probably close-enough to a realistic re-creation
            let hi_byte = hi_byte(self.addr_data).wrapping_add(1);
            set_hi_byte(&mut self.addr_data, hi_byte);
            // Could also probably do something like, might infact get compiler-optimized to this
            // self.addr_data = self.addr_data.wrapping_add(0x0100);
            self.page_boundary_crossed = false;
            return false;
        }
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
            (Addr::IMM | Addr::REL, PipelineStatus::Addr0, true) => { self.fetched = *data_bus; self.addr_data = *data_bus as u16; self.pc += 1; false }
            (Addr::IMM | Addr::REL, PipelineStatus::Addr1, false) => { true }, // Rel offset is stored in both fetched and addr_data
            (Addr::IMM | Addr::REL, PipelineStatus::Addr1, true ) => { true }, // shouldn't be reached but nonetheless

            (Addr::ZP0, PipelineStatus::Addr0, false) => { *address_bus = self.pc; *address_rw = true; false },
            (Addr::ZP0, PipelineStatus::Addr0, true) => { self.fetched = *data_bus; self.addr_data = *data_bus as u16; self.pc += 1; skip_read }
            (Addr::ZP0, PipelineStatus::Addr1, false) => { *address_bus = self.addr_data; *address_rw = true; false }
            (Addr::ZP0, PipelineStatus::Addr1, true ) => { self.fetched = *data_bus; true },
            (Addr::ZP0, PipelineStatus::Addr2, false) => { true },
            (Addr::ZP0, PipelineStatus::Addr2, true ) => { true }, // shouldn't be reached but nonetheless

            // This functionality is common to all instructions with >1 bytes of opcode
            (_, PipelineStatus::Addr0, false) => { *address_bus = self.pc; *address_rw = true; false },
            (_, PipelineStatus::Addr0, true) => { self.fetched = *data_bus; self.addr_data = *data_bus as u16; self.pc += 1; false }

            (Addr::ZPX, PipelineStatus::Addr1, false) => { self.addr_data = u8::wrapping_add(lo_byte(self.addr_data), self.x) as u16; false }, // These have no page-boundary since they will only access page 0
            (Addr::ZPX, PipelineStatus::Addr1, true ) => { !skip_read }, // should dummy-read maybe
            (Addr::ZPX, PipelineStatus::Addr2, false) => { *address_bus = self.addr_data; *address_rw = true; false }
            (Addr::ZPX, PipelineStatus::Addr2, true ) => { self.fetched = *data_bus; false }
            (Addr::ZPX, PipelineStatus::Addr3, false) => { true },
            (Addr::ZPX, PipelineStatus::Addr3, true ) => { true }, // shouldn't be reached but nonetheless

            (Addr::ZPY, PipelineStatus::Addr1, false) => { self.addr_data = u8::wrapping_add(lo_byte(self.addr_data), self.y) as u16; false },
            (Addr::ZPY, PipelineStatus::Addr1, true ) => { !skip_read }, // should dummy-read maybe
            (Addr::ZPY, PipelineStatus::Addr2, false) => { *address_bus = self.addr_data; *address_rw = true; false }
            (Addr::ZPY, PipelineStatus::Addr2, true ) => { self.fetched = *data_bus; false }
            (Addr::ZPY, PipelineStatus::Addr3, false) => { true },
            (Addr::ZPY, PipelineStatus::Addr3, true ) => { true }, // shouldn't be reached but nonetheless

            (Addr::ABS, PipelineStatus::Addr1, false) => { *address_bus = self.pc; *address_rw = true; self.pc += 1; false }
            (Addr::ABS, PipelineStatus::Addr1, true) => { self.log_byte2(*data_bus); set_hi_byte(&mut self.addr_data, *data_bus); skip_read }
            (Addr::ABS, PipelineStatus::Addr2, false) => { 
                if opcode == InstructionOperations::JMP {
                    self.pc = self.addr_data;
                    return true;
                }
                *address_bus = self.addr_data; 
                *address_rw = true; 
                false 
            }
            (Addr::ABS, PipelineStatus::Addr2, true) => { self.fetched = *data_bus; false }
            (Addr::ABS, PipelineStatus::Addr3, false) => {true},
            (Addr::ABS, PipelineStatus::Addr3, true) => {true},

            (Addr::ABX, PipelineStatus::Addr1, false) => {
                let old_lo_byte = lo_byte(self.addr_data);
                let new_lo_byte = old_lo_byte.wrapping_add(offset);
                set_lo_byte(&mut self.addr_data, new_lo_byte);
                self.page_boundary_crossed = new_lo_byte < old_lo_byte;

                *address_bus = self.pc;
                *address_rw = true;
                self.pc += 1;

                false
            },
            (Addr::ABX, PipelineStatus::Addr1, true) => { let b = *data_bus; self.log_byte2(b); set_hi_byte(&mut self.addr_data, b); skip_read }
            (Addr::ABX, PipelineStatus::Addr2, false) => { *address_bus = self.addr_data; *address_rw = true; false }
            (Addr::ABX, PipelineStatus::Addr2, true) => { self.fetched = *data_bus; false }
            (Addr::ABX, PipelineStatus::Addr3, false) => { !is_rwm } // if this is DEC, this cycle is dedicated to fixing page boundary
            (Addr::ABX, PipelineStatus::Addr3, true) => { false }

            (Addr::ABY, PipelineStatus::Addr1, false) => {
                let old_lo_byte = lo_byte(self.addr_data);
                let new_lo_byte = old_lo_byte.wrapping_add(offset);
                set_lo_byte(&mut self.addr_data, new_lo_byte);
                self.page_boundary_crossed = new_lo_byte < old_lo_byte;

                *address_bus = self.pc;
                *address_rw = true;
                self.pc += 1;

                false
            },
            (Addr::ABY, PipelineStatus::Addr1, true) => { let b = *data_bus; self.log_byte2(b); set_hi_byte(&mut self.addr_data, b); skip_read }
            (Addr::ABY, PipelineStatus::Addr2, false) => { *address_bus = self.addr_data; *address_rw = true; false }
            (Addr::ABY, PipelineStatus::Addr2, true) => { self.fetched = *data_bus; false }
            (Addr::ABY, PipelineStatus::Addr3, false) => { true },
            (Addr::ABY, PipelineStatus::Addr3, true) => { true },

            (Addr::IND, PipelineStatus::Addr1, false) => { *address_bus = self.pc; *address_rw = true; self.pc += 1; false },
            (Addr::IND, PipelineStatus::Addr1, true) => { let b = *data_bus; self.log_byte2(b); set_hi_byte(&mut self.addr_data, b); false }
            (Addr::IND, PipelineStatus::Addr2, false) => {
                *address_bus = self.addr_data;
                *address_rw = true;
                // emulate indirect page boundary bug
                let new_lo_byte = lo_byte(self.addr_data).wrapping_add(1);
                set_lo_byte(&mut self.addr_data, new_lo_byte);
                false
            },
            (Addr::IND, PipelineStatus::Addr2, true) => { let b = *data_bus; set_lo_byte(&mut self.pc, b); false }
            (Addr::IND, PipelineStatus::Addr3, false) => { *address_bus = self.addr_data; *address_rw = true; false },
            (Addr::IND, PipelineStatus::Addr3, true) => { let b = *data_bus; set_hi_byte(&mut self.pc, b); false },

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
            (Addr::IDX, PipelineStatus::Addr3, true) => { set_hi_byte(&mut self.addr_data, *data_bus); skip_read },
            (Addr::IDX, PipelineStatus::Addr4, false) => { *address_bus = self.addr_data; *address_rw = true; false },
            (Addr::IDX, PipelineStatus::Addr4, true) => { self.fetched = *data_bus; false },

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
                let old_lo_byte = self.fetched;
                let new_lo_byte = self.fetched.wrapping_add(self.y);
                set_lo_byte(&mut self.addr_data, new_lo_byte);
                self.page_boundary_crossed = new_lo_byte < old_lo_byte; // page-boundary crossed if we wrapped around and 
                                                                        // addition creates a lower value than we started with
                false
            }
            (Addr::IDY, PipelineStatus::Addr2, true) => { set_hi_byte(&mut self.addr_data, *data_bus); skip_read }
            (Addr::IDY, PipelineStatus::Addr3, false) => { *address_bus = self.addr_data; *address_rw = true; false }
            (Addr::IDY, PipelineStatus::Addr3, true) => { self.fetched = *data_bus; false }

            _ => { true }
        }
    }

    /// TODO: Find out why ABS_X or ABS_Y or IND_Y in ASL, DEC, INC, LSR, ROL,
    /// ROR, STA requires an extra cycle compared to ABS, according to [Obelisk](https://www.nesdev.org/obelisk-6502-guide/reference.html)
    /// E.g. zp,x, abs, abs,x and abs,y usually are 4 cycles in most instructions with (ind, x) being 6
    /// and (ind),y being 5. In STA, abs,x and abs,y are 5 while (ind),y is 6.
    /// Edit: looks to be caused by write-instructions always fixing the PCH, even if it doesn't
    /// need it
    fn execute_instruction(&mut self, opcode: u8, addrmode: u8, address_bus: &mut u16, data_bus: &mut u8, address_rw: &mut bool, phi: bool) -> bool {
        use InstructionOperations as InsOp;
        use PipelineStatus as PS;
        match (opcode, self.pipeline_status, phi) {
            (InsOp::NOP, _, _) => {
                true
            },
            // Add with carry
            (InsOp::ADC, PS::Exec0, false) => {
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
            (InsOp::BCC | InsOp::BCS | InsOp::BEQ | InsOp::BMI | InsOp::BNE | InsOp::BPL | InsOp::BVC | InsOp::BVS, PS::Exec1, false) => {
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
            (InsOp::BCC | InsOp::BCS | InsOp::BEQ | InsOp::BMI | InsOp::BNE | InsOp::BPL | InsOp::BVC | InsOp::BVS, PS::Exec1, true) => { false }
            (InsOp::BCC | InsOp::BCS | InsOp::BEQ | InsOp::BMI | InsOp::BNE | InsOp::BPL | InsOp::BVC | InsOp::BVS, PS::Exec2, _) => { true }
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
                self.fetched = hi_byte(self.pc);
                *address_bus = 0x0100 + (self.stkpt as u16);
                *address_rw = false;
                self.stkpt = self.stkpt.wrapping_sub(1);
                false
            },
            (InsOp::BRK, PS::Exec0, true) => { *data_bus = self.fetched; false }
            (InsOp::BRK, PS::Exec1, false) => {
                self.fetched = lo_byte(self.pc);
                *address_bus = 0x0100 + (self.stkpt as u16);
                *address_rw = false;
                self.stkpt = self.stkpt.wrapping_sub(1);
                false
            },
            (InsOp::BRK, PS::Exec1, true) => { *data_bus = self.fetched; false}
            (InsOp::BRK, PS::Exec2, false) => {
                self.fetched = (self.status | Flags6502::B | Flags6502::U).bits();
                *address_bus = 0x0100 + (self.stkpt as u16);
                *address_rw = false;
                self.stkpt = self.stkpt.wrapping_sub(1);
                self.set_flag(Flags6502::I, true);
                false
            },
            (InsOp::BRK, PS::Exec2, true) => { *data_bus = self.fetched; false }
            (InsOp::BRK, PS::Exec3, false) => { *address_bus = 0xFFFE; *address_rw = true; false },
            (InsOp::BRK, PS::Exec3, true) => { set_lo_byte(&mut self.pc, *data_bus); false }
            (InsOp::BRK, PS::Exec4, false) => { *address_bus = 0xFFFF; *address_rw = true; false },
            (InsOp::BRK, PS::Exec4, true) => { set_hi_byte(&mut self.pc, *data_bus); false },
            (InsOp::BRK, PS::Exec5, _) => { true },
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
            (InsOp::EOR, PS::Exec0, false) => {
                self.a = self.a ^ self.fetched;
                self.check_nz_flags(self.a);
                true
            }
            (InsOp::EOR, PS::Exec0, true) => { true } // possibly unneeded
            // Increment memory at address
            (InsOp::INC, PS::Exec0, false) => { self.fetched = self.increment(self.fetched); false },
            (InsOp::INC, PS::Exec0, true) => { false },
            (InsOp::INC, PS::Exec1, false) => { *address_bus = self.addr_data; *address_rw = false; false }
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
            // TODO: Confirm subtraction works
            (InsOp::SBC, PS::Exec0, false) => {
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
            (InsOp::SBC, PS::Exec0, true) => { true }
            // Set carry flag
            (InsOp::SEC, PS::Exec0, _) => { self.set_flag(Flags6502::C, true); true }
            // Set decimal flag
            (InsOp::SED, PS::Exec0, _) => { self.set_flag(Flags6502::D, true); true }
            // Set inturrupt disable
            (InsOp::SEI, PS::Exec0, _) => { self.set_flag(Flags6502::I, true); true }
            // Store Accumulator
            (InsOp::STA, PS::Exec0, false) => { *address_bus = self.addr_data; *address_rw = false; self.fetched = self.a; false }
            (InsOp::STA, PS::Exec0, true) => { *data_bus = self.fetched; false }
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

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
