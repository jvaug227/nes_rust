use nes_rust::{cpu::{Cpu, CpuPinout}, ppu::{Ppu, PpuPinout}};

pub struct NESBoard {
    cpu: Cpu,
    cpu_pins: CpuPinout,

    ppu: Ppu,
    ppu_pins: PpuPinout,
    ppu_address_latch: u8,

    ram: Vec<u8>,
    vram: Vec<u8>,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    cycles: usize,

    dma_active: bool,
    dma_address: u8,
}

impl NESBoard {
    // Initialize a new circuit
    // Set the inturrupt lines to false so that way the cpu begins startup correctly by detecting a
    // reset inturrupt
    pub fn new(cpu: Cpu, internal_ram: Vec<u8>, internal_vram: Vec<u8>, prg_rom: Vec<u8>, chr_rom: Vec<u8>, ram_size: u8) -> NESBoard {
        let cycles = 0;
        let cpu_pins = CpuPinout { irq: false, nmi: false, reset: false, phi: false, ready: false, data_bus: 0, address_bus: 0, address_rw: true, sync: false };
        let ppu = Ppu::new();
        let ppu_pins = PpuPinout { nmi: false, cpu_rw: false, cpu_data: 0, ppu_address_data_low: 0, ppu_address_high: 0, ppu_r: false, ppu_w: false, ppu_sync: false, ppu_ale: false, cpu_control: false, cpu_addr: 0, finished_frame: false, };
        let prg_ram = vec![0u8; ram_size as usize];
        NESBoard {
            cpu,
            cpu_pins,
            ppu,
            ppu_pins,
            ppu_address_latch: 0,
            ram: internal_ram,
            vram: internal_vram,
            prg_rom,
            chr_rom,
            prg_ram,
            cycles,
            dma_active: false,
            dma_address: 0,
        }
    }

    fn cpu_clock(&mut self, phi: bool) {
        self.cpu_pins.phi = phi;
        let cycle_occured = self.cpu.clock(&mut self.cpu_pins);
        if cycle_occured {
            self.cycles = self.cycles.wrapping_add(1);
        }
        if self.cpu_pins.address_rw && !phi {
            self.cpu_mem_read();
        } else if !self.cpu_pins.address_rw && phi {
            self.cpu_mem_write();
        }
    }

    // Assume PHI 1 and read configuration
    fn cpu_mem_read(&mut self) {
        let addr = self.cpu_pins.address_bus;
        match addr {
            0x0000..0x2000 => {
                let addr = usize::from(addr) % 0x0800;
                self.cpu_pins.data_bus = self.ram[addr];
            },
            0x2000..0x4000 => {
                // access ppu
                let addr = usize::from(addr - 0x2000) % 8;
                self.ppu_pins.cpu_rw = self.cpu_pins.address_rw;
                self.ppu_pins.cpu_addr = addr as u8;
                self.ppu_pins.cpu_control = true;
                // Data bus will be filled via the ppu_block fn
            },
            0x4000..0x4020 => {
                // apu and IO
            },
            0x4020..0x6000 => {
                // nothing
            },
            0x6000..0x8000 => {
                // prg ram
                let addr = usize::from(addr - 0x6000) % 0x0800;
                self.cpu_pins.data_bus = self.prg_ram[addr];
            },
            0x8000..=0xFFFF => {
                // prg rom
                let addr = usize::from(addr - 0x8000) % self.prg_rom.len();
                self.cpu_pins.data_bus = self.prg_rom[addr];
            }
        }

    }

    // Assume PHI 2 and write configuration
    fn cpu_mem_write(&mut self) {
        let addr = self.cpu_pins.address_bus;
        match addr {
            0x0000..0x2000 => {
                let addr = usize::from(addr) % 0x0800;
                self.ram[addr] = self.cpu_pins.data_bus;
            },
            0x2000..0x4000 => {
                // access ppu
                let addr = usize::from(addr - 0x2000) % 8;
                self.ppu_pins.cpu_rw = self.cpu_pins.address_rw;
                self.ppu_pins.cpu_addr = addr as u8;
                self.ppu_pins.cpu_control = true;
                self.ppu_pins.cpu_data = self.cpu_pins.data_bus; // hand over data from cpu to ppu 
            },
            0x4000..0x4020 => {
                match addr {
                    0x4014 => {
                        // OAM DMA
                        self.dma_active = true;
                        self.dma_address = self.cpu_pins.data_bus;
                    },
                    _ => {}
                }
                // apu and IO
            },
            0x4020..0x6000 => {
                // nothing
            },
            0x6000..0x8000 => {
                // prg ram, if available (Will panic if accessed and no ram)
                let addr = usize::from(addr - 0x6000) % 0x0800;
                self.prg_ram[addr] = self.cpu_pins.data_bus;
            },
            0x8000..=0xFFFF => {
                // prg rom, no writes
            }
        }

    }

