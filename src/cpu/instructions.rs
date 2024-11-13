use super::CpuLog;

#[derive(Clone, Copy)]
pub struct Instruction {
    operate: u8,
    addrmode: u8,
}

impl Instruction {
    pub const fn new(
                operate: u8,
                addrmode: u8,
                ) -> Self {
        Self { operate, addrmode }
    }

    pub fn op(&self) -> u8 {
        self.operate
    }

    pub fn addrmode(&self) -> u8 {
        self.addrmode
    }
}


pub mod lookup { 
        use crate::cpu::InstructionAddressingModes as A;
        use crate::cpu::InstructionOperations as O;

        use super::Instruction;
        // use Instruction::new as I;
    const fn I(o: u8, a: u8) -> Instruction {
        Instruction::new(o, a)
    }
    pub const LOOKUP_TABLE: [super::Instruction; 256] = [
        I(O::BRK, A::IMP),I(O::ORA, A::IDX),I(O::JAM, A::IMP),I(O::SLO, A::IDX),
        I(O::NOP, A::ZP0),I(O::ORA, A::ZP0),I(O::ASL, A::ZP0),I(O::SLO, A::ZP0),
        I(O::PHP, A::IMP),I(O::ORA, A::IMM),I(O::ASL, A::ACC),I(O::ANC, A::IMM),
        I(O::NOP, A::ABS),I(O::ORA, A::ABS),I(O::ASL, A::ABS),I(O::SLO, A::ABS),
        I(O::BPL, A::REL),I(O::ORA, A::IDY),I(O::JAM, A::REL),I(O::SLO, A::IDY),
        I(O::NOP, A::ZPX),I(O::ORA, A::ZPX),I(O::ASL, A::ZPX),I(O::SLO, A::ZPX),
        I(O::CLC, A::IMP),I(O::ORA, A::ABY),I(O::NOP, A::IMP),I(O::SLO, A::ABY),
        I(O::NOP, A::ABX),I(O::ORA, A::ABX),I(O::ASL, A::ABX),I(O::SLO, A::ABX),

        I(O::JSR, A::ABS),I(O::AND, A::IDX),I(O::JAM, A::IMP),I(O::RLA, A::IDX),
        I(O::BIT, A::ZP0),I(O::AND, A::ZP0),I(O::ROL, A::ZP0),I(O::RLA, A::ZP0),
        I(O::PLP, A::IMP),I(O::AND, A::IMM),I(O::ROL, A::ACC),I(O::ANC, A::IMM),
        I(O::BIT, A::ABS),I(O::AND, A::ABS),I(O::ROL, A::ABS),I(O::RLA, A::ABS),
        I(O::BMI, A::REL),I(O::AND, A::IDY),I(O::JAM, A::REL),I(O::RLA, A::IDY),
        I(O::NOP, A::ZPX),I(O::AND, A::ZPX),I(O::ROL, A::ZPX),I(O::RLA, A::ZPX),
        I(O::SEC, A::IMP),I(O::AND, A::ABY),I(O::NOP, A::IMP),I(O::RLA, A::ABY),
        I(O::NOP, A::ABX),I(O::AND, A::ABX),I(O::ROL, A::ABX),I(O::RLA, A::ABX),

        I(O::RTI, A::IMP),I(O::EOR, A::IDX),I(O::JAM, A::IMP),I(O::SRE, A::IDX),
        I(O::NOP, A::ZP0),I(O::EOR, A::ZP0),I(O::LSR, A::ZP0),I(O::SRE, A::ZP0),
        I(O::PHA, A::IMP),I(O::EOR, A::IMM),I(O::LSR, A::ACC),I(O::ALR, A::IMM),
        I(O::JMP, A::ABS),I(O::EOR, A::ABS),I(O::LSR, A::ABS),I(O::SRE, A::ABS),
        I(O::BVC, A::REL),I(O::EOR, A::IDY),I(O::JAM, A::REL),I(O::SRE, A::IDY),
        I(O::NOP, A::ZPX),I(O::EOR, A::ZPX),I(O::LSR, A::ZPX),I(O::SRE, A::ZPX),
        I(O::CLI, A::IMP),I(O::EOR, A::ABY),I(O::NOP, A::IMP),I(O::SRE, A::ABY),
        I(O::NOP, A::ABX),I(O::EOR, A::ABX),I(O::LSR, A::ABX),I(O::SRE, A::ABX),

        I(O::RTS, A::IMP),I(O::ADC, A::IDX),I(O::JAM, A::IMP),I(O::RRA, A::IDX),
        I(O::NOP, A::ZP0),I(O::ADC, A::ZP0),I(O::ROR, A::ZP0),I(O::RRA, A::ZP0),
        I(O::PLA, A::IMP),I(O::ADC, A::IMM),I(O::ROR, A::ACC),I(O::ARR, A::IMM),
        I(O::JMP, A::IND),I(O::ADC, A::ABS),I(O::ROR, A::ABS),I(O::RRA, A::ABS),
        I(O::BVS, A::REL),I(O::ADC, A::IDY),I(O::JAM, A::REL),I(O::RRA, A::IDY),
        I(O::NOP, A::ZPX),I(O::ADC, A::ZPX),I(O::ROR, A::ZPX),I(O::RRA, A::ZPX),
        I(O::SEI, A::IMP),I(O::ADC, A::ABY),I(O::NOP, A::IMP),I(O::RRA, A::ABY),
        I(O::NOP, A::ABX),I(O::ADC, A::ABX),I(O::ROR, A::ABX),I(O::RRA, A::ABX),

        I(O::NOP, A::IMM),I(O::STA, A::IDX),I(O::NOP, A::IMM),I(O::SAX, A::IDX),
        I(O::STY, A::ZP0),I(O::STA, A::ZP0),I(O::STX, A::ZP0),I(O::SAX, A::ZP0),
        I(O::DEY, A::IMP),I(O::NOP, A::IMM),I(O::TXA, A::IMP),I(O::ANE, A::IMM), // STA (imm)
        I(O::STY, A::ABS),I(O::STA, A::ABS),I(O::STX, A::ABS),I(O::SAX, A::ABS),
        I(O::BCC, A::REL),I(O::STA, A::IDY),I(O::JAM, A::REL),I(O::SHA, A::IDY),
        I(O::STY, A::ZPX),I(O::STA, A::ZPX),I(O::STX, A::ZPY),I(O::SAX, A::ZPY),
        I(O::TYA, A::IMP),I(O::STA, A::ABY),I(O::TXS, A::IMP),I(O::TAS, A::ABY),
        I(O::SHY, A::ABX),I(O::STA, A::ABX),I(O::SHX, A::ABY),I(O::SHA, A::ABY),

        I(O::LDY, A::IMM),I(O::LDA, A::IDX),I(O::LDX, A::IMM),I(O::LAX, A::IDX),
        I(O::LDY, A::ZP0),I(O::LDA, A::ZP0),I(O::LDX, A::ZP0),I(O::LAX, A::ZP0),
        I(O::TAY, A::IMP),I(O::LDA, A::IMM),I(O::TAX, A::IMP),I(O::LAX, A::IMM),
        I(O::LDY, A::ABS),I(O::LDA, A::ABS),I(O::LDX, A::ABS),I(O::LAX, A::ABS),
        I(O::BCS, A::REL),I(O::LDA, A::IDY),I(O::JAM, A::REL),I(O::LAX, A::IDY),
        I(O::LDY, A::ZPX),I(O::LDA, A::ZPX),I(O::LDX, A::ZPY),I(O::LAX, A::ZPY),
        I(O::CLV, A::IMP),I(O::LDA, A::ABY),I(O::TSX, A::IMP),I(O::LAS, A::ABY),
        I(O::LDY, A::ABX),I(O::LDA, A::ABX),I(O::LDX, A::ABY),I(O::LAX, A::ABY),

        I(O::CPY, A::IMM),I(O::CMP, A::IDX),I(O::NOP, A::IMM),I(O::DCP, A::IDX),
        I(O::CPY, A::ZP0),I(O::CMP, A::ZP0),I(O::DEC, A::ZP0),I(O::DCP, A::ZP0),
        I(O::INY, A::IMP),I(O::CMP, A::IMM),I(O::DEX, A::IMP),I(O::SBX, A::IMM),
        I(O::CPY, A::ABS),I(O::CMP, A::ABS),I(O::DEC, A::ABS),I(O::DCP, A::ABS),
        I(O::BNE, A::REL),I(O::CMP, A::IDY),I(O::JAM, A::REL),I(O::DCP, A::IDY),
        I(O::NOP, A::ZPX),I(O::CMP, A::ZPX),I(O::DEC, A::ZPX),I(O::DCP, A::ZPX),
        I(O::CLD, A::IMP),I(O::CMP, A::ABY),I(O::NOP, A::IMP),I(O::DCP, A::ABY),
        I(O::NOP, A::ABX),I(O::CMP, A::ABX),I(O::DEC, A::ABX),I(O::DCP, A::ABX),

        I(O::CPX, A::IMM),I(O::SBC, A::IDX),I(O::NOP, A::IMM),I(O::ISC, A::IDX),
        I(O::CPX, A::ZP0),I(O::SBC, A::ZP0),I(O::INC, A::ZP0),I(O::ISC, A::ZP0),
        I(O::INX, A::IMP),I(O::SBC, A::IMM),I(O::NOP, A::IMP),I(O::SBC, A::IMM),
        I(O::CPX, A::ABS),I(O::SBC, A::ABS),I(O::INC, A::ABS),I(O::ISC, A::ABS),
        I(O::BEQ, A::REL),I(O::SBC, A::IDY),I(O::JAM, A::REL),I(O::ISC, A::IDY),
        I(O::NOP, A::ZPX),I(O::SBC, A::ZPX),I(O::INC, A::ZPX),I(O::ISC, A::ZPX),
        I(O::SED, A::IMP),I(O::SBC, A::ABY),I(O::NOP, A::IMP),I(O::ISC, A::ABY),
        I(O::NOP, A::ABX),I(O::SBC, A::ABX),I(O::INC, A::ABX),I(O::ISC, A::ABX),
    ];
}

