
// True size of Rendering Area
const DOTS_PER_SCANLINE: usize = 341;
// Contains pre-scan line
const SCANLINES_PER_FRAME: usize = 262;
// Actual drawn Region
const DOTS_PER_IMAGE_ROW: usize = 256;
const SCANLINES_PER_IMAGE: usize = 240;

pub const VIDEO_MEMORY_SIZE: usize = DOTS_PER_IMAGE_ROW * SCANLINES_PER_IMAGE * 4;

#[derive(PartialEq, PartialOrd, Debug)]
enum VRamManip {
    /// No action is to be taken
    None,
    /// The cycle enabling the R signal
    /// Doesn't actually read the byte until the start of the cycle it returns to None
    Read,
    /// The cycle enabling the W signal
    /// Outputs the data the same cycle
    Write,
}

struct LoopyShiftRegister(u16);
impl LoopyShiftRegister {
    fn new() -> Self { Self(0) }
    fn set(&mut self, data: u8) { self.0 = (self.0 & 0xFF00) | (data as u16); }
    fn shift(&mut self) { self.0 <<= 1; }
    fn get(&self, offset: u8) -> bool { (self.0 & (0x8000 >> u16::from(offset))) > 0 }
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

    /// Will exist to tell the outside world that a frame should have been completed
    /// Not an official pin in the NES
    pub finished_frame: bool,
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
    vram_manip: VRamManip,

    next_nametable: u8,
    next_attribute: u8,
    next_tile_lsb: u8,
    next_tile_msb: u8,

    tile_msb_scroll: LoopyShiftRegister,
    tile_lsb_scroll: LoopyShiftRegister,
    attribute_msb_scroll: LoopyShiftRegister,
    attribute_lsb_scroll: LoopyShiftRegister,

    fine_x_scroll: u8,

    scanline: usize,
    cycle: usize,
    is_odd_frame: bool,
    internal_read_buffer: u8,
    oam_memory: [u8; 256],
    frame_palette_memory: [u8; 32],
    system_palette_memory: [u8; 64 * 3],

    /// TODO: remove from the ppu
    /// Treat ppu as a pixel-generating function
    video_data: Vec<u8>,
}


impl Ppu {
    pub fn dump(&self) {
        println!("\n=====PPU DUMP=====");
        println!("v-address: {:0>4X}", self.vram_address);
        println!("t-address: {:0>4X}", self.temp_address);
        println!("w-latch: {}", self.w_register);
        println!("cycle: {}, scanline: {}", self.cycle, self.scanline);
        println!("MSB Shift register: {:0>16b}", self.tile_msb_scroll.0);
        println!("LSB Shift register: {:0>16b}", self.tile_lsb_scroll.0);
        println!("Next Nametable: {:0>2X}", self.next_nametable);
        println!("Next MSB byte: {:0>2X}", self.next_tile_msb);
        println!("Next LSB byte: {:0>2X}", self.next_tile_lsb);
        println!("Next Attribute: {:0>2X}", self.next_attribute);
    }

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
            vram_manip: VRamManip::None,

            next_nametable: 0,
            next_attribute: 0,
            next_tile_lsb: 0,
            next_tile_msb: 0,

            tile_msb_scroll: LoopyShiftRegister::new(),
            tile_lsb_scroll: LoopyShiftRegister::new(),
            attribute_msb_scroll: LoopyShiftRegister::new(),
            attribute_lsb_scroll: LoopyShiftRegister::new(),

            fine_x_scroll: 0,

            scanline: 261,
            cycle: 0,
            is_odd_frame: false,
            internal_read_buffer: 0,
            oam_memory: [0; 256],
            // 6-bit lookup into real palettes
            // $3FYX:
            //  Y = 0h BG, 1h Sprite
            //  X = 0h - Fh, X = 0, 4, 8, C is always 0
            frame_palette_memory: [0; 32],
            system_palette_memory: [0; 64 * 3],

