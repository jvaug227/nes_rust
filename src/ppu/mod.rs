
// True size of Rendering Area
const DOTS_PER_SCANLINE: usize = 341;
// Contains pre-scan line
const SCANLINES_PER_FRAME: usize = 262;
// Actual drawn Region
const DOTS_PER_IMAGE_ROW: usize = 256;
const SCANLINES_PER_IMAGE: usize = 240;

pub const VIDEO_MEMORY_SIZE: usize = DOTS_PER_IMAGE_ROW * SCANLINES_PER_IMAGE * 4;

enum VRamManip {
    /// The cycle enabling the R signal
    Read1,
    /// The cycle after enabling the R signal, data should be on the bus
    Read2,
    Write,
    None,
}

enum DataReadDestination {
    CpuDataBus,
    Nametable,
    AttributeTable,
    TilePatternLo,
    TilePatternHi,
}

struct LoopyRegister(u16);
impl LoopyRegister {
    fn new() -> Self { Self(0) }
    fn set(&mut self, data: u8) { self.0 = (self.0 & 0xF0) | (data as u16); }
    fn shift(&mut self) { self.0 <<= 1; }
    fn get(&self, offset: u16) -> bool { (self.0 & (0x1000 >> offset)) != 0 }
}

pub struct PpuPinout {
    pub nmi: bool,
    pub cpu_control: bool,
    pub cpu_rw: bool,
    pub cpu_addr: u8,
    pub cpu_data: u8,
    pub ppu_address_data_low: u8,
    pub ppu_address_high: u8,
    pub ppu_r: bool,
    pub ppu_w: bool,
    pub ppu_sync: bool,
    pub ppu_ale: bool,
}

pub struct Ppu {
    control_register: u8,
    mask_register: u8,
    status_register: u8,
    oam_address_register: u8,

    temp_address: u16,
    w_register: bool,

    vram_address: u16,
    vram_data: u8,
    vram_address_cycle: bool,
    vram_manip: VRamManip,

    data_destination: DataReadDestination,

    next_nametable: u8,
    next_attribute: u8,
    next_tile_lsb: u8,
    next_tile_msb: u8,

    tile_msb_scroll: LoopyRegister,
    tile_lsb_scroll: LoopyRegister,

    fine_x_scroll: u8,

    scanline: usize,
    cycle: usize,
    is_odd_frame: bool,
    internal_read_buffer: u8,
    oam_memory: [u8; 256],
    palette_memory: [u8; 32],

    video_data: Vec<u8>,
}


impl Ppu {
    pub fn new() -> Self {
        Self {
            control_register: 0,
            mask_register: 0,
            status_register: 0, 
            oam_address_register: 0,
            temp_address: 0,
            w_register: false,

            vram_address: 0,
            vram_data: 0,
            vram_address_cycle: true,
            vram_manip: VRamManip::None,

            data_destination: DataReadDestination::CpuDataBus,

            next_nametable: 0,
            next_attribute: 0,
            next_tile_lsb: 0,
            next_tile_msb: 0,

            tile_msb_scroll: LoopyRegister::new(),
            tile_lsb_scroll: LoopyRegister::new(),

            fine_x_scroll: 0,

            scanline: 261,
            cycle: 0,
            is_odd_frame: false,
            internal_read_buffer: 0,
            oam_memory: [0; 256],
            palette_memory: [0; 32],

            video_data: vec![255; VIDEO_MEMORY_SIZE]
        }
    }

