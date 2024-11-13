use std::io::Write;

use nes_rust::cpu::{instructions::{lookup::LOOKUP_TABLE, opcode_to_str, Instruction}, Cpu, InstructionAddressingModes};


pub struct NESBoard {
    cpu: Cpu,
    cpu_copy: Cpu,
    debug_buffer: Vec<u8>,
    ram: Vec<u8>,
    debug_ram_access: [u8; 7],
    debug_ram_count: usize,
    addr_rw: bool,
    addr_bus: u16,
    data_bus: u8,
}

impl NESBoard {
    pub fn new(cpu: Cpu, ram: Vec<u8>) -> NESBoard {
        let debug_buffer = b"XXXX  XX XX XX  XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX  A:XX X:XX Y:XX P:XX SP:XX PPU:XXX,XXX CYC:XXXXX\n".to_vec();
        let mut cpu_copy = cpu;
        cpu_copy.pc -= 1;
        NESBoard { ram, cpu, cpu_copy, debug_buffer, addr_rw: true, addr_bus: 0, data_bus: 0, debug_ram_access: [0; 7], debug_ram_count: 0 }
    }

    // Emulate one master clok cycle
    pub fn clock(&mut self) {
        self.cpu.clock(&mut self.addr_bus, &mut self.data_bus, &mut self.addr_rw, false);
        if self.addr_rw {
            let addr = self.addr_bus as usize;
            self.data_bus = self.ram[addr];
            self.push_byte();
        }
        if self.cpu.clock(&mut self.addr_bus, &mut self.data_bus, &mut self.addr_rw, true) {
            self.print_log();
            self.cpu_copy = self.cpu; // update the copy
            self.cpu_copy.pc -= 1; // pc increments before we can copy it
        }
        if !self.addr_rw {
            let addr = self.addr_bus as usize;
            self.ram[addr] = self.data_bus;
        }
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn ram(&self) -> &[u8] {
        &self.ram
    }

    pub fn reset(&mut self) {

    }
    pub fn irq(&mut self) {

    }
    pub fn nmi(&mut self) {

    }

    fn push_byte(&mut self) {
        self.debug_ram_access[self.debug_ram_count] = self.data_bus;
        self.debug_ram_count += 1;
    }

    fn write_hex_to_buffer(mut value: u16, buffer: &mut [u8], start: usize, digits: usize) {
        for digit in (0..digits).rev() {
            let a = b'0' + (value % 16) as u8;
            let a = if a > b'9' { a + 7 } else { a };
            value /= 16;
            buffer[start+digit] = a;
        }
    }

    fn write_decimal_to_buffer(mut value: usize, buffer: &mut [u8], start: usize, digits: usize) {
        for digit in (0..digits).rev() {
            let a = if value > 0 { b'0' + (value % 10) as u8 } else { b' ' };
            value /= 10;
            buffer[start+digit] = a;
        }
    }

    fn write_no_padding_decimal_to_buffer(value: usize, buffer: &mut [u8], start: usize, digits: usize) {
        let calculated_length = value.checked_ilog10().unwrap_or(0) as usize + 1;
        let new_end = calculated_length.min(digits);
        Self::write_decimal_to_buffer(value, buffer, start, new_end);
        for i in calculated_length..(digits) {
            buffer[start+i] = b' ';
        }
    }

    fn write_spaces(buffer: &mut [u8], from: usize, to: usize) {
        (from..to).for_each(|i| { buffer[i] = b' '; });
    }

    fn write_assembly_string(instruction: &Instruction, bytes: &[u8], buffer: &mut [u8], mut start: usize, pad_to: usize, pc: u16) {
        let ins_name = opcode_to_str(instruction.op()).as_bytes();
        buffer[start+0] = ins_name[0];
        buffer[start+1] = ins_name[1];
        buffer[start+2] = ins_name[2];
        buffer[start+3] = b' ';

        use InstructionAddressingModes as Addrmode;
        match instruction.addrmode() {
            Addrmode::IMP | Addrmode::ACC => { start += 3; }
            Addrmode::IMM => {
                buffer[start+4] = b'#';
                buffer[start+5] = b'$';
                Self::write_hex_to_buffer(bytes[0] as u16, buffer, start+6, 2);
                start += 8;
            },
            Addrmode::ZP => {
                buffer[start+4] = b'$';
                Self::write_hex_to_buffer(bytes[0] as u16, buffer, start+5, 2);
                start += 7;
            },
            Addrmode::ZPX => {
                buffer[start+4] = b'$';
                Self::write_hex_to_buffer(bytes[0] as u16, buffer, start+5, 2);
                buffer[start+7] = b',';
                buffer[start+8] = b'X';
                start += 9;
            },
            Addrmode::ZPY => {
                buffer[start+4] = b'$';
                Self::write_hex_to_buffer(bytes[0] as u16, buffer, start+5, 2);
                buffer[start+7] = b',';
                buffer[start+8] = b'Y';
                start += 9;
            },
            Addrmode::REL => {
                buffer[start+4] = b'$';
                let b = (bytes[0] as u16) + 2 + pc;
                Self::write_hex_to_buffer(b, buffer, start+5, 4);
                start += 9;
            }
            Addrmode::ABS => {
                buffer[start+4] = b'$';
                Self::write_hex_to_buffer(bytes[0] as u16 | (u16::from(bytes[1]) << 8), buffer, start+5, 4);
                start += 9;
            },
            Addrmode::ABX => {
                buffer[start+4] = b'$';
                Self::write_hex_to_buffer(bytes[0] as u16 | (u16::from(bytes[1]) << 8), buffer, start+5, 4);
                buffer[start+9] = b',';
                buffer[start+10] = b'X';
                start += 11;
            },
            Addrmode::ABY => {
                buffer[start+4] = b'$';
                Self::write_hex_to_buffer(bytes[0] as u16 | (u16::from(bytes[1]) << 8), buffer, start+5, 4);
                buffer[start+9] = b',';
                buffer[start+10] = b'Y';
                start += 11;
            },
            Addrmode::IND => {
                buffer[start+4] = b'(';
                buffer[start+5] = b'$';
                Self::write_hex_to_buffer(bytes[0] as u16 | (u16::from(bytes[1]) << 8), buffer, start+6, 4);
                buffer[start+10] = b')';
                start += 11;
            },
            Addrmode::IDX => {
                buffer[start+4] = b'(';
                buffer[start+5] = b'$';
                Self::write_hex_to_buffer(bytes[0] as u16 | (u16::from(bytes[1]) << 8), buffer, start+6, 4);
                buffer[start+10] = b',';
                buffer[start+11] = b'X';
                buffer[start+12] = b')';
                start += 13;
            }
            Addrmode::IDY => {
                buffer[start+4] = b'(';
                buffer[start+5] = b'$';
                Self::write_hex_to_buffer(bytes[0] as u16 | (u16::from(bytes[1]) << 8), buffer, start+6, 4);
                buffer[start+10] = b')';
                buffer[start+11] = b',';
                buffer[start+12] = b'Y';
                start += 13;
            }

            _ => {}
        }
        Self::write_spaces(buffer, start, pad_to);
    }

    fn print_log(&mut self) {
        let byte1: u8 = self.debug_ram_access[0];
        let byte2: u8 = self.debug_ram_access[1];
        let instruction = &LOOKUP_TABLE[self.cpu_copy.opcode as usize];
        let opcode_count = if self.debug_ram_count == 0 { 0 } else { self.debug_ram_count - 1 };
        // let ins_str: String = stringify_ins_from_log(&self.cpu_copy);
        let cycles: usize = self.cpu_copy.cycles;
        let ppucycles = cycles * 3;
        let ppu1: usize = ppucycles / 340;
        let ppu2: usize = ppucycles % 340;
        // 0         1         2         3         4         5         6         7         8         9
        // 01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234
        //"XXXX  XX XX XX  XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX  A:XX X:XX Y:XX P:XX SP:XX PPU:XXX,XXX CYC:XXXXX"
        let buffer = &mut self.debug_buffer;
        Self::write_hex_to_buffer(self.cpu_copy.pc, buffer, 0, 4);
        Self::write_hex_to_buffer(self.cpu_copy.opcode as u16, buffer, 6, 2);
        if opcode_count == 1 || opcode_count == 2 { Self::write_hex_to_buffer(byte1 as u16, buffer, 9, 2); } else { Self::write_spaces(buffer, 9, 11); }
        if opcode_count == 2 { Self::write_hex_to_buffer(byte2 as u16, buffer, 12, 2); } else { Self::write_spaces(buffer, 12, 14); }
        Self::write_assembly_string(instruction, &self.debug_ram_access, buffer, 16, 46, self.cpu_copy.pc);
        Self::write_hex_to_buffer(self.cpu_copy.a as u16, buffer, 50, 2);
        Self::write_hex_to_buffer(self.cpu_copy.x as u16, buffer, 55, 2);
        Self::write_hex_to_buffer(self.cpu_copy.y as u16, buffer, 60, 2);
        Self::write_hex_to_buffer(self.cpu_copy.get_status().bits() as u16, buffer, 65, 2);
        Self::write_hex_to_buffer(self.cpu_copy.stkpt as u16, buffer, 71, 2);
        Self::write_decimal_to_buffer(ppu1, buffer, 78, 3);
        Self::write_decimal_to_buffer(ppu2, buffer, 82, 3);
        Self::write_no_padding_decimal_to_buffer(cycles, buffer, 90, 5);
        self.debug_ram_count = 0;
        let mut stdout = std::io::stdout().lock();
        let _e = stdout.write_all(&self.debug_buffer);
        let _e = stdout.flush();
    }
}
