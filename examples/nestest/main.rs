use std::error::Error;
use nes::NESBoard;
use nes_rust::{cpu::*, cartidge::CartridgeData };
use anyhow::Result;

mod nes;


fn main() -> Result<(), Box<dyn Error>> {

    let cpu = Cpu::new();

    //let program = vec![0xA2, 0x0A, 0x8E, 0x00, 0x00, 0xA2, 0x03, 0x8E, 0x01, 0x00, 0xAC, 0x00, 0x00, 0xA9, 0x00, 0x18, 0x6D, 0x01, 0x00, 0x88, 0xD0, 0xFA, 0x8D, 0x02, 0x00, 0xEA, 0xEA, 0xEA];
    //let program = include_bytes!("official_only.nes");
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
    // println!("{:?}", &program[cartridge_data.prg_rom_range.clone()]);
    if mirror_count > 1 {
        let program_range = &program[cartridge_data.prg_rom_range.clone()];
        // println!("Needs to mirror");
        for i in 0..mirror_count {
            let mirror_start = 0x8000 + mirror_length*i;
            let mirror_end = mirror_start + mirror_length;
            // println!("Mirror {i} from {mirror_start:0>4X} to {mirror_end:0>4X}");
            ram[mirror_start .. mirror_end].copy_from_slice(program_range);   
        }
    } else {
        ram[0x8000 ..=0xFFFF].copy_from_slice(&program[cartridge_data.prg_rom_range.clone()]);
    }

    ram[0xFFFC] = 0x00;
    ram[0xFFFD] = 0xC0;

    let mut nes_board = NESBoard::new(cpu, ram);

    for _ in 0..26554 {
        nes_board.clock(false);
    }

    Ok(())
}
