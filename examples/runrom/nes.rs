use std::io::Write;

use nes_rust::{cpu::{instructions::{is_unofficial_instruction, lookup::LOOKUP_TABLE, opcode_to_str, Instruction}, Cpu, CpuPinout, InstructionAddressingModes}, ppu::{Ppu, PpuPinout}};


pub struct NESBoard {
    cpu: Cpu,
    cpu_pins: CpuPinout,

    ppu: Ppu,
    ppu_pins: PpuPinout,
    ppu_data_latch: u8,

    debug_buffer: Vec<u8>,
    ram: Vec<u8>,
    cycles: usize,
}

impl NESBoard {
    // Initialize a new circuit
    // Set the inturrupt lines to false so that way the cpu begins startup correctly by detecting a
    // reset inturrupt
    pub fn new(cpu: Cpu, ram: Vec<u8>) -> NESBoard {
        let debug_buffer = b"XXXX  XX XX XX  XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX  A:XX X:XX Y:XX P:XX SP:XX PPU:XXX,XXX CYC:XXXXX\n".to_vec();
        let cycles = 0;
        let cpu_pins = CpuPinout { irq: false, nmi: false, reset: false, phi: false, ready: false, data_bus: 0, address_bus: 0, address_rw: true, sync: false };
        let ppu = Ppu::new();
        let ppu_pins = PpuPinout { nmi: false, cpu_rw: false, cpu_data: 0, ppu_address_data_low: 0, ppu_address_high: 0, ppu_r: false, ppu_w: false, ppu_sync: false, ppu_ale: false };
        NESBoard {
            ram,
            cpu,
            cpu_pins,
            ppu,
            ppu_pins,
            ppu_data_latch: 0,
            debug_buffer,
            cycles
        }
    }

    fn cpu_clock(&mut self, phi: bool) {
        self.cpu_pins.phi = phi;
        let cycle_occured = self.cpu.clock(&mut self.cpu_pins);
        let addr = self.cpu_pins.address_bus as usize;
        if self.cpu_pins.address_rw {
            if self.cpu_pins.sync {
                self.print_log();
            }
            self.cpu_pins.data_bus = self.ram[addr];
            // self.push_byte();
        } else {
            self.ram[addr] = self.cpu_pins.data_bus;
        }
        if cycle_occured {
            self.cycles = self.cycles.wrapping_add(1);
        }
    }

    fn ppu_clock(&mut self) {
        self.ppu.clock(&mut self.ppu_pins);
        self.cpu_pins.nmi = self.ppu_pins.nmi;
        if self.ppu_pins.ppu_ale {
            self.ppu_data_latch = self.ppu_pins.ppu_address_data_low;
        }
    }

    // Emulate one master clok cycle
    pub fn clock(&mut self, _ready: bool) {

        self.ppu_clock();
        self.ppu_clock();

        self.cpu_clock(false);

        self.ppu_clock();

        self.cpu_clock(true);

        // Reset inturrupt requests
        self.cpu_pins.reset = true;
        self.cpu_pins.irq = true;
        self.cpu_pins.nmi = true;
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn ram(&self) -> &[u8] {
        &self.ram
    }

    pub fn video_memory(&self) -> &[u8] {
        self.ppu.video_data()
    }

    pub fn reset(&mut self) {
        self.cpu_pins.reset = false;
    }
    pub fn irq(&mut self) {
        self.cpu_pins.irq = false;
    }
    pub fn nmi(&mut self) {
        self.cpu_pins.nmi = false;
    }

    // fn push_byte(&mut self) {
        // self.debug_ram_access[self.debug_ram_count] = self.data_bus;
        // self.debug_ram_count += 1;
    // }

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
        buffer[start  ] = ins_name[0];
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
        let pc = self.cpu.pc;
        let opcode    = self.ram[pc as usize    ];
        let byte1: u8 = self.ram[pc as usize + 1];
        let byte2: u8 = self.ram[pc as usize + 2];
        let a = self.cpu.a;
        let x = self.cpu.x;
        let y = self.cpu.y;
        let p = self.cpu.get_status().bits();
        let s = self.cpu.stkpt;

        let instruction = &LOOKUP_TABLE[opcode as usize];
        let debug_ram_count = 0;
        let opcode_count = if debug_ram_count == 0 { 0 } else { debug_ram_count - 1 };
        // let ins_str: String = stringify_ins_from_log(&self.cpu_copy);
        let cycles: usize = self.cycles + 1; // add 1 since begin counting at 0
        let ppucycles = cycles * 3;
        let ppu1: usize = ppucycles / 341;
        let ppu2: usize = ppucycles % 341;
        // 0         1         2         3         4         5         6         7         8         9
        // 01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234
        //"XXXX  XX XX XX  XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX  A:XX X:XX Y:XX P:XX SP:XX PPU:XXX,XXX CYC:XXXXX"
        let buffer = &mut self.debug_buffer;
        Self::write_hex_to_buffer(pc, buffer, 0, 4);
        Self::write_hex_to_buffer(opcode as u16, buffer, 6, 2);
        if opcode_count == 1 || opcode_count == 2 { Self::write_hex_to_buffer(byte1 as u16, buffer, 9, 2); } else { Self::write_spaces(buffer, 9, 11); }
        if opcode_count == 2 { Self::write_hex_to_buffer(byte2 as u16, buffer, 12, 2); } else { Self::write_spaces(buffer, 12, 14); }
        buffer[15] = if is_unofficial_instruction(instruction, opcode) { b'*' } else { b' ' };
        Self::write_assembly_string(instruction, &self.ram[(pc as usize +1)..], buffer, 16, 46, pc);
        Self::write_hex_to_buffer(a as u16, buffer, 50, 2);
        Self::write_hex_to_buffer(x as u16, buffer, 55, 2);
        Self::write_hex_to_buffer(y as u16, buffer, 60, 2);
        Self::write_hex_to_buffer(p as u16, buffer, 65, 2);
        Self::write_hex_to_buffer(s as u16, buffer, 71, 2);
        Self::write_decimal_to_buffer(ppu1, buffer, 78, 3);
        Self::write_decimal_to_buffer(ppu2, buffer, 82, 3);
        Self::write_no_padding_decimal_to_buffer(cycles, buffer, 90, 5);
        let mut stdout = std::io::stdout().lock();
        let _e = stdout.write_all(&self.debug_buffer);
        let _e = stdout.flush();
    }
}