    pub fn clock(&mut self, pins: &mut PpuPinout) {
        pins.nmi = true;

        pins.ppu_ale = self.w_register;

        if self.w_register {
            pins.ppu_address_data_low = self.vram_address as u8;
            pins.ppu_address_high = (self.vram_address >> 8) as u8;
            self.vram_address_cycle = false;
        } else {
            match self.vram_manip {
                VRamManip::Read1 => {
                    self.vram_manip = VRamManip::Read2;
                    pins.ppu_r = true;
                }
                VRamManip::Read2 => {
                    self.internal_read_buffer = pins.ppu_address_data_low;
                    self.vram_manip = VRamManip::None;
                }
                VRamManip::Write => {
                    pins.ppu_address_data_low = self.vram_data;
                    pins.ppu_w = true;
                    self.vram_manip = VRamManip::None;
                }
                VRamManip::None => {
                    pins.ppu_r = false;
                    pins.ppu_w = false;
                }
            }
        }

        if pins.cpu_control {
            self.handle_cpu_io(pins);
        }

        if self.is_render_fetch_cycle() {
            if let Some(address) = self.render_fetch() {
                pins.ppu_address_data_low = address as u8;
                pins.ppu_address_high = (address >> 8) as u8;
                pins.ppu_ale = true;
                self.vram_manip = VRamManip::Read1;
            }
        }

        if (self.enabled_sprite_rendering() || self.enabled_background_rendering()) && self.cycle == 256 {
            if (self.vram_address & 0x7000) != 0x7000 {                         // if fine Y < 7
                self.vram_address += 0x1000;                                    // increment fine Y
            }
            else {
                self.vram_address &= !0x7000;                                   // fine Y = 0
                let mut y = (self.vram_address & 0x03E0) >> 5;                  // let y = coarse Y
                if y == 29 {
                    y = 0;                                                      // coarse Y = 0
                    self.vram_address ^= 0x0800;                                // switch vertical nametable
                }
                else if y == 31 {
                    y = 0;                                                      // coarse Y = 0, nametable not switched
                }
                else {
                    y += 1;                                                     // increment coarse Y
                }
                self.vram_address = (self.vram_address & !0x03E0) | (y << 5);   // put coarse Y back into v
            }
        }

        if self.cycle == 257 {
            const X_MASK: u16 = 0b1111101111100000;
            self.vram_address = (self.vram_address & X_MASK) | (self.temp_address & !X_MASK);
            self.tile_msb_scroll.set(self.next_tile_msb);
            self.tile_lsb_scroll.set(self.next_tile_lsb);
        }

        if self.scanline == 262 && (280..305).contains(&self.cycle) {
            const Y_MASK: u16 = 0b1111101111100000;
            self.vram_address = (self.vram_address & !Y_MASK) | (self.temp_address & Y_MASK);
        }

        if self.is_render_cycle() {
            let point = (self.scanline * 256 + self.cycle) * 4;
            let colors = [
                0xFF, 0x00, 0x00,
                0x00, 0xFF, 0x00,
                0x00, 0x00, 0xFF,
                0xF0, 0xF0, 0x00,
            ];
            let msb = self.tile_msb_scroll.get(self.fine_x_scroll as u16) as u8;
            let lsb = self.tile_lsb_scroll.get(self.fine_x_scroll as u16) as u8;
            let tint = usize::from((msb << 1) | lsb) * 3;
            self.video_data[point    ] = colors[ tint + 0 ];
            self.video_data[point + 1] = colors[ tint + 1 ];
            self.video_data[point + 2] = colors[ tint + 2 ];
            self.video_data[point + 3] = 255;
            self.tile_lsb_scroll.shift();
            self.tile_msb_scroll.shift();
        }

        if self.is_begin_vblank_cycle() {
            self.set_vblank_flag(true);
            pins.nmi = !self.nmi_enabled();
        }

        if self.is_end_vblank_cycle() {
            self.set_vblank_flag(false);
            self.is_odd_frame = !self.is_odd_frame;
        }

        self.cycle = self.cycle.wrapping_add(1) % DOTS_PER_SCANLINE;
        if self.cycle == 0 { self.scanline = self.scanline.wrapping_add(1) % SCANLINES_PER_FRAME; }
    }

    pub fn video_data(&self) -> &[u8] {
        &self.video_data
    }

    fn is_render_fetch_cycle(&self) -> bool {
        ((1..257).contains(&self.cycle) || (321..341).contains(&self.cycle)) && ((0..240).contains(&self.scanline) || self.scanline == 261)
    }

    fn is_render_cycle(&self) -> bool {
        (0..240).contains(&self.scanline) && (0..256).contains(&self.cycle)
    }

    fn is_begin_vblank_cycle(&self) -> bool {
        self.cycle == 1 && self.scanline == 241
    }

    fn is_end_vblank_cycle(&self) -> bool {
        self.cycle == 1 && self.scanline == 261
    }

    fn render_fetch(&mut self) -> Option<u16> {
        let v = self.vram_address;
        let v_cycle = self.cycle-1;
        let cycle_fetch_period = v_cycle % 8;
        match cycle_fetch_period {
            0 => {
                // Read in MSB just in-time
                self.next_tile_msb = self.internal_read_buffer;

                self.tile_msb_scroll.set(self.next_tile_msb);
                self.tile_lsb_scroll.set(self.next_tile_lsb);

                let nametable_address = 0x2000 | (v & 0x0FFF);
                Some(nametable_address)
            }
            2 => {
                self.next_nametable = self.internal_read_buffer;

                let mut attribute_address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
                if (self.vram_address & 0x40) > 0 { attribute_address >>= 4 };
                if (self.vram_address & 0x02) > 0 { attribute_address >>= 2 };
                attribute_address &= 0x03;
                Some(attribute_address)
            }
            4 => {
                self.next_attribute = self.internal_read_buffer;

                let tile_lsb_address = ((self.background_pattern_table() as u16) << 12)
                    + (u16::from(self.next_nametable) << 4) + ((self.vram_address >> 12) & 0b111);
                Some(tile_lsb_address)
            }
            6 => {
                self.next_tile_lsb = self.internal_read_buffer;

                let tile_msb_address = ((self.background_pattern_table() as u16) << 12)
                    + (u16::from(self.next_nametable) << 4) + ((self.vram_address >> 12) & 0b111) + 8;
                Some(tile_msb_address)
            }
            7 => {
                if self.enabled_background_rendering() || self.enabled_sprite_rendering() {
                    if self.vram_address & 0x001F == 31 {
                        self.vram_address &= !0x001F;
                        self.vram_address ^= 0x0400;
                    } else {
                        self.vram_address = self.vram_address.wrapping_add(1);
                    }
                }
                // Increment hoizontal
                None
            }
            _ => None // second cycles of fetches that don't do anything
        }

    }