#[allow(non_snake_case, dead_code)]
pub mod Instructions {
    // LDA; Affects Z,N; "Loads a byte of memory into accumulator"
    pub const LDA_IMM: u8   = 0xA9;     // 2
    pub const LDA_ZP: u8    = 0xA5;     // 3
    pub const LDA_ZP_X: u8  = 0xB5;     // 4
    pub const LDA_ABS: u8   = 0xAD;     // 4
    pub const LDA_ABS_X: u8 = 0xBD;     // 4 (+PB)
    pub const LDA_ABS_Y: u8 = 0xB9;     // 4 (+PB)
    pub const LDA_IND_X: u8 = 0xA1;     // 5
    pub const LDA_IND_Y: u8 = 0xB1;     // 6 (+PB)

    // LDX; Affects Z,N; "Loads a byte of memory into X register"
    pub const LDX_IMM: u8   = 0xA2;     // 2
    pub const LDX_ZP: u8    = 0xA6;     // 3
    pub const LDX_ZP_Y: u8  = 0xB6;     // 4
    pub const LDX_ABS: u8   = 0xAE;     // 4
    pub const LDX_ABS_Y: u8 = 0xBE;     // 4 (+PB)

    // LDY; Affects Z,N; "Loads a byte of memory into Y register"
    pub const LDY_IMM: u8   = 0xA0;     // 2
    pub const LDY_ZP: u8    = 0xA4;     // 3
    pub const LDY_ZP_Y: u8  = 0xB4;     // 4
    pub const LDY_ABS: u8   = 0xAC;     // 4
    pub const LDY_ABS_Y: u8 = 0xBC;     // 4 (+PB)

