use std::ops::Range;
pub mod mapper;

pub struct CartridgeData {
    pub trainer_range: Option<Range<usize>>,
    pub prg_rom_range: Range<usize>,
    pub chr_rom_range: Option<Range<usize>>,
    pub playchoice_rom_range: Option<Range<usize>>,
    pub playchoice_prom_range: Option<Range<usize>>,
    pub nametable_arrangement: NameTableArrangement,
    pub nametable_alternate: bool,
    pub battery: bool,
    pub title: Option<String>,
    pub mapper: usize,
}

pub enum CartidgeFileFormat {
    INESArchaic,
    INES,
    NES2,
    TNES,
}

impl CartridgeData {
    pub fn decode(program: &[u8]) -> Self {
        let data_size = program.len();
        let byte0 = program[0];
        let byte1 = program[1];
        let byte2 = program[2];
        let byte3 = program[3];

        let byte7 = program[7];
        /*let byte9 = program[9];*/

        let format_test = byte7 & 0x0C;
        let is_nes_name = byte0 == b'N' && byte1 == b'E' && byte2 == b'S' && byte3 == 0x1A;
        let is_tnes_name = byte0 == b'T' && byte1 == b'N' && byte2 == b'E' && byte3 == b'S';

        // More thorough testing of Cartidge formats is needed, but this follows the recommended
        // procedure set by https://www.nesdev.org/wiki/INES
        let format = if is_nes_name {
            match format_test {
                0x08 /*if usize::from(byte9) < rom_size*/ => CartidgeFileFormat::NES2,
                0x04 => CartidgeFileFormat::INESArchaic,
                0x00 => CartidgeFileFormat::INES,
                _ => CartidgeFileFormat::INESArchaic,
            } 
        }
        else if is_tnes_name {
            CartidgeFileFormat::TNES
        }
        else { unimplemented!() }; 
        
        match format {
            CartidgeFileFormat::INESArchaic => {
                let ines_archaic_data = INESArchaicFormat::decode(program);
                let mut offset = 16usize;
                let trainer_range = if ines_archaic_data.contains_trainer {
                    let begin = offset;
                    offset += 512usize;
                    let end = offset;
                    Some(begin..end)
                }  else {
                    None
                };

                let prg_rom_range = {
                    let range = ines_archaic_data.calculate_prg_rom_size();
                    let begin = offset;
                    offset += range;
                    let end = offset;
                    begin..end
                };

                let chr_rom_range = {
                    let range = ines_archaic_data.calculate_chr_rom_size();
                    if range == 0 {
                        None // Uses CHR_RAM instead
                    } else {
                        let begin = offset;
                        offset += range;
                        let end = offset;
                        Some(begin..end)
                    }
                };

                let playchoice_rom_range = None;
                let playchoice_prom_range = None;

                let nametable_arrangement = ines_archaic_data.nametable_arrangement;
                let nametable_alternate = ines_archaic_data.alternative_nametable_layout;

                let battery = ines_archaic_data.non_volatile_data;
                let title = None;
                let mapper = ines_archaic_data.mapper;
                Self {
                    trainer_range,
                    prg_rom_range,
                    chr_rom_range,
                    playchoice_rom_range,
                    playchoice_prom_range,
                    nametable_arrangement,
                    nametable_alternate,
                    battery,
                    title,
                    mapper,
                }
            },
            CartidgeFileFormat::INES => {
                let base = INESArchaicFormat::decode(program);
                let ines_data = INESFormat::decode(program, base);

                let mut offset = 16usize;
                let trainer_range = if ines_data.base.contains_trainer {
                    let begin = offset;
                    offset += 512usize;
                    let end = offset;
                    Some(begin..end)
                }  else {
                    None
                };

                let prg_rom_range = {
                    let range = ines_data.calculate_prg_rom_size();
                    let begin = offset;
                    offset += range;
                    let end = offset;
                    begin..end
                };

                let chr_rom_range = {
                    let range = ines_data.calculate_chr_rom_size();
                    if range == 0 {
                        None // Uses CHR_RAM instead
                    } else {
                        let begin = offset;
                        offset += range;
                        let end = offset;
                        Some(begin..end)
                    }
                };

                let playchoice_rom_range = None;
                let playchoice_prom_range = None;

                let nametable_arrangement = ines_data.base.nametable_arrangement;
                let nametable_alternate = ines_data.base.alternative_nametable_layout;

                let battery = ines_data.base.non_volatile_data;
                let title = None;
                let mapper = ines_data.base.mapper;
                Self {
                    trainer_range,
                    prg_rom_range,
                    chr_rom_range,
                    playchoice_rom_range,
                    playchoice_prom_range,
                    nametable_arrangement,
                    nametable_alternate,
                    battery,
                    title,
                    mapper,
                }
            },
            CartidgeFileFormat::NES2 => {
                let base = INESArchaicFormat::decode(program);
                let ines2_data = INES2Format::decode(program, base);

                let mut offset = 16usize;
                let trainer_range = if ines2_data.base.contains_trainer {
                    let begin = offset;
                    offset += 512usize;
                    let end = offset;
                    Some(begin..end)
                }  else {
                    None
                };

                let prg_rom_range = {
                    let range = ines2_data.calculate_prg_rom_size();
                    let begin = offset;
                    offset += range;
                    let end = offset;
                    begin..end
                };

                let chr_rom_range = {
                    let range = ines2_data.calculate_chr_rom_size();
                    if range == 0 {
                        None // Uses CHR_RAM instead
                    } else {
                        let begin = offset;
                        offset += range;
                        let end = offset;
                        Some(begin..end)
                    }
                };

                let playchoice_rom_range = None;
                let playchoice_prom_range = None;

                let nametable_arrangement = ines2_data.base.nametable_arrangement;
                let nametable_alternate = ines2_data.base.alternative_nametable_layout;

                let battery = ines2_data.base.non_volatile_data;
                let title = None;

                let mapper = ines2_data.base.mapper;
                Self {
                    trainer_range,
                    prg_rom_range,
                    chr_rom_range,
                    playchoice_rom_range,
                    playchoice_prom_range,
                    nametable_arrangement,
                    nametable_alternate,
                    battery,
                    title,
                    mapper,
                }
            },
            _ => {
                let tnes_data = TNESFormat::decode(program);
                let mut offset = 16usize;
                let trainer_range = None;

                let prg_rom_range = {
                    let range = tnes_data.calculate_prg_rom_size();
                    let begin = offset;
                    offset += range;
                    let end = offset;
                    begin..end
                };

                let chr_rom_range = {
                    let range = tnes_data.calculate_chr_rom_size();
                    if range == 0 {
                        None // Uses CHR_RAM instead
                    } else {
                        let begin = offset;
                        offset += range;
                        let end = offset;
                        Some(begin..end)
                    }
                };

                let playchoice_rom_range = None;
                let playchoice_prom_range = None;

                let nametable_arrangement = tnes_data.mirroring;
                let nametable_alternate = false;

                let battery = tnes_data.non_volatile_data;
                let title = None;
                let mapper = match tnes_data.mapper {
                    0 => 0,
                    1 => 1,
                    2 => 9,
                    3 => 4,
                    4 => 10,
                    5 => 5,
                    6 => 2,
                    7 => 3,
                    9 => 7,
                    31 => 86,
                    100 => unimplemented!("FDS is not yet implemented"),
                    _ => unimplemented!("Not a known TNES mapper"),
                };
                Self {
                    trainer_range,
                    prg_rom_range,
                    chr_rom_range,
                    playchoice_rom_range,
                    playchoice_prom_range,
                    nametable_arrangement,
                    nametable_alternate,
                    battery,
                    title,
                    mapper,
                }
            },
        }
    }
}