    fn ppu_clock(&mut self) {
        self.ppu.clock(&mut self.ppu_pins);
        self.cpu_pins.nmi = self.ppu_pins.nmi;
        if self.ppu_pins.ppu_ale {
            self.ppu_address_latch = self.ppu_pins.ppu_address_data_low;
        }
        let addr = ((self.ppu_pins.ppu_address_high as usize) << 8) | self.ppu_address_latch as usize;
        if self.ppu_pins.ppu_w {
            // println!("\tWriting {0} to 0x{1:0>4X} ({1})", self.ppu_pins.ppu_address_data_low, addr);
        } else {
            // println!("\tReading from 0x{0:0>4X} ({0})", addr);
        }
        if self.ppu_pins.ppu_r || self.ppu_pins.ppu_w {
            match addr {
                0x0000..0x2000 => {
                    if self.ppu_pins.ppu_r {
                        self.ppu_pins.ppu_address_data_low = self.chr_rom[addr];
                    } else {
                        println!("\t!!!Writing to an RO portion of vram: 0x{:0>4X}({})!!!", addr, addr);
                    }
                },
                0x2000.. => {
                    // internal NES vram or mapped by cartidge
                    // For nametables
                    let addr = (addr - 0x2000) % self.vram.len();
                    if self.ppu_pins.ppu_r {
                        // TODO: Determine if this needs to occur during the ALE cycle or the next
                        // cycle - this depends on if the cpu expects the data the same cycle it
                        // enables the read line or the line after
                        self.ppu_pins.ppu_address_data_low = self.vram[addr];
                    } else {
                        self.vram[addr] = self.ppu_pins.ppu_address_data_low;
                    }
                },
                // 0x3000.. => {
                //     println!("\t!!!Writing to an RO portion of vram: 0x{:0>4X}({})!!!", addr, addr);
                    // 0x3000..0x3EFF => unused mirron of vram
                    // 0x3F00.. is the start of palette ram, but it's internal to the PPU and doesn't
                    // reach here
                // }
                
            }
        }

        // pass buffered data back to the cpu during a cpu read - this should be occuring between phi1 and
        // phi2
        if self.ppu_pins.cpu_control && self.ppu_pins.cpu_rw {
            self.cpu_pins.data_bus = self.ppu_pins.cpu_data;
        }
        // Don't keep the ppu in a state of manipulating registers
        self.ppu_pins.cpu_control = false;
    }

    // Emulate one master clock cycle
    pub fn clock(&mut self, _ready: bool) {

        self.ppu_clock();
        self.ppu_clock();

        self.cpu_clock(false);

        // One ppu clock between phi1 and phi2 to handle reading from ppu
        self.ppu_clock();

        self.cpu_clock(true);

        // Reset inturrupt requests
        self.cpu_pins.reset = true;
        self.cpu_pins.irq = true;
        self.cpu_pins.nmi = true; // might be unnecessary as ppu manages nmi
    }

    pub fn dump_ppu(&self) {
        // for i in 0..(self.vram.len() / 16) {
        //     let vram = &self.vram;
        //     println!("{i}: [ {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}, {:0>2X}]", 
        //         vram[i +  0], vram[i +  1], vram[i +  2], vram[i +  3],
        //         vram[i +  4], vram[i +  5], vram[i +  6], vram[i +  7],
        //         vram[i +  8], vram[i +  9], vram[i + 10], vram[i + 11],
        //         vram[i + 12], vram[i + 13], vram[i + 14], vram[i + 15]);
        // }
        self.ppu.dump();
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

    pub fn finished_frame(&self) -> bool {
        self.ppu_pins.finished_frame
    }

    pub fn reset_finished_frame(&mut self) {
        self.ppu_pins.finished_frame = false
    }

    pub fn pattern_table_memory(&self) -> &[u8] {
        &self.chr_rom[0x0000..0x2000]
    }

    pub fn nametable_memory(&self, index: usize) -> &[u8] {
        let start = 0x0400 * index;
        let end = start + 0x0400;
        &self.vram[start..end]
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
}