    // LSR (Logical Shift Right); Affects C,Z,N; "Shifts the bits in A or M to the right by one.
    // Bit 0 becomes the carry flag and 7 becomes zero"
    pub const LSR_ACC: u8   = 0x4A;     // 2
    pub const LSR_ZP: u8    = 0x46;     // 5
    pub const LSR_ZP_X: u8  = 0x56;     // 6
    pub const LSR_ABS: u8   = 0x4E;     // 6
    pub const LSR_ABS_X: u8 = 0x5E;     // 7

    // NOP (No Operation); "Causes no changes to the processor, increments the program counter by 1"
    pub const NOP: u8       = 0xEA;     // 2

    // ADC (Add with Carry); Affects Z,C,N; "Adds the contents of a memory location to the
    // accumulator with the carry bit. An overflow will re-set the carry bit to allow for multiple
    // byte additions"
    pub const ADC_IMM: u8   = 0x69;     // 2
    pub const ADC_ZP: u8    = 0x65;     // 3
    pub const ADC_ZP_X: u8  = 0x75;     // 4
    pub const ADC_ABS: u8   = 0x6D;     // 4
    pub const ADC_ABS_X: u8 = 0x7D;     // 4 (+PB)
    pub const ADC_ABS_Y: u8 = 0x79;     // 4 (+PB)
    pub const ADC_IND_X: u8 = 0x61;     // 6
    pub const ADC_IND_Y: u8 = 0x71;     // 5 (+PB)