pub enum NameTableArrangement {
    HORIZONTAL,
    VERTICAL,
    MapperControlled,
}

enum TVSystem {
    Ntsc,
    Pal,
    DualCompatible,
}

enum ConsoleType {
    Nes,
    Nvs,
    NPlaychoice10,
    Extended,
}

enum TimingMode {
    RP2C02, // NTSC NES
    RP2C07, // Liscenced PAL NES
    MulReg,
    UA6538, // Dendy
}

enum ExtraHardwareInfo {
    VSSystemType { ppu_type: u8, hardware_type: u8 },
    ExtendedConsole { extended_console_type: u8 },
}

struct INESArchaicFormat {
    prg_rom_size: usize, // 16KB ROM Banks
    chr_rom_size: usize, // 8KB VROM Banks
    nametable_arrangement: NameTableArrangement,
    non_volatile_data: bool,
    contains_trainer: bool,
    alternative_nametable_layout: bool, // different meanings, but generally a 4-screen layout
    mapper: usize, // END Archaic INES
}

impl INESArchaicFormat {
    fn decode(program: &[u8]) -> Self {
        let byte4 = program[4];
        let byte5 = program[5];
        let byte6 = program[6];
        let prg_rom_size = usize::from(byte4);
        let chr_rom_size = usize::from(byte5);

        let nametable_arrangement = if byte6 & 0x01 == 0 { NameTableArrangement::HORIZONTAL } else { NameTableArrangement::VERTICAL };
        let non_volatile_data = (byte6 & 0x02) > 0;
        let contains_trainer = (byte6 & 0x04) > 0;
        let alternative_nametable_layout = (byte6 & 0x08) > 0;

        let mapper = usize::from(byte6 >> 4);
        
        Self {
            prg_rom_size,
            chr_rom_size,
            nametable_arrangement,
            non_volatile_data,
            contains_trainer,
            alternative_nametable_layout,
            mapper
        }
    }

