use std::io::Write;

use nes_rust::cpu::Cpu;


pub struct NESBoard {
    cpu: Cpu,
    cpu_copy: Cpu,
    debug_buffer: Vec<u8>,
    ram: Vec<u8>,
    addr_rw: bool,
    addr_bus: u16,
    data_bus: u8,
}

impl NESBoard {
    pub fn new(cpu: Cpu, ram: Vec<u8>) -> NESBoard {
        let debug_buffer = b"XXXX  XX XX XX  XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX  A:XX X:XX Y:XX P:XX SP:XX PPU:XXX,XXX CYC:XXXXX\n".to_vec();
        let mut cpu_copy = cpu;
        cpu_copy.pc -= 1;
        NESBoard { ram, cpu, cpu_copy, debug_buffer, addr_rw: true, addr_bus: 0, data_bus: 0 }
    }

    // Emulate one master clok cycle
    pub fn clock(&mut self) {
        self.cpu.clock(&mut self.addr_bus, &mut self.data_bus, &mut self.addr_rw, false);
        if self.addr_rw {
            let addr = self.addr_bus as usize;
            self.data_bus = self.ram[addr];
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

    fn print_log(&mut self) {
        let byte1: u8 = 0;
        let byte2: u8 = 0;
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
        Self::write_hex_to_buffer(byte1 as u16, buffer, 9, 2);
        Self::write_hex_to_buffer(byte2 as u16, buffer, 12, 2);
        Self::write_hex_to_buffer(self.cpu_copy.a as u16, buffer, 50, 2);
        Self::write_hex_to_buffer(self.cpu_copy.x as u16, buffer, 55, 2);
        Self::write_hex_to_buffer(self.cpu_copy.y as u16, buffer, 60, 2);
        Self::write_hex_to_buffer(self.cpu_copy.get_status().bits() as u16, buffer, 65, 2);
        Self::write_hex_to_buffer(self.cpu_copy.stkpt as u16, buffer, 71, 2);
        Self::write_decimal_to_buffer(ppu1, buffer, 78, 3);
        Self::write_decimal_to_buffer(ppu2, buffer, 82, 3);
        Self::write_decimal_to_buffer(cycles, buffer, 90, 5);
        let mut stdout = std::io::stdout().lock();
        let _e = stdout.write_all(&self.debug_buffer);
        let _e = stdout.flush();
    }
}