    fn base_nametable_address(&self) -> u8 {
        self.control_register & 0b00000011
    }
    fn vram_address_increment(&self) -> u8 {
        if (self.control_register & 0b00000100) > 0 { 32 } else { 1 }
    }
    fn sprite_pattern_table(&self) -> u8 {
        (self.control_register & 0b00001000) >> 3
    }
    fn background_pattern_table(&self) -> u8 {
        (self.control_register & 0b00010000) >> 4
    }
    fn sprite_size(&self) -> bool {
        (self.control_register & 0b00100000) > 0
    }
    fn master_slave_select(&self) -> bool {
        (self.control_register & 0b01000000) > 0
    }
    fn nmi_enabled(&self) -> bool {
        (self.control_register & 0b10000000) > 0
    }

    fn greyscale(&self) -> bool {
        (self.mask_register & 0b1) > 0
    }
    fn show_background_left_bits(&self) -> bool {
        (self.mask_register & 0b10) > 0
    }
    fn show_sprite_left_bits(&self) -> bool {
        (self.mask_register & 0b100) > 0
    }
    fn enabled_background_rendering(&self) -> bool {
        (self.mask_register & 0b1000) > 0
    }
    fn enabled_sprite_rendering(&self) -> bool {
        (self.mask_register & 0b10000) > 0
    }
    fn emphasize_red(&self) -> bool {
        (self.mask_register & 0b100000) > 0
    }
    fn emphasize_green(&self) -> bool {
        (self.mask_register & 0b1000000) > 0
    }
    fn emphasize_blue(&self) -> bool {
        (self.mask_register & 0b10000000) > 0
    }

    fn set_sprite_overflow(&mut self, status: bool) {
        self.status_register = (self.status_register & 0b11011111) | ((status as u8) << 5)
    }
    fn set_sprite_hit(&mut self, status: bool) {
        self.status_register = (self.status_register & 0b10111111) | ((status as u8) << 6)
    }
    fn set_vblank_flag(&mut self, status: bool) {
        self.status_register = (self.status_register & 0b01111111) | ((status as u8) << 7)
    }

    fn handle_cpu_io(&mut self, pins: &mut PpuPinout) {
        match pins.cpu_addr {
            0 => {
                // going to assume write is intended.. for now
                self.control_register = pins.cpu_data;
                self.temp_address = (self.temp_address & 0b1111001111111111) | ((self.base_nametable_address() as u16) << 10);
            }
            1 => {
                // going to assume write is intended.. for now
                self.mask_register = pins.cpu_data;
            }
            2 => {
                pins.cpu_data = self.status_register;
                self.set_vblank_flag(false);
                self.w_register = false;
            }
            3 => {
                self.oam_address_register = pins.cpu_data;
            }
            4 => {
                if pins.cpu_rw {
                    pins.cpu_data = self.oam_memory[self.oam_address_register as usize];
                } else {
                    self.oam_memory[self.oam_address_register as usize] = pins.cpu_data;
                    self.oam_address_register = self.oam_address_register.wrapping_add(1);
                }
            }
            5 => {
                if !self.w_register {
                    self.fine_x_scroll = pins.cpu_data & 0x07;
                } else {
                    self.temp_address = (self.temp_address & 0x0FFF) | (((pins.cpu_data & 0x07) as u16) << 12);
                    self.temp_address = (self.temp_address & 0xFC1F) | (((pins.cpu_data as u16) >> 3) << 5)
                }
                self.w_register = !self.w_register;
            }
            6 => {
                if !self.w_register {
                    self.temp_address = (self.temp_address & 0x00FF) | ((pins.cpu_data as u16) << 8);
                } else {
                    self.temp_address = (self.temp_address & 0xFF00) | (pins.cpu_data as u16);
                    self.vram_address = self.temp_address;
                }
                self.w_register = !self.w_register;
            }
            _ => { // assume 7
                if pins.cpu_rw {
                    pins.cpu_data = if (0x3F00..=0x3FFF).contains(&self.temp_address) {
                        let palette_address = (self.temp_address - 0x3F00) % 0x20;
                        self.palette_memory[palette_address as usize]
                    } else {
                        self.internal_read_buffer
                    };
                    // tell ppu to read from addr
                    self.vram_address_cycle = true;
                    self.vram_manip = VRamManip::Read1;
                } else {
                    // tell ppu to write to addr
                    if (0x3F00..=0x3FFF).contains(&self.temp_address) {
                        let palette_address = (self.temp_address - 0x3F00) % 0x20;
                        self.palette_memory[palette_address as usize] = pins.cpu_data;
                    } else {
                        self.vram_data = pins.cpu_data;
                        self.vram_address_cycle = true;
                        self.vram_address = self.temp_address;
                        self.vram_manip = VRamManip::Write;
                    };

                }
                self.temp_address = self.temp_address.wrapping_add(self.vram_address_increment() as u16);
                self.vram_address = self.temp_address;
            }
        }
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}