    // AND (Logical AND); Affects Z,N; "A logical and is performed on the accumulators contents
    // using a byte of memory"
    pub const AND_IMM: u8   = 0x29;     // 2
    pub const AND_ZP: u8    = 0x25;     // 3
    pub const AND_ZP_X: u8  = 0x35;     // 4
    pub const AND_ABS: u8   = 0x2D;     // 4
    pub const AND_ABS_X: u8 = 0x3D;     // 4 (+PB)
    pub const AND_ABS_Y: u8 = 0x39;     // 4 (+PB)
    pub const AND_IND_X: u8 = 0x21;     // 6
    pub const AND_IND_Y: u8 = 0x31;     // 5 (+PB)

    // ASL (Arithmetic Shift Left); Affects Z,C,N; "Shifts all bits of accumulator or memory one
    // bit left. Bit 0 becomes zero and the carry is set to 7"
    pub const ASL_ACC: u8   = 0x0A;     // 2
    pub const ASL_ZP: u8    = 0x06;     // 5
    pub const ASL_ZP_X: u8  = 0x16;     // 6
    pub const ASL_ABS: u8   = 0x0E;     // 6
    pub const ASL_ABS_X: u8 = 0x1E;     // 7

    // BCC (Branch if Carry Clear); "If the carry flag is clear, add relative displacement to PC"
    pub const BCC_REL: u8   = 0x90;     // 2 (+1 Branch Success) (+2 New Page)

    // BCS (Branch if Carry Set); "If the carry flag is set, add relative displacement to PC"
    pub const BCS_REL: u8   = 0xB0;     // 2 (+1 Branch Success) (+2 New Page)

    // BEQ (Branch if Equal); "If the zero flag is set, add relative displacement to PC"
    pub const BEQ_REL: u8   = 0xF0;     // 2 (+1 Branch Success) (+2 New Page)

    // BIT (BIt Test); Affects Z,V,N; "Test if bits are set in target memory location. Pattern in A is & with
    // value in memory"
    pub const BIT_ZP: u8    = 0x24;     // 3
    pub const BIT_ABS: u8   = 0x2C;     // 4

    // BMI (Branch if MInus); "If the negative flag (N) is set, add relative displacement to PC"
    pub const BMI_REL: u8   = 0x30;     // 2 (+1 Branch Success) (+2 New Page)

    // BNE (Branch if Not Equal); "If the zero flag (Z) is set, add relative displacement to PC"
    pub const BNE_REL: u8   = 0xD0;     // 2 (+1 Branch Success) (+2 New Page)

    // BPL (Branch if Positive); "If the negative flag (N) is clear, add relative dispalcement to PC"
    pub const BPL_REL: u8   = 0x10;     // 2 (+1 Branch SUccess) (+2 New Page)

    // BRK (Force Interrupt); Affects B; "Forces the generation of an interrupt request. PC and PS are pushed
    // onto stack. PC is set to FFFE. Break flag set to one"
    pub const BRK_IMP: u8   = 0x00;     // 7

    // BVC (Break is overflow clear); "If the overflow flag (V) is clear; add relative displacement to
    // PC"
    pub const BVC_REL: u8   = 0x50;     // 2 (+1 Branch Success) (+2 New Page)

    // BVS (Break is overflow Set); "If the overflow flag (V) is set; add relative displacement to
    // PC"
    pub const BVS_REL: u8   = 0x70;     // 2 (+1 Branch Success) (+2 New Page)

