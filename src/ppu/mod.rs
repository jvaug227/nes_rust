pub struct PpuPinout {
    pub nmi: bool,
    pub cpu_rw: bool,
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
    oam_data_register: u8,
    scroll_register: u8,
    address_register: u8,
    data_register: u8,
    oam_dma_register: u8,

    cycle_count: usize,

    video_data: Vec<u8>,
}

// True size of Rendering Area
const DOTS_PER_SCANLINE: usize = 341;
// Contains pre-scan line
const SCANLINES_PER_FRAME: usize = 262;
// Actual drawn Region
const DOTS_PER_IMAGE_ROW: usize = 256;
const SCANLINES_PER_IMAGE: usize = 240;

pub const VIDEO_MEMORY_SIZE: usize = DOTS_PER_IMAGE_ROW * SCANLINES_PER_IMAGE * 4;
impl Ppu {
    pub fn new() -> Self {
        Self {
            control_register: 0,
            mask_register: 0,
            status_register: 0, 
            oam_address_register: 0,
            oam_data_register: 0,
            scroll_register: 0,
            address_register: 0,
            data_register: 0,
            oam_dma_register: 0,

            cycle_count: 0,

            video_data: vec![255; VIDEO_MEMORY_SIZE]
        }
    }

    pub fn clock(&mut self, pins: &mut PpuPinout) {
        pins.nmi = true;

        // let x = 0;
        // let y = 0;
        // let point = (y * 240 + x) * 4;
        let point = self.cycle_count * 4;
        self.video_data[point    ] = 255;
        self.video_data[point + 1] = 0;
        self.video_data[point + 2] = 127;
        self.video_data[point + 3] = 255;

        self.cycle_count = self.cycle_count.wrapping_add(1) % (DOTS_PER_IMAGE_ROW * SCANLINES_PER_IMAGE);
    }

    pub fn video_data(&self) -> &[u8] {
        &self.video_data
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}