    fn calculate_prg_rom_size(&self) -> usize {
        16384 * self.prg_rom_size
    }

    fn calculate_chr_rom_size(&self) -> usize {
        8192 * self.chr_rom_size
    }
}

struct INESFormat {
    base: INESArchaicFormat,
    vs_unisystem: bool, // byte 7
    playchoice: bool, // byte 7
    prg_ram_size: usize, // byte 8 - 8KB RAM Banks
    tv_system: TVSystem, // byte 9 - if this is 0 and tv_system_2 is non-zero, use tv_system_2
    tv_system_2: TVSystem, // byte 10 - unofficial
    prg_ram_present_bit: bool, // byte 10 - unofficial
    contains_bus_conflicts: bool, // byte 10 - unofficial
}

impl INESFormat {
    fn decode(program: &[u8], mut base: INESArchaicFormat) -> Self {
        let byte7 = program[7];
        let byte8 = program[8];
        let byte9 = program[9];
        let byte10 = program[10];

        let vs_unisystem = (byte7 & 0x01) > 0;
        let playchoice = (byte7 & 0x02) > 0;
        let mapper_upper_half = usize::from(byte7 & 0b11110000);

        let prg_ram_size = usize::from(byte8);

        let tv_system = if byte9 == 0 { TVSystem::Ntsc } else { TVSystem::Pal };

        let tv_system_2 = match byte10 & 0b11 {
            0 => TVSystem::Ntsc,
            2 => TVSystem::Pal,
            _ => TVSystem::DualCompatible,
        };
        let prg_ram_present_bit = (byte10 & 0x10) > 0;
        let contains_bus_conflicts = (byte10 & 0x20) > 0;

        let byte12 = program[12];
        let byte13 = program[13];
        let byte14 = program[14];
        let byte15 = program[15];
        let upper_bytes_empty = byte12 == 0 && byte13 == 0 && byte14 == 0 && byte15 == 0;
        if upper_bytes_empty {
            base.mapper |= mapper_upper_half;
        }

        Self {
            base,
            vs_unisystem,
            playchoice,
            prg_ram_size,
            tv_system,
            tv_system_2,
            prg_ram_present_bit,
            contains_bus_conflicts
        }
    }

    fn calculate_prg_rom_size(&self) -> usize {
        16384 * self.base.prg_rom_size
    }

    fn calculate_chr_rom_size(&self) -> usize {
        8192 * self.base.chr_rom_size
    }
}

struct INES2Format {
    base: INESArchaicFormat,
    console_type: ConsoleType,
    submapper: u8,
    prg_ram_shift_count: u8,
    prg_nvram_shift_count: u8,
    prg_rom_msb: usize,
    chr_rom_msb: usize,
    chr_ram_size: u8,
    chr_nvram_size: u8,
    timing_mode: TimingMode,
    extra_hardware_info: Option<ExtraHardwareInfo>,
    miscellaneous_roms: u8,
    default_expansion_device: u8,
}