    // CLC (Clear Carry Flag); Affects C; "Set the carry flag (C) to Zero"
    pub const CLC_IMP: u8   = 0x18;     // 2

    // CDL (Clear Decimal Flag); Affects D; "Set decimal flag (D) to Zero. State of D is uncertain,
    // always set it before +/- operations"
    pub const CLD_IMP: u8   = 0xD8;     // 2

    // CLI (Clear Inturrupt Disable); Affects I; "Set inturrupt disable flag (I) to Zero"
    pub const CLI_IMP: u8   = 0x58;     // 2

    // CVL (Clear Overflow Flag); Affects V; "Set overflow flag (V) to Zero"
    pub const CLV_IMP: u8   = 0xB8;     // 2

    // CMP (Compare); Affects Z,C,N; "Compares contents of Accumulator to a memory location. Sets
    // the zero and carry flags"
    pub const CMP_IMM: u8   = 0xC9;     // 2
    pub const CMP_ZP: u8    = 0xC5;     // 3
    pub const CMP_ZP_X: u8  = 0xD5;     // 4
    pub const CMP_ABS: u8   = 0xCD;     // 4
    pub const CMP_ABS_X: u8 = 0xDD;     // 4 (+PB)
    pub const CMP_ABS_Y: u8 = 0xD9;     // 4 (+PB)
    pub const CMP_IND_X: u8 = 0xC1;     // 6
    pub const CMP_IND_Y: u8 = 0xD1;     // 5 (+PB)

    // CPX (Compare X); Affects Z,C,N; "Compares the contents of X register with a memory location"
    pub const CPX_IMM: u8   = 0xE0;     // 2
    pub const CPX_ZP: u8    = 0xE4;     // 3
    pub const CPZ_ABS: u8   = 0xEC;     // 4

    // CPY (Compare Y); Affects Z,C,N; "Compares the contents of Y register with a memory location"
    pub const CPY_IMM: u8   = 0xC0;     // 2
    pub const CPY_ZP: u8    = 0xC4;     // 3
    pub const CPY_ABS: u8   = 0xCC;     // 4

    // DEC (Decrement Memory); Affects Z,N; "Subtracts 1 from a value at a memory location"
    pub const DEC_ZP: u8    = 0xC6;     // 5
    pub const DEC_ZP_X: u8  = 0xD6;     // 6
    pub const DEC_ABS: u8   = 0xCE;     // 6
    pub const DEX_ABS_X: u8 = 0xDE;     // 7

    // DEX (Decrement X); Affects Z,N; "Subtracts 1 from the X register"
    pub const DEX_IMP: u8   = 0xCA;     // 2

    // DEY (Decrement Y); Affects Z,N; "Subtracts 1 from the Y register"
    pub const DEY_IMP: u8   = 0x88;     // 2

    // EOR (Exclusive OR | XOR); Affects Z,N; "Performs the exlcusive OR operation on the contents
    // of the accumulator using the contents of a byte of memory"
    pub const EOR_IMM: u8   = 0x49;     // 2
    pub const EOR_ZP: u8    = 0x45;     // 3
    pub const EOR_ZP_X: u8  = 0x55;     // 4
    pub const EOR_ABS: u8   = 0x4D;     // 4
    pub const EOR_ABS_X: u8 = 0x5D;     // 4 (+PB)
    pub const EOR_ABS_Y: u8 = 0x59;     // 4 (+PB)
    pub const EOR_IND_X: u8 = 0x41;     // 6
    pub const EOR_IND_Y: u8 = 0x51;     // 5 (+PB)

    // INC (Increment memory); Affects Z,N; "Adds 1 to the value at a location in memory"
    pub const INC_ZP: u8    = 0xE6;     // 5
    pub const INC_ZP_X: u8  = 0xF6;     // 6
    pub const INC_ABS: u8   = 0xEE;     // 6
    pub const INC_ABS_X: u8 = 0xFE;     // 7

    // INX (Increment X); Affects Z,N; "Adds 1 to the X register"
    pub const INX_IMP: u8   = 0xE8;     // 2

    // INY (Increment Y); Affects Z,N; "Adds 1 to the Y reigster"
    pub const INY_IMP: u8   = 0xC8;     // 2

