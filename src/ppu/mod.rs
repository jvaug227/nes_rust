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

struct PixelBuffer {
    data: [u8; 256],
}
impl PixelBuffer {
    fn new() -> Self {
        Self {
            data: [0; 256],
        }
    }
    fn set(&mut self, x: u8, lsb: u8, msb: u8, priority: u8, palette: u8, sprite_0: bool) {
        for i in 0..8 {
            let index = x.wrapping_add(i) as usize;
            let lsb = lsb >> (7-i) & 1;
            let msb = msb >> (7-i) & 1;
            let data = lsb | (msb << 1);
            let existing_data_transparent = self.data[index] & 3 == 0;
            if existing_data_transparent {
                self.data[index] = data | (palette << 2) | priority << 7 | (u8::from(sprite_0) << 6);
            }
        }
    }
    fn get(&self, pixel: u8) -> (u8, u8, bool) {
        let data = self.data[pixel as usize];
        (data & 15, data >> 7, (data & 0x40) > 0)
    }
    fn clear(&mut self) {
        const FILL: u8 = 0x0;
        self.data.fill(FILL);
    }
}

#[derive(Clone, Copy)]
struct EvaluatedSprite {
    x: u8,
    attrib: u8,
    /// Tile will serve two purposes here:
    /// A: Will be the tile index in the relevant character table before fetch
    /// B: Will store the lsb and msb of the tile after fetch
    tile: u16,
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
    fine_x_scroll: u8,

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


    scanline: usize,
    cycle: usize,
    is_odd_frame: bool,
    internal_read_buffer: u8,

    oam_memory: [u8; 4 * 64],
    secondary_oam_buffer: [Option<EvaluatedSprite>; 8],
    oam_pixel_buffer: PixelBuffer,
    secondary_oam_buffer_count: usize,

    frame_palette_memory: [u8; 32],
    system_palette_memory: [u8; 64 * 3],

    /// TODO: remove from the ppu
    /// Treat ppu as a pixel-generating function
    video_data: Vec<u8>,
}


