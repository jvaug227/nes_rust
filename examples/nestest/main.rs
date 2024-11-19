use std::{num::ParseIntError, str::FromStr};
use nes::NESBoard;
use nes_rust::{cpu::*, cartidge::CartridgeData };
use anyhow::Result;
use thiserror::Error;

mod nes;

#[derive(Error, Debug)]
#[error("Parsing line {line}: {reason}.")]
struct NesTestError {
    line: usize,
    reason: NesTestErrorReason,
}

#[derive(Error, Debug)]
enum NesTestErrorReason {
    #[error("OP:{0}, {1}")]
    OP(String, ParseIntError),
    #[error("PC:{0}, {1}")]
    PC(String, ParseIntError),
    #[error("A: {0}, {1}")]
    A(String, ParseIntError),
    #[error("X: {0}, {1}")]
    X(String, ParseIntError),
    #[error("Y: {0}, {1}")]
    Y(String, ParseIntError),
    #[error("P: {0}, {1}")]
    P(String, ParseIntError),
    #[error("S: {0}, {1}")]
    S(String, ParseIntError),
    #[error("{0} {1}")]
    C(String, ParseIntError),
    #[error("Length is less than 90")]
    LessThan90Length,
}
// 0         1         2         3         4         5         6         7         8         9         
// 0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789
// C000  4C F5 C5  JMP $C5F5                       A:00 X:00 Y:00 P:24 SP:FD PPU:  0, 21 CYC:7
#[derive(Debug, Clone, Copy)]
struct NesTestLine {
    opcode: u8,
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    s: u8,
    starting_cycle: usize,
}

impl FromStr for NesTestLine {
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use NesTestErrorReason::*;
        let s_len = s.len();
        if s_len < 90 {
            println!("Less than 90!");
            return Err(LessThan90Length)
        }
        let st = &s[0..4];
        let pc = u16::from_str_radix(st, 16).map_err(|e| PC(st.to_string(), e))?;
        let st = &s[6..8];
        let opcode = u8::from_str_radix(st, 16).map_err(|e| A(st.to_string(), e))?;
        let st = &s[50..52];
        let a = u8::from_str_radix(st, 16).map_err(|e| A(st.to_string(), e))?;
        let st = &s[55..57];
        let x = u8::from_str_radix(st, 16).map_err(|e| X(st.to_string(), e))?;
        let st = &s[60..62];
        let y = u8::from_str_radix(st, 16).map_err(|e| X(st.to_string(), e))?;
        let st = &s[65..67];
        let p = u8::from_str_radix(st, 16).map_err(|e| P(st.to_string(), e))?;
        let st = &s[71..73];
        let sp = u8::from_str_radix(st, 16).map_err(|e| S(st.to_string(), e))?;
        let st = &s[90..s_len-1];
        let starting_cycle = usize::from_str(st).map_err(|e| C(st.to_string(), e))?;

        Ok(NesTestLine { opcode, pc, a, x, y, p, s: sp, starting_cycle })
    }

    type Err = NesTestErrorReason;
}

fn main() -> Result<()> {

    let cpu = Cpu::new();

    let program = include_bytes!("nestest.nes");
    let cartridge_data = CartridgeData::decode(program);
    println!("Read Catridge: (Maybe Named) {:?}", cartridge_data.title);
    println!("Program is {} bytes", program.len());
    println!("Program Rom Block: {:?} at {} bytes", cartridge_data.prg_rom_range, cartridge_data.prg_rom_range.len());
    println!("Character Rom Block: {:?} at {} bytes", cartridge_data.chr_rom_range, cartridge_data.chr_rom_range.clone().map(|r| r.len()).unwrap_or(0));

    const RAM_SIZE: usize = 256 * 2048;
    const PROGRAM_RANGE: usize = 32768;
    let mut ram = vec![0u8; RAM_SIZE];
    let mirror_count = PROGRAM_RANGE / cartridge_data.prg_rom_range.len();
    let mirror_length = cartridge_data.prg_rom_range.len();
    if mirror_count > 1 {
        let program_range = &program[cartridge_data.prg_rom_range.clone()];
        for i in 0..mirror_count {
            let mirror_start = 0x8000 + mirror_length*i;
            let mirror_end = mirror_start + mirror_length;
            ram[mirror_start .. mirror_end].copy_from_slice(program_range);   
        }
    } else {
        ram[0x8000 ..=0xFFFF].copy_from_slice(&program[cartridge_data.prg_rom_range.clone()]);
    }

    ram[0xFFFC] = 0x00;
    ram[0xFFFD] = 0xC0;

    let nestest_log = include_str!("nestest.log");

    let log_data: Result<Vec<NesTestLine>, NesTestError> = nestest_log.split_terminator('\n').enumerate().map(|(line, s)| NesTestLine::from_str(s).map_err(|reason| NesTestError { reason, line })).collect();
    let log_data = log_data?;

    let mut nes_board = NESBoard::new(cpu, ram, log_data);

    for _ in 0..26555 {
        nes_board.clock(false);
    }

    Ok(())
}
