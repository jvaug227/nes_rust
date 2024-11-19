use nes_rust::{cpu::{Cpu, CpuPinout}, ppu::{Ppu, PpuPinout}};


pub struct NESBoard {
    cpu: Cpu,
    cpu_pins: CpuPinout,

    ppu: Ppu,
    ppu_pins: PpuPinout,
    ppu_data_latch: u8,

    ram: Vec<u8>,
    cycles: usize,
}

impl NESBoard {
    // Initialize a new circuit
    // Set the inturrupt lines to false so that way the cpu begins startup correctly by detecting a
    // reset inturrupt
    pub fn new(cpu: Cpu, ram: Vec<u8>) -> NESBoard {
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
            cycles
        }
    }

    fn cpu_clock(&mut self, phi: bool) {
        self.cpu_pins.phi = phi;
        let cycle_occured = self.cpu.clock(&mut self.cpu_pins);
        let addr = self.cpu_pins.address_bus as usize;
        if self.cpu_pins.address_rw {
            self.cpu_pins.data_bus = self.ram[addr];
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
}