impl INES2Format {
    fn decode(program: &[u8], mut base: INESArchaicFormat) -> Self {
        let byte7 = program[7];
        let byte8 = program[8];
        let byte9 = program[9];
        let byte10 = program[10];
        let byte11 = program[11];
        let byte12 = program[12];
        let byte13 = program[13];
        let byte14 = program[14];
        let byte15 = program[15];

        let console_type = match byte7 & 0b11 {
            1 => ConsoleType::Nvs,
            2 => ConsoleType::NPlaychoice10,
            3 => ConsoleType::Extended,
            _ => ConsoleType::Nes,
        };
        let mapper_middle_half = usize::from(byte7 & 0b11110000);
        
        let mapper_upper_half = usize::from(byte8 & 0b00001111) << 8;
        let submapper = byte8 >> 4;

        let prg_rom_msb = usize::from(byte9 & 0b00001111) << 8;
        let chr_rom_msb = usize::from(byte9 >> 4) << 8;

        let prg_ram_shift_count = byte10 & 0b00001111;
        let prg_nvram_shift_count = byte10 >> 4;

        let chr_ram_size = byte11 & 0b00001111;
        let chr_nvram_size = byte11 >> 4;

        let timing_mode = match byte12 & 0b11 {
            1 => TimingMode::RP2C07,
            2 => TimingMode::MulReg,
            3 => TimingMode::UA6538,
            _ => TimingMode::RP2C02,
        };

        let extra_hardware_info = match &console_type {
            ConsoleType::Nvs => Some(ExtraHardwareInfo::VSSystemType { ppu_type: byte13 & 0b00001111, hardware_type: byte13 >> 4 }),
            ConsoleType::Extended => Some(ExtraHardwareInfo::ExtendedConsole { extended_console_type: byte13 & 0b00001111 }),
            _ => None,
        };

        let miscellaneous_roms = byte14 & 0b11;

        let default_expansion_device = byte15 & 0b00111111;

        base.mapper |= mapper_middle_half | mapper_upper_half;

        Self {
            base,
            console_type,
            submapper,
            prg_ram_shift_count,
            prg_nvram_shift_count,
            prg_rom_msb,
            chr_rom_msb,
            chr_ram_size,
            chr_nvram_size,
            timing_mode,
            extra_hardware_info,
            miscellaneous_roms,
            default_expansion_device
        }
    }

    fn calculate_prg_rom_size(&self) -> usize {
        let base = self.base.prg_rom_size;
        if self.prg_rom_msb == 0xF {
            let mm = (base & 0b00000011) * 2 + 1;
            let ee = (base & 0b11111100) >> 2;
            2usize.pow(ee as u32) * mm
        } else {
            16384 * (base | (self.prg_rom_msb << 8)) 
        }
        
    }

    fn calculate_chr_rom_size(&self) -> usize {
        let base = self.base.chr_rom_size;
        if self.chr_rom_msb == 0xF {
            let mm = (base & 0b00000011) * 2 + 1;
            let ee = (base & 0b11111100) >> 2;
            2usize.pow(ee as u32) * mm
        } else {
            8192 * (base | (self.chr_rom_msb << 8))
        }
        
    }
}

struct TNESFormat {
    mapper: u8,
    prg_rom_size: usize, // 8192 byte banks
    chr_rom_size: usize, // 8192 byte banks
    wram: usize,
    mirroring: NameTableArrangement,
    non_volatile_data: bool,
}

impl TNESFormat {
    fn decode(program: &[u8]) -> Self {
        let byte4 = program[4];
        let byte5 = program[5];
        let byte6 = program[6];
        let byte7 = program[7];
        let byte8 = program[8];
        let byte9 = program[9];

        let mapper = byte4;
        let prg_rom_size = usize::from(byte5);
        let chr_rom_size = usize::from(byte6);
        let wram = usize::from(byte7);
        let nametable_arrangement = match byte8 {
            0 => NameTableArrangement::MapperControlled,
            1 => NameTableArrangement::HORIZONTAL,
            _ => NameTableArrangement::VERTICAL,
        };
        let non_volatile_data = byte9 != 0;

        
        Self {
            mapper,
            prg_rom_size,
            chr_rom_size,
            wram,
            mirroring: nametable_arrangement,
            non_volatile_data,
        }
    }

    fn calculate_prg_rom_size(&self) -> usize {
        8192 * self.prg_rom_size
    }

    fn calculate_chr_rom_size(&self) -> usize {
        8192 * self.chr_rom_size
    }
}