            video_data: vec![255; VIDEO_MEMORY_SIZE]
        }
    }

    pub fn set_palette(&mut self, data: &[u8; 64*3]) {
        self.system_palette_memory.clone_from_slice(data);
    }

    pub fn video_data(&self) -> &[u8] {
        &self.video_data
    }

    pub fn clock(&mut self, pins: &mut PpuPinout) {

        // Grab read byte before we manipulate the state of the ppu at all
        if self.vram_manip == VRamManip::Read && pins.ppu_r {
            self.internal_read_buffer = pins.ppu_address_data_low;
            self.vram_manip = VRamManip::None;
        }

        // Reset to reasonable defaults,
        // While it's pratically guaranteed that the ppu will always be reading during the
        // non-vblank period, the vblank period is governed by the cpu and its unclear what the ppu
        // does when not asked to read/write, so Im just choosing to default it to nothing.
        pins.nmi = true;
        pins.ppu_r = false;
        pins.ppu_w = false;
        pins.ppu_ale = false;

        // Why do I keep switching between checking-before-calling and checking-after-calling
        if pins.cpu_control {
            self.handle_cpu_io(pins);
        }

        // Render a pixel (if rendering), and/or configure bus to read from vram
        self.render(pins);

        // If we manipulated the address, put the address on the bus and don't set a read or write
        // flag; else, use the bus accordingly.
        // ppu_ale should go low every other tick as long as the ppu is not configured again
        if pins.ppu_ale {
        } else {
            match self.vram_manip {
                VRamManip::Read => {
                    pins.ppu_r = true;
                    // will finish at the very start of the next cycle after bus operations
                }
                VRamManip::Write => {
                    self.vram_manip = VRamManip::None;
                    pins.ppu_address_data_low = self.vram_data;
                    pins.ppu_w = true;
                }
                _ => { }
            }
        }

    }

    fn render(&mut self, pins: &mut PpuPinout) {
        if self.is_rendering_enabled() {
            if self.is_render_fetch_cycle() {
                if let Some(address) = self.render_fetch() {
                    pins.ppu_address_data_low = address as u8;
                    pins.ppu_address_high = (address >> 8) as u8;
                    pins.ppu_ale = true;
                    self.vram_manip = VRamManip::Read;
                }
            }

            if self.cycle == 256 && self.scanline < 240 {
                self.increment_y();
            }

            // "Transfer X" + 
            if self.cycle == 257 && (self.scanline < 240 || self.scanline == 261) {
                const X_MASK: u16 = 0b1111101111100000;
                self.vram_address = (self.vram_address & X_MASK) | (self.temp_address & !X_MASK);
            }

            // "Transfer Y"
            if self.scanline == 261 && (280..305).contains(&self.cycle) {
                const Y_MASK: u16 = 0b0111101111100000;
                self.vram_address = (self.vram_address & !Y_MASK) | (self.temp_address & Y_MASK);
            }
            if self.is_render_cycle() {
                let sprite_palette_index = if self.is_render_cycle() && self.enabled_sprite_rendering() {
                    0
                } else { 0 };
                let bg_palette_index =  if self.enabled_background_rendering() {
                    let offset = self.get_fine_x();
                    let tile_msb = self.tile_msb_scroll.get(offset) as u8;
                    let tile_lsb = self.tile_lsb_scroll.get(offset) as u8;

                    let attribute_msb = self.attribute_msb_scroll.get(offset) as u8;
                    let attribute_lsb = self.attribute_lsb_scroll.get(offset) as u8;

                    // 2-bit value selecting index inside the palette
                    let pixel_value = usize::from((tile_msb << 1) | tile_lsb);
                    // 2-bit value selecting which palette
                    let attribute_value = usize::from((attribute_msb<<1) | attribute_lsb);
                    pixel_value | (attribute_value << 2)
                } else { 0 };

                let pixels = [bg_palette_index, sprite_palette_index];
                let priority = 0;
                // 1-bit value selecting which half (bg/sprite) of frame palette ram to access
                let winning_pixel = 0;
                let system_palette_index = self.frame_palette_memory[(winning_pixel << 5) | pixels[winning_pixel]] as usize;
                // Copy range of rgb from system palette to video data; the alpha should always be
                // 255 as the initial values in video memory are 255 and alpha is never touched
                // again.
                let tint = system_palette_index * 3;
                let point = (self.scanline * 256 + self.cycle) * 4;
                self.video_data[point..(point+3)].copy_from_slice(&self.system_palette_memory[tint..(tint+3)]);
            }
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

    fn render_fetch(&mut self) -> Option<u16> {
        // A shift happens every cycle there is a possibility of reading from vram
        self.tile_msb_scroll.shift();
        self.tile_lsb_scroll.shift();
        self.attribute_msb_scroll.shift();
        self.attribute_lsb_scroll.shift();

        let v = self.vram_address;
        let v_cycle = self.cycle-1;
        let cycle_fetch_period = v_cycle % 8;
        match cycle_fetch_period {
            0 => {
                // Read in MSB just in-time
                self.next_tile_msb = self.internal_read_buffer;

                self.tile_msb_scroll.set(self.next_tile_msb);
                self.tile_lsb_scroll.set(self.next_tile_lsb);
                // Extend the bit to a full set of 1's or 0's across the scroll register
                self.attribute_msb_scroll.set(0xFF * ((self.next_attribute >> 1) & 1));
                self.attribute_lsb_scroll.set(0xFF * (self.next_attribute & 1));


                let nametable_address = 0x2000 | (v & 0x0FFF);
                Some(nametable_address)
            }
            2 => {
                self.next_nametable = self.internal_read_buffer;

                let attribute_address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
                Some(attribute_address)
            }
            4 => {
                self.next_attribute = self.internal_read_buffer;
                // Palette is one of 4 2-bit indicies into the frame_palette_memory
                // bit 1 (second bit) of course-y determines indicies 0|1 or 2|3
                // bit 1 (second bit) of course-x determines indicies 0|2 or 1|3
                // e.g.                                         [ 3  2  1  0]
                //  course-x = 1, course-y = 0, attribute-byte = [01|11|10|00]
                //      course-y = 0, so indicies 0 and 1,
                //      course-x = 1, so index 1
                //  course-x = 1, course-y = 1
                //      course-y = 1, so indicies 2 and 3
                //      course-x = 1, so index 3

                // if (self.vram_address & 0x40) > 0 { self.next_attribute >>= 4 };
                // if (self.vram_address & 0x02) > 0 { self.next_attribute >>= 2 };
                let masked_address = self.vram_address & 0x42;
                let x = ((masked_address >> 1) & 1) * 2;
                let y = ((masked_address >> 6) & 1) * 4;
                self.next_attribute >>= x|y;
                self.next_attribute &= 0x03;

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
                self.increment_x();
                // Increment hoizontal
                None
            }
            _ => None // second cycles of fetches that don't do anything
        }

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
                    self.set_fine_x(pins.cpu_data & 0x07);
                    self.set_course_x(pins.cpu_data >> 3);
                } else {
                    self.set_fine_y(pins.cpu_data & 0x07);
                    self.set_course_y(pins.cpu_data >> 3);
                }
                self.w_register = !self.w_register;
            }
            6 => {
                if !self.w_register {
                    self.temp_address = (self.temp_address & 0x00FF) | (((pins.cpu_data & 0x3F) as u16) << 8);
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
                        self.frame_palette_memory[palette_address as usize]
                    } else {
                        self.internal_read_buffer
                    };
                    // tell ppu to read from addr
                    self.vram_manip = VRamManip::Read;
                } else {
                    // tell ppu to write to addr
                    if (0x3F00..=0x3FFF).contains(&self.vram_address) {
                        let palette_address = (self.vram_address - 0x3F00) % 0x20;
                        self.frame_palette_memory[palette_address as usize] = pins.cpu_data;
                    } else {
                        self.vram_data = pins.cpu_data;
                        self.vram_manip = VRamManip::Write;
                    };

                }
                pins.ppu_address_high = (self.vram_address >> 8) as u8;
                pins.ppu_address_data_low = self.vram_address as u8;
                pins.ppu_ale = true;
                self.vram_address = self.vram_address.wrapping_add(self.vram_address_increment() as u16);
            }
        }
    }

    // -- Rendering helper functions --
    fn increment_x(&mut self) {
        if self.vram_address & 0x001F == 31 {
            self.vram_address &= !0x001F;
            self.vram_address ^= 0x0400;
        } else {
            self.vram_address = self.vram_address.wrapping_add(1);
        }
    }

    fn increment_y(&mut self) {
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
            self.vram_address = (self.vram_address & (!0x03E0)) | (y << 5);   // put coarse Y back into v
        }
    }

    // -- Rendering timing functions --
    fn is_render_fetch_cycle(&self) -> bool {
        ((1..257).contains(&self.cycle) || (321..337).contains(&self.cycle)) && ((0..240).contains(&self.scanline) || self.scanline == 261)
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

    fn is_rendering_enabled(&self) -> bool {
        self.enabled_background_rendering() || self.enabled_sprite_rendering()
    }

    // -- loopy address helpers --
    fn set_course_x(&mut self, b: u8) {
        const COURSE_X_MASK: u16 = 0x001F;
        self.temp_address = (self.temp_address & !COURSE_X_MASK) | (u16::from(b) & COURSE_X_MASK);
    }
    fn set_fine_x(&mut self, b: u8) {
        self.fine_x_scroll = b;
    }
    fn get_fine_x(&self) -> u8 {
        self.fine_x_scroll
    }
    fn set_course_y(&mut self, b: u8) {
        const COURSE_Y_MASK: u16 = 0b0000001111100000;
        self.temp_address = (self.temp_address & !COURSE_Y_MASK) | ((u16::from(b) & COURSE_Y_MASK) << 5);
    }
    fn set_fine_y(&mut self, b: u8) {
        const FINE_Y_MASK: u16 = 0b0111000000000000;
        self.temp_address = (self.temp_address & !FINE_Y_MASK) | ((u16::from(b) & FINE_Y_MASK) << 12);
    }

    // -- Register accessors & mutators --
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
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}