    // JMP (Jump); "Sets the program counter to to the address specified by the operand"
    pub const JMP_ABS: u8   = 0x4C;     // 3
    pub const JMP_IND: u8   = 0x6C;     // 5

    // JSR (Jump to Subroutine); "Pushes the address-1 (return point) onto the stack. Sets the
    // PC to the target memory address"
    pub const JSR_ABS: u8   = 0x20;     // 6

    // ORA (Logical Inclusive OR); Affects Z,N; "An inclusive OR is performed in the contents of A"
    pub const ORA_IMM: u8   = 0x09;     // 
    pub const ORA_ZP: u8    = 0x05;
    pub const ORA_ZP_X: u8  = 0x15;
    pub const ORA_ABS: u8   = 0x0D;
    pub const ORA_ABS_X: u8 = 0x1D;
    pub const ORA_ABS_Y: u8 = 0x19;
    pub const ORA_IND_X: u8 = 0x01;
    pub const ORA_IND_Y: u8 = 0x11;

    // PHA (Push Accumulator); "Pushes a copy of A onto the stack"
    pub const PHA_IMP: u8   = 0x48;     // 3

    // PHP (Push Processor Status); "Pushes a copy of the status flags on to the stack"
    pub const PHP_IMP: u8   = 0x08;     // 3

    // PLA (Pull Accumulator); Affects Z,N; "A is assigned to a pulled 8-bit value from stack"
    pub const PLA_IMP: u8   = 0x68;     // 4

    // PLP (Pull Processor Status); Affects All; "Flags are set to an 8-bit value pulled from stack"
    pub const PLP_IMP: u8   = 0x28;     // 4

    // ROL (Rotate Left); Affects C,Z,N; "Move bits in A or memory location one place to the
    // left. Bit 0 is assigned the carry flag and the carry flag is assigned the original value of
    // Bit 7"
    pub const ROL_ACC: u8   = 0x2A;     // 2
    pub const ROL_ZP: u8    = 0x26;     // 5
    pub const ROL_ZP_X: u8  = 0x36;     // 6
    pub const ROL_ABS: u8   = 0x2E;     // 6
    pub const ROL_ABS_X: u8 = 0x3E;     // 7

    // ROR (Rotate Right); Affects C,Z,N; "Move bits in A or memory location one place to the
    // right. Bit 7 is assigned the carry flag and the carry flag is assigned the original value of
    // Bit 0"
    pub const ROR_ACC: u8   = 0x6A;     // 2
    pub const ROR_ZP: u8    = 0x66;     // 5
    pub const ROR_ZP_X: u8  = 0x76;     // 6
    pub const ROR_ABS: u8   = 0x6E;     // 6
    pub const ROR_ABS_X: u8 = 0x7E;     // 7

    // RTI (Return from Inturrupt); "Pulls processor flags from stack. Pulls PC from stack"
    pub const RTI_IMP: u8   = 0x40;     // 6

    // RTS (Return from Subroutine); "Return to calling routine by pulling PC-1 from stack"
    pub const RTS_IMP: u8   = 0x60;     // 6

    // SBC (Subtract with Carry); Affects Z,C,N,V; "Subtracts the contents of a memory location to
    // the accumulator together with the not of the carry bit (A = A-M-(1-C))"
    pub const SBC_IMM: u8   = 0xE9;
    pub const SBC_ZP: u8    = 0xE5;
    pub const SBC_ZP_X: u8  = 0xF5;
    pub const SBC_ABS: u8   = 0xED;
    pub const SBC_ABS_X: u8 = 0xFD;
    pub const SBC_ABS_Y: u8 = 0xF9;
    pub const SBC_IND_X: u8 = 0xE1;
    pub const SBC_IND_Y: u8 = 0xF1;

    // SEC (Set Carry Flag); Affects C; "Sets carry flag (C) to One"
    pub const SEC_IMP: u8   = 0x38;     // 2

    // SED (Set Decimal Flag); Affects D; "Sets decimal flag (D) to One"
    pub const SED_IMP: u8   = 0xF8;     // 2

    // SEI (Set Inturript Disable); Affects I; "Sets inturrupt disable (I) flag to One"
    pub const SEI_IMP: u8   = 0x78;