impl Ppu {
    pub fn dump(&self) {
        println!("\n=====PPU DUMP=====");
        // println!("v-address: {:0>4X}", self.vram_address);
        // println!("t-address: {:0>4X}", self.temp_address);
        // println!("w-latch: {}", self.w_register);
        // println!("cycle: {}, scanline: {}", self.cycle, self.scanline);
        // println!("MSB Shift register: {:0>16b}", self.tile_msb_scroll.0);
        // println!("LSB Shift register: {:0>16b}", self.tile_lsb_scroll.0);
        // println!("Next Nametable: {:0>2X}", self.next_nametable);
        // println!("Next MSB byte: {:0>2X}", self.next_tile_msb);
        // println!("Next LSB byte: {:0>2X}", self.next_tile_lsb);
        // println!("Next Attribute: {:0>2X}", self.next_attribute);
        for i in 0..64 {
            let i2 = i * 4;
            println!("OAM {i:0>2}: {3:0>3},{0:0>3} {1:0>2X}:{2:0>8b}", self.oam_memory[i2], self.oam_memory[i2+1], self.oam_memory[i2+2], self.oam_memory[i2+3]);

        }
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
            secondary_oam_buffer: [None; 8],
            oam_pixel_buffer: PixelBuffer::new(),
            secondary_oam_buffer_count: 0,
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

    fn shift(&mut self) {
        if self.enabled_background_rendering() {
            self.tile_msb_scroll.shift();
            self.tile_lsb_scroll.shift();
            self.attribute_msb_scroll.shift();
            self.attribute_lsb_scroll.shift();
        }
    } 

    fn render(&mut self, pins: &mut PpuPinout) {
        if self.is_render_fetch_cycle() && self.is_rendering_enabled() {
            if let Some(address) = self.render_fetch() {
                pins.ppu_address_data_low = address as u8;
                pins.ppu_address_high = (address >> 8) as u8;
                pins.ppu_ale = true;
                self.vram_manip = VRamManip::Read;
            }
        }
        if self.is_sprite_fetch_cycle() && self.is_rendering_enabled() {
            if let Some(address) = self.sprite_fetch() {
                pins.ppu_address_data_low = address as u8;
                pins.ppu_address_high = (address >> 8) as u8;
                pins.ppu_ale = true;
                self.vram_manip = VRamManip::Read;
            }
        }
        if (self.cycle > 0 && self.cycle < 257) || (self.cycle >= 321 && self.cycle < 337) {
            self.shift();
        }

        if self.is_rendering_enabled() {
            if self.is_sprite_evaluation_cycle() && self.scanline < 240 {
                if self.cycle == 65 {
                    // Restart evaluation for next scanline
                    self.oam_address_register = 0;
                    self.secondary_oam_buffer_count = 0;
                    self.secondary_oam_buffer.fill(None);
                    self.set_sprite_overflow(false);
                }
                let sprite = self.evaluate_sprite();
                self.oam_address_register = self.oam_address_register.wrapping_add(4);
                // println!("Incrementing OAM by evaluation");
                if sprite.is_some() {
                    if self.secondary_oam_buffer_count < 8 {
                        // Push evaluated sprites into a back-buffer so as to not replace sprites
                        // currently in processing
                        self.secondary_oam_buffer[self.secondary_oam_buffer_count] = sprite;
                        self.secondary_oam_buffer_count += 1;
                    } else {
                        // TODO: Emulate buggy overflow behavior for more than 8 sprites on a scanline
                        self.set_sprite_overflow(true);
                    }
                }
            }

            if self.cycle == 260 {
                self.oam_pixel_buffer.clear();
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
                let (sprite_palette_index, sprite_priority, sprite_opaque, sprite_0) = if self.enabled_sprite_rendering() {
                    let (pixel, priority, sprite_0) = self.oam_pixel_buffer.get(self.cycle as u8);
                    (pixel as usize, priority as usize, (pixel & 3) > 0, sprite_0)
                } else { (0, 0, false, false) };
                
                let (bg_palette_index, bg_opaque) =  if self.enabled_background_rendering() {
                    let offset = self.get_fine_x();
                    let tile_msb = self.tile_msb_scroll.get(offset) as u8;
                    let tile_lsb = self.tile_lsb_scroll.get(offset) as u8;

                    let attribute_msb = self.attribute_msb_scroll.get(offset) as u8;
                    let attribute_lsb = self.attribute_lsb_scroll.get(offset) as u8;

                    // 2-bit value selecting index inside the palette
                    let pixel_value = usize::from((tile_msb << 1) | tile_lsb);
                    // 2-bit value selecting which palette
                    let attribute_value = usize::from((attribute_msb<<1) | attribute_lsb);
                    (pixel_value | (attribute_value << 2), pixel_value > 0)
                } else { (0, false) };

                if self.enabled_background_rendering() && self.enabled_sprite_rendering() && sprite_0 && sprite_opaque && bg_opaque && !self.get_sprite_hit() {
                    self.set_sprite_hit();
                }
                
                let calculate_winning_pixel = |bg_opaque: usize, sprite_opaque: usize, priority: usize| -> usize {
                    let idx = bg_opaque | (sprite_opaque << 1);
                    let is_flippable = bg_opaque & sprite_opaque;
                    // When both bits are 1's, we flip one of them off based on priority
                    // Priority 0: we flip the bottom bit off
                    // Priority 1: we flip the top bit off
                    idx ^ (is_flippable << priority)
                };

                let pixels = [0, bg_palette_index, sprite_palette_index];
                // 1-bit value selecting which half (bg/sprite) of frame palette ram to access
                let winning_pixel = calculate_winning_pixel(bg_opaque as usize, sprite_opaque as usize, sprite_priority);
                let winning_page = (winning_pixel / 2) << 4;
                
                let system_palette_index = self.get_frame_palette(winning_page | pixels[winning_pixel]) as usize;
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
            self.clear_sprite_hit();
            self.is_odd_frame = !self.is_odd_frame;
        }

        pins.finished_frame = self.scanline == 261 && self.cycle == 340;
        if pins.finished_frame && self.is_rendering_enabled() {
            self.vram_address = self.temp_address;
        }

        self.cycle = self.cycle.wrapping_add(1) % DOTS_PER_SCANLINE;
        if self.cycle == 0 { self.scanline = self.scanline.wrapping_add(1) % SCANLINES_PER_FRAME; }
    }

    fn render_fetch(&mut self) -> Option<u16> {

        let v = self.vram_address;
        let v_cycle = self.cycle - 1;
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

    /// TODO: Determine if sprite 7 will get it's cycle 0
    fn sprite_fetch(&mut self) -> Option<u16> {
        let relative_cycle = self.cycle - 257;
        let c = relative_cycle % 8;
        let sprite_index = (relative_cycle.wrapping_sub(1) >> 3) & 7;
        if let Some(sprite) = &mut self.secondary_oam_buffer[sprite_index] {
            let addr = sprite.tile;
            let flip_horizontally = (sprite.attrib & 0x40) > 0;
            let data = if flip_horizontally { self.internal_read_buffer.reverse_bits() } else { self.internal_read_buffer } as u16;
            match c {
                0 => {
                    // fetch msb byte? determine if a fetch should occur since cycle 0 will not
                    // have a read value before this. First msb fetch not until cycle 7
                    // INFO: This may have been solved by skipping the first dummy cycle that would
                    // have read the msb
                    self.oam_pixel_buffer.set(sprite.x, sprite.tile as u8, data as u8, ((sprite.attrib & 0x20) > 0) as u8, sprite.attrib & 3, (sprite.attrib & 4) > 0);
                    // sprite.tile = (sprite.tile & 0x00FF) | (data << 8);
                    None
                }
                4 => {
                    // Calculate lsb address
                    Some(addr)
                }
                6 => {
                    // store lsb, msb is not here yet
                    sprite.tile = data;
                    // Calculate msb address
                    Some(addr | 0x08)
                }
                _ => { None }
            }
        } else {
            None
        }
        
    }

    fn evaluate_sprite(&mut self) -> Option<EvaluatedSprite> {
        // Starting from cycle 65, each sprite takes 3 cycles each
        let c = (self.cycle - 65) % 3;
        let s = (self.scanline) as u8;

        // Evaluate only on the last cycle of each sprite,
        // no need to perfectly emulate the evaluation in over multiple 
        // cycles as there is no outside bus interaction
        if c == 2 {
            let sprite = self.oam_address_register as usize;
            let sprite_0_flag = u8::from(sprite == 0) << 2;
            // byte 0 - y position of top of sprite
            let y = self.oam_memory[sprite];
            // byte 1 - tile index:
            //  8x8 tile - 8-bit index + pattern table bit in control register
            //  8x16 tile - 1 bit table + 7-bit (ignores the pattern bit in control register, uses
            //  the bit in the byte). This works because the tiles are consecutive and byte 255 can
            //  only be used in this case if the tile address is 254.
            let tile = self.oam_memory[sprite+1];
            // byte 2 - attributes
            //  2-bit palette
            //  3-bit unused
            //  1-bit priority
            //  1-bit flip horizontally
            //  1-bit flip vertically
            // Insert sprite-0 into unused field at bit 2
            let attrib = self.oam_memory[sprite+2] | sprite_0_flag;
            // byte 3 - left x position
            let x = self.oam_memory[sprite+3];

            let (table, tile, sprite_height) = if self.sprite_size() { (tile & 1, tile >> 1, 16) } else { (self.sprite_pattern_table(), tile, 8) };

            let flip_vertically = (attrib & 0x80) > 0;
            let y_s = s.wrapping_sub(y);
            // TODO: find way to not need to compare y < 240
            // Sprites stored off-screen at 255 are wrapping around in the comparison and comparing at 0
            if y < 240 && y_s < sprite_height {
                // we got a hit!
                let y_t = if flip_vertically { 7 - y_s } else { y_s } as u16;
                let tile = ((table as u16) << 12) | ((tile as u16) << 4) | y_t;
                // println!("Matched sprite {:0>2X} in table {} on scanline {} for y {} ({}), fetching data from {:0>4X}", sprite >> 2, table, self.scanline, y_s, y, tile);
                return Some(EvaluatedSprite {
                    x,
                    attrib,
                    tile,
                });
            }
        }
        None
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
                // println!("Cpu Reading status register: {:0>8b} during scanline {}, rendering enabled: {}", self.status_register, self.scanline, self.is_rendering_enabled());
                pins.cpu_data = self.status_register;
                self.set_vblank_flag(false);
                self.w_register = false;
            }
            3 => {
                self.oam_address_register = pins.cpu_data;
            }
            4 => {
                if pins.cpu_rw {
                    println!("Reading OAM");
                    pins.cpu_data = self.oam_memory[self.oam_address_register as usize];
                } else {
                    self.oam_memory[self.oam_address_register as usize] = pins.cpu_data;
                    self.oam_address_register = self.oam_address_register.wrapping_add(1);
                }
            }
            5 => {
                if !self.w_register {
                    // println!("Writing to x scroll: {}, {} on scanline {}", pins.cpu_data & 0x07, pins.cpu_data >> 3, self.scanline);
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
                    self.temp_address = ((pins.cpu_data & 0x3F) as u16) << 8;
                } else {
                    self.temp_address = (self.temp_address & 0xFF00) | (pins.cpu_data as u16);
                    self.vram_address = self.temp_address;
                    // println!("Writing {} to vram address", self.temp_address);
                }
                self.w_register = !self.w_register;
            }
            _ => { // assume 7
                if pins.cpu_rw {
                    pins.cpu_data = if (0x3F00..=0x3FFF).contains(&self.vram_address) {
                        let palette_address = (self.vram_address - 0x3F00) % 0x20;
                        self.get_frame_palette(palette_address as usize)
                    } else {
                        self.internal_read_buffer
                    };
                    // tell ppu to read from addr
                    self.vram_manip = VRamManip::Read;
                } else {
                    // tell ppu to write to addr
                    if (0x3F00..=0x3FFF).contains(&self.vram_address) {
                        let palette_address = (self.vram_address - 0x3F00) % 0x20;
                        self.set_frame_palette(palette_address as usize, pins.cpu_data);
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

    fn is_fetch_scanline(&self) -> bool {
        (0..240).contains(&self.scanline) || self.scanline == 261
    }

    // -- Rendering timing functions --
    fn is_render_fetch_cycle(&self) -> bool {
        ((1..257).contains(&self.cycle) || (321..337).contains(&self.cycle)) && self.is_fetch_scanline()
    }

    fn is_render_cycle(&self) -> bool {
        (0..240).contains(&self.scanline) && (0..256).contains(&self.cycle)
    }

    fn is_sprite_evaluation_cycle(&self) -> bool {
        (65..257).contains(&self.cycle)
    }

    /// This starts at cycle 258 to account for the fact that the MSB byte is *not* addressed
    /// before the first cycle of this period
    fn is_sprite_fetch_cycle(&self) -> bool {
        (258..322).contains(&self.cycle) && self.is_fetch_scanline()
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
    fn set_sprite_hit(&mut self) {
        self.status_register = (self.status_register & 0b10111111) | (1 << 6)
    }
    fn get_sprite_hit(&mut self) -> bool {
        (self.status_register & 0b01000000) > 0
    }
    fn clear_sprite_hit(&mut self) {
        self.status_register &= 0b10111111
    }
    fn set_vblank_flag(&mut self, status: bool) {
        // println!("Setting vblank to {}", status);
        self.status_register = (self.status_register & 0b01111111) | ((status as u8) << 7)
    }

    fn get_frame_palette(&self, index: usize) -> u8 {
        self.frame_palette_memory[index]
    }
    fn set_frame_palette(&mut self, index: usize, value: u8) {
        self.frame_palette_memory[index] = value;
        // if this is entry 0 of the palette, also write to entry 0 of the opposite palette (bg ->
        // sprite or sprite -> bg; selected by bit 5, or 0x10)
        // Yes, this does mean that there will always be a double write
        self.frame_palette_memory[index ^ (0x10 * usize::from(index & 3 == 0))] = value;
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}