    // STA (Store Accumulator); "Stores contents of Accumulator into memory"
    pub const STA_ZP: u8    = 0x85;
    pub const STA_ZP_X: u8  = 0x95;
    pub const STA_ABS: u8   = 0x8D;
    pub const STA_ABS_X: u8 = 0x9D;
    pub const STA_ABS_Y: u8 = 0x99;
    pub const STA_IND_X: u8 = 0x81;
    pub const STA_IND_Y: u8 = 0x91;

    // STX (Store X Register); "Stores contents of X into memory"
    pub const STX_ZP: u8    = 0x86;     // 3
    pub const STX_ZP_Y: u8  = 0x96;     // 4
    pub const STX_ABS: u8   = 0x8E;     // 4

    // STY (Store Y Register); "Stores contents of Y into memory"
    pub const STY_ZP: u8    = 0x84;     // 3
    pub const STY_ZP_X: u8  = 0x94;     // 4
    pub const STY_ABS: u8   = 0x8C;     // 4

    // TAX (Transfer A to X); Affects Z,N
    pub const TAX_IMP: u8   = 0xAA;     // 2

    // TAY (Transfer A to Y); Affects Z,N
    pub const TAY_IMP: u8   = 0xA8;     // 2

    // TSX (Transfer Stack Pointer to X); Affects Z,N
    pub const TSX_IMP: u8   = 0xBA;     // 2

    // TXA (Transfer X to A); Affects Z,N
    pub const TXA_IMP: u8   = 0x8A;     // 2

    // TXS (Transfer X to Stack Pointer)
    pub const TXS_IMP: u8   = 0x9A;     // 2
    
    // TYA (Transfer Y to A); Affects Z;N
    pub const TYA_IMP: u8   = 0x98;     // 2
}

pub fn stringify_ins_from_log(log: &CpuLog) -> String {
    let ins_name = opcode_to_str(log.opcode);
    let addr_name = "$";
    let has_read_value = false;
    let read_value = "0";

    if has_read_value {
        format!("{ins_name} {addr_name} = {read_value}")
    } else {
        format!("{ins_name} {addr_name}")
    }
}

pub fn opcode_to_str(opcode: u8) -> &'static str {
    match opcode {
        0x00 => { "NOP" },
        0x01 => { "LDA" },
        0x02 => { "LDX" },
        0x03 => { "LDY" },
        0x04 => { "ADC" },
        0x05 => { "AND" },
        0x06 => { "ASL" },
        0x07 => { "BCC" },
        0x08 => { "BCS" },
        0x09 => { "BEQ" },
        0x0A => { "BIT" },
        0x0B => { "BMI" },
        0x0C => { "BNE" },
        0x0D => { "BPL" },
        0x0E => { "BRK" },
        0x0F => { "BVC" },
        0x10 => { "BVS" },
        0x11 => { "CLC" },
        0x12 => { "CLD" },
        0x13 => { "CLI" },
        0x14 => { "CLV" },
        0x15 => { "CMP" },
        0x16 => { "CPX" },
        0x17 => { "CPY" },
        0x18 => { "DEC" },
        0x19 => { "DEX" },
        0x1A => { "DEY" },
        0x1B => { "EOR" },
        0x1C => { "INC" },
        0x1D => { "INX" },
        0x1E => { "INY" },
        0x1F => { "JMP" },
        0x20 => { "JSR" },
        0x21 => { "LSR" },
        0x22 => { "ORA" },
        0x23 => { "PHA" },
        0x24 => { "PHP" },
        0x25 => { "PLA" },
        0x26 => { "PLP" },
        0x27 => { "ROL" },
        0x28 => { "ROR" },
        0x29 => { "RTI" },
        0x2A => { "RTS" },
        0x2B => { "SBC" },
        0x2C => { "SEC" },
        0x2D => { "SED" },
        0x2E => { "SEI" },
        0x2F => { "STA" },
        0x30 => { "STX" },
        0x31 => { "STY" },
        0x32 => { "TAX" },
        0x33 => { "TAY" },
        0x34 => { "TSX" },
        0x35 => { "TXA" },
        0x36 => { "TXS" },
        0x37 => { "TYA" },
        _ => { "XXX" },
    }
}
