use anyhow::{anyhow, Result};
use egui::{Color32, Pos2, RichText, TextureId, Ui};
use nes::NESBoard;
use nes_rust::{cartidge::CartridgeData, cpu::*};
use std::sync::{
    atomic::{AtomicU16, AtomicU32, AtomicU8, AtomicUsize},
    Arc,
};
use wgpu::Backends;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::{Window, WindowAttributes},
};

mod nes;

// fn draw_ram(ui: & mut Ui, ram: &[u8], addr: u16, rows: u32, cols: usize) {
//     ui.vertical_centered_justified(|ui| {
//         for row in 0..rows {
//             let row_addr = addr as usize + (cols * row as usize);
//             let end = row_addr + cols;
//
//             ui.label(format!("\t${:04X?}:\t{:02X?}", row_addr, &ram[row_addr..end]));
//         }
//     });
//
// }

fn draw_cpu_flag(ui: &mut Ui, value: u8, text: &str) {
    let color = if value > 0 {
        Color32::GREEN
    } else {
        Color32::RED
    };
    ui.label(RichText::new(text).color(color));
}

fn draw_cpu(ui: &mut Ui, cpu: &Cpu) {
    ui.label("Status:");
    ui.horizontal(|ui: &mut Ui| {
        let status = cpu.get_status().bits();
        draw_cpu_flag(ui, status & 0x80, "N");
        draw_cpu_flag(ui, status & 0x40, "V");
        draw_cpu_flag(ui, status & 0x20, "U");
        draw_cpu_flag(ui, status & 0x10, "B");
        draw_cpu_flag(ui, status & 0x08, "D");
        draw_cpu_flag(ui, status & 0x04, "I");
        draw_cpu_flag(ui, status & 0x02, "Z");
        draw_cpu_flag(ui, status & 0x01, "C");
    });
    ui.horizontal(|ui: &mut Ui| {
        ui.label(RichText::new(format!("PC: ${:#02x}", cpu.pc)));
        ui.label(RichText::new(format!("A: ${:#x}", cpu.a)));
        ui.label(RichText::new(format!("X: ${:#x}", cpu.x)));
        ui.label(RichText::new(format!("Y: ${:#x}", cpu.y)));
    });
    ui.horizontal(|ui: &mut Ui| {
        ui.label(RichText::new(format!("Stack Ptr: ${:#x}", cpu.stkpt)));
        ui.label(RichText::new(format!("Fetched: ${:#x}", cpu.fetched)));
        ui.label(RichText::new(format!("Addr_data: ${:#x}", cpu.addr_data)));
    });
    ui.horizontal(|ui: &mut Ui| {
        ui.label(RichText::new(format!("Opcode: ${:#x}", cpu.opcode)));
        ui.label(RichText::new(format!(
            "Pipeline Status: ${:#x?}",
            cpu.pipeline_status
        )));
    });
}

struct Gpu {
    queue: wgpu::Queue,
    device: wgpu::Device,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    window: Arc<Window>,
}

impl Gpu {
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    pub fn surface(&self) -> &wgpu::Surface<'_> {
        &self.surface
    }
    pub fn surface_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.surface_config
    }
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }
    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}

pub struct EguiIntegrator {
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

impl EguiIntegrator {
    pub(crate) fn new(gpu: &Gpu) -> Self {
        let viewport_id = egui::ViewportId::default();
        let context = egui::Context::default();
        Self {
            state: egui_winit::State::new(context, viewport_id, &gpu.window, None, None, None),
            renderer: egui_wgpu::Renderer::new(gpu.device(), gpu.surface_format(), None, 1, false),
        }
    }

    pub fn on_event(
        &mut self,
        window: &Window,
        event: &winit::event::WindowEvent,
    ) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn state(&self) -> &egui_winit::State {
        &self.state
    }
    pub fn state_mut(&mut self) -> &mut egui_winit::State {
        &mut self.state
    }
    pub fn renderer(&self) -> &egui_wgpu::Renderer {
        &self.renderer
    }
    pub fn renderer_mut(&mut self) -> &mut egui_wgpu::Renderer {
        &mut self.renderer
    }
}

struct App {
    nes: NESBoard,
    egui: EguiIntegrator,
    frame_texture: wgpu::Texture,
    frame_texture_id: TextureId,
    _pattern_table_1_texture: wgpu::Texture,
    pattern_table_1_texture_id: TextureId,
    _pattern_table_2_texture: wgpu::Texture,
    pattern_table_2_texture_id: TextureId,
    nametable_texture: wgpu::Texture,
    nametable_texture_id: TextureId,
    nametable_texture_buffer: Vec<u8>,
    clock_cpu: bool,
    run_frame: bool,
    last_time: std::time::Instant,
    frame_time_start: std::time::Instant,
    frame_time_end: std::time::Instant,
    stream: cpal::Stream,
    freq: Arc<AtomicU32>,
    volume: Arc<AtomicU16>,
    sound_fn: Arc<AtomicU8>,
    h: Arc<AtomicUsize>,
    fcycle: Arc<AtomicU32>,
    // At bottom to maintain destroy order
    gpu: Gpu,
}

impl App {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let cpu = Cpu::new();

        use cpal::traits::DeviceTrait;
        use cpal::traits::HostTrait;
        use cpal::traits::StreamTrait;
        use cpal::Sample;
        let host = cpal::default_host();
        let maybe_device = HostTrait::default_output_device(&host);
        let device = maybe_device.expect("I dont care, gimme an output device");
        let mut supported_config_range = DeviceTrait::supported_output_configs(&device)
            .expect("I dont care, gimme a config range");
        let supported_config = supported_config_range
            .next()
            .expect("I dont care, gimme a config")
            .with_sample_rate(cpal::SampleRate(44100));
        let supported_config = supported_config.into();
        let freq = Arc::new(AtomicU32::new(440));
        let sound_fn = Arc::new(AtomicU8::new(0));
        let volume = Arc::new(AtomicU16::new(5));
        let h = Arc::new(AtomicUsize::new(2));
        let fcycle = Arc::new(AtomicU32::new(60));
        let sample_rate = 44100.0;
        let mut sample_index = 0.0;
        let stream = device
            .build_output_stream(
                &supported_config,
                {
                    let freq = freq.clone();
                    let volume = volume.clone();
                    let sound_fn = sound_fn.clone();
                    let h = h.clone();
                    let fcycle = fcycle.clone();
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        fn sample_sin(freq: f64, time: f64) -> f64 {
                            f64::sin(std::f64::consts::TAU * freq * time)
                        }
                        fn sample_square(freq: f64, time: f64, h: usize, f_cycle: f64) -> f64 {
                            let mut a = 0.0;
                            let mut b = 0.0;
                            let p = std::f64::consts::TAU * f_cycle;
                            for n in 1..h {
                                let f_n = n as f64;
                                let c = f_n * freq * std::f64::consts::TAU * time;
                                a += f64::sin(c) / f_n;
                                b += f64::sin((c - p) * f_n) / f_n;
                            }
                            std::f64::consts::FRAC_2_PI * (a - b)
                        }
                        // let time: f64 = std::time::Instant::now().duration_since(initial_timestamp).as_secs_f64();
                        let volume: f64 = (1.0
                            + f64::from(volume.load(std::sync::atomic::Ordering::Relaxed))
                                / 1000.0)
                            .log2();
                        let freq: f64 = freq.load(std::sync::atomic::Ordering::Relaxed).into();
                        let sound_fn: u8 = sound_fn.load(std::sync::atomic::Ordering::Relaxed);
                        let h: usize = h.load(std::sync::atomic::Ordering::Relaxed);
                        let fcycle: f64 =
                            (fcycle.load(std::sync::atomic::Ordering::Relaxed) as f64) / 1000.0;
                        for sample in data.iter_mut() {
                            let time = sample_index / sample_rate;
                            let raw_sample = volume
                                * match sound_fn {
                                    0 => sample_sin(freq, time),
                                    _ => sample_square(freq, time, h, fcycle),
                                };
                            *sample = raw_sample.to_sample::<f32>();
                            sample_index = (sample_index + 1.0) % sample_rate;
                        }
                    }
                },
                move |err| {
                    eprintln!("{err}");
                },
                None,
            )
            .unwrap();
        stream.play().unwrap();

        let program_path = {
            let mut args = std::env::args();
            _ = args.next();
            args.next().expect("Needs a rom path")
        };
        println!("Reading from file: {}", program_path);
        let program = std::fs::read(program_path).expect("A valid path to a rom must be provided");
        let cartridge_data = CartridgeData::decode(&program);
        println!("Read Catridge: (Maybe Named) {:?}", cartridge_data.title);
        println!("Program is {} bytes", program.len());
        println!(
            "Trainer Block: {:?} at {} bytes",
            cartridge_data.trainer_range,
            cartridge_data
                .trainer_range
                .clone()
                .map(|r| r.len())
                .unwrap_or(0)
        );
        println!(
            "Program Rom Block: {:?} at {} bytes",
            cartridge_data.prg_rom_range,
            cartridge_data.prg_rom_range.len()
        );
        println!(
            "Character Rom Block: {:?} at {} bytes",
            cartridge_data.chr_rom_range,
            cartridge_data
                .chr_rom_range
                .clone()
                .map(|r| r.len())
                .unwrap_or(0)
        );
        println!("Mapper: {}", cartridge_data.mapper);

        // const RAM_SIZE: usize = 256 * 2048;
        // const PROGRAM_RANGE: usize = 32768;
        let internal_ram = vec![0u8; 2048];
        let internal_vram = vec![0u8; 2048];

        let program_rom = program[cartridge_data.prg_rom_range.clone()].to_vec();
        let character_rom = cartridge_data
            .chr_rom_range
            .clone()
            .map(|range| program[range].to_vec())
            .unwrap_or_default();
        let program_ram_size = 0;

        let nes = NESBoard::new(
            cpu,
            internal_ram,
            internal_vram,
            program_rom,
            character_rom,
            program_ram_size,
        );
        let gpu = pollster::block_on(App::create_gpu_struct(event_loop)).unwrap();

        let mut egui = EguiIntegrator::new(&gpu);

        let frame_texture = gpu.device().create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 256,
                height: 240,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("NES_video_output"),
            view_formats: &[],
        });
        let frame_texture_view = frame_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let frame_texture_id = egui.renderer_mut().register_native_texture(
            gpu.device(),
            &frame_texture_view,
            wgpu::FilterMode::Nearest,
        );

        let pattern_table_descriptor = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("NES_video_output"),
            view_formats: &[],
        };
        let pattern_table_1_texture = gpu.device().create_texture(&pattern_table_descriptor);
        let pattern_table_1_texture_view =
            pattern_table_1_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let pattern_table_1_texture_id = egui.renderer_mut().register_native_texture(
            gpu.device(),
            &pattern_table_1_texture_view,
            wgpu::FilterMode::Nearest,
        );

        let pattern_table_2_texture = gpu.device().create_texture(&pattern_table_descriptor);
        let pattern_table_2_texture_view =
            pattern_table_2_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let pattern_table_2_texture_id = egui.renderer_mut().register_native_texture(
            gpu.device(),
            &pattern_table_2_texture_view,
            wgpu::FilterMode::Nearest,
        );
        Self::upload_nes_pattern_table_textures(
            &gpu,
            &nes,
            &pattern_table_1_texture,
            &pattern_table_2_texture,
        );

        let nametable_texture = gpu.device().create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 32,
                height: 30,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("NES_video_output"),
            view_formats: &[],
        });
        let nametable_texture_view =
            nametable_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let nametable_texture_id = egui.renderer_mut().register_native_texture(
            gpu.device(),
            &nametable_texture_view,
            wgpu::FilterMode::Nearest,
        );
        Self {
            nes,
            gpu,
            egui,
            frame_texture_id,
            frame_texture,
            _pattern_table_1_texture: pattern_table_1_texture,
            pattern_table_1_texture_id,
            _pattern_table_2_texture: pattern_table_2_texture,
            pattern_table_2_texture_id,
            nametable_texture,
            nametable_texture_id,
            nametable_texture_buffer: vec![255; 4 * 32 * 30],
            clock_cpu: false,
            run_frame: false,
            last_time: std::time::Instant::now(),
            frame_time_start: std::time::Instant::now(),
            frame_time_end: std::time::Instant::now(),

            stream,
            freq,
            volume,
            sound_fn,
            h,
            fcycle,
        }
    }

    async fn create_gpu_struct(event_loop: &winit::event_loop::ActiveEventLoop) -> Result<Gpu> {
        let window = Arc::new(event_loop.create_window(WindowAttributes::default())?);
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone())?;

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
        {
            Ok(adapter) => adapter,
            Err(e) => return Err(anyhow!("No Available Adapters: {e}")),
        };

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                trace: wgpu::Trace::Off,
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            })
            .await?;

        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let surface_config = match surface.get_default_config(&adapter, size.width, size.height) {
            Some(config) => config,
            None => return Err(anyhow!("No formats or present mode available for surface!")),
        };
        surface.configure(&device, &surface_config);
        let gpu = Gpu {
            window,
            surface,
            surface_config,
            device,
            queue,
        };
        Ok(gpu)
    }

    fn upload_nes_nametable_texture(
        gpu: &Gpu,
        nes: &NESBoard,
        nametable_texture: &wgpu::Texture,
        nametable_texture_buffer: &mut [u8],
    ) {
        let nametable = nes.nametable_memory(0);
        for (i, b) in nametable.iter().enumerate().take(32 * 30) {
            let i = i * 4;
            nametable_texture_buffer[i] = *b & 0xf0;
            nametable_texture_buffer[i + 1] = *b << 4;
            nametable_texture_buffer[i + 2] = 0x00;
            nametable_texture_buffer[i + 3] = 0xff;
        }
        gpu.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: nametable_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            nametable_texture_buffer,
            // The layout of the texture
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * 32),
                rows_per_image: Some(30),
            },
            wgpu::Extent3d {
                width: 32,
                height: 30,
                depth_or_array_layers: 1,
            },
        );
    }

    fn upload_nes_pattern_table_textures(
        gpu: &Gpu,
        nes: &NESBoard,
        pattern_table_1_texture: &wgpu::Texture,
        pattern_table_2_texture: &wgpu::Texture,
    ) {
        let mut pt1_buff = vec![0xff; 128 * 128 * 4];
        let mut pt2_buff = vec![0xff; 128 * 128 * 4];
        let pattern_table = nes.pattern_table_memory();
        let colors = [
            0xff, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0xff, 0x7f, 0x34, 0x00,
        ];

        let tiles_to_pixels = |buff: &mut [u8], pattern_table: &[u8]| {
            for py in 0..128 {
                for px in 0..128 {
                    let tx = px / 8; // [0, 16)
                    let ty = py / 8; // [0, 16)
                    let t = (ty * 16 + tx) * 16; // 16 bytes per tile
                    let to_l = py & 7;
                    let to_h = to_l + 8;
                    let bitmask = 0x80 >> (px & 7);
                    let t_l = ((pattern_table[t + to_l] & bitmask) > 0) as u8;
                    let t_h = ((pattern_table[t + to_h] & bitmask) > 0) as u8;

                    let color_index = usize::from(((t_h << 1) | t_l) * 3);
                    let r = colors[color_index];
                    let g = colors[color_index + 1];
                    let b = colors[color_index + 2];

                    let p = (py * 128 + px) * 4;
                    buff[p] = r;
                    buff[p + 1] = g;
                    buff[p + 2] = b;
                    buff[p + 3] = 0xff;
                }
            }
        };
        let half = pattern_table.len() / 2;
        tiles_to_pixels(&mut pt1_buff, &pattern_table[0..half]);
        tiles_to_pixels(&mut pt2_buff, &pattern_table[half..]);

        gpu.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: pattern_table_1_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &pt1_buff,
            // The layout of the texture
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * 128),
                rows_per_image: Some(128),
            },
            wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: pattern_table_2_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &pt2_buff,
            // The layout of the texture
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * 128),
                rows_per_image: Some(128),
            },
            wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            },
        );
    }

    fn draw(&mut self) -> Result<()> {
        let gpu = &self.gpu;
        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });
        gpu.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.frame_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            self.nes.video_memory(),
            // The layout of the texture
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * 256),
                rows_per_image: Some(240),
            },
            wgpu::Extent3d {
                width: 256,
                height: 240,
                depth_or_array_layers: 1,
            },
        );
        let output_frame = gpu.surface().get_current_texture()?;
        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin to draw the UI frame.
        let input = self.egui.state.take_egui_input(gpu.window());
        let ctx = self.egui.state.egui_ctx();
        ctx.begin_pass(input);
        egui::SidePanel::right("CPU").resizable(true).max_width(400.0).show(ctx,|ui| {

            ui.label("SPACE = Continuous Run    F = Run Frame Worth of Cycles    P = Step Cycle    O = Dump PPU Data");
            ui.label("W = UP    S = DOWN    A = LEFT    D = RIGHT    H = A    J = B    K = SELECT    L = START");
            draw_cpu(ui, self.nes.cpu());
            ui.separator();
            let frame_time = (self.frame_time_end - self.frame_time_start).as_secs_f64();
            let average_fps = 1.0 / frame_time;
            ui.label(RichText::new(format!("Frame time: {}", frame_time)));
            ui.label(RichText::new(format!("AVG FPS: {:.2}", average_fps)));
            ui.separator();
            if ui.button("RESET").clicked() { self.nes.reset(); }
            if ui.button("IRQ").clicked() { self.nes.irq(); }
            if ui.button("NMI").clicked() { self.nes.nmi(); }
            {
                let mut m_sound_fn = self.sound_fn.load(std::sync::atomic::Ordering::Relaxed);
                ui.add(egui::Slider::new(&mut m_sound_fn, 0..=4).text("Sound Fn"));
                self.sound_fn.store(m_sound_fn, std::sync::atomic::Ordering::Relaxed);

                let mut m_vol = self.volume.load(std::sync::atomic::Ordering::Relaxed);
                ui.add(egui::Slider::new(&mut m_vol, 0..=1000).text("Volume"));
                self.volume.store(m_vol, std::sync::atomic::Ordering::Relaxed);

                let mut m_freq = f64::from(self.freq.load(std::sync::atomic::Ordering::Relaxed));
                ui.add(egui::Slider::new(&mut m_freq, 0.0..=1000.0).text("Frequency"));
                self.freq.store(m_freq as u32, std::sync::atomic::Ordering::Relaxed);

                let mut m_h: usize = self.h.load(std::sync::atomic::Ordering::Relaxed);
                ui.add(egui::Slider::new(&mut m_h, 2..=255).text("H"));
                self.h.store(m_h, std::sync::atomic::Ordering::Relaxed);

                let mut m_fcycle = self.fcycle.load(std::sync::atomic::Ordering::Relaxed);
                ui.add(egui::Slider::new(&mut m_fcycle, 0..=1000).text("FCycle"));
                self.fcycle.store(m_fcycle, std::sync::atomic::Ordering::Relaxed);

            }
            ui.separator();
            ui.collapsing("Debug Information", |ui| {
                Self::upload_nes_nametable_texture(gpu, &self.nes, &self.nametable_texture, &mut self.nametable_texture_buffer);
                let draw_texture = |ui: &mut Ui, texture_id: TextureId, x: f32, y: f32, width: f32, height: f32| {
                    let bounding_box = egui::Rect { min: Pos2 {x, y}, max: Pos2 { x: x + width, y: y + height } };
                    let uv = egui::Rect { min: Pos2 {x: 0.0, y: 0.0}, max: Pos2 {x: 1.0, y: 1.0} };
                    ui.expand_to_include_rect(bounding_box);
                    ui.painter().image(texture_id, bounding_box, uv, Color32::WHITE);
                };
                let (x, y) = ui.next_widget_position().into();
                let width = 256.0;
                let height = 240.0;
                draw_texture(ui, self.nametable_texture_id, x, y, width, height);

                let y = y + height;
                let height = 256.0;

                draw_texture(ui, self.pattern_table_1_texture_id, x, y, width, height);

                let y = y + height;
                let height = 256.0;
                draw_texture(ui, self.pattern_table_2_texture_id, x, y, width, height);
            });

        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("NES (6502) Emulator");
            ui.painter().image(
                self.frame_texture_id,
                egui::Rect {
                    min: Pos2 { x: 0.0, y: 0.0 },
                    max: Pos2 {
                        x: 256.0 * 2.0,
                        y: 240.0 * 2.0,
                    },
                },
                egui::Rect {
                    min: Pos2 { x: 0.0, y: 0.0 },
                    max: Pos2 { x: 1.0, y: 1.0 },
                },
                Color32::WHITE,
            );
        });

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let output = ctx.end_pass();
        let paint_jobs = ctx.tessellate(output.shapes, output.pixels_per_point);

        self.egui
            .state
            .handle_platform_output(gpu.window(), output.platform_output);

        // Upload all resources for the GPU.
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [gpu.surface_config().width, gpu.surface_config().height],
            pixels_per_point: gpu.window().scale_factor() as f32,
        };

        for (tid, image_delta) in &output.textures_delta.set {
            self.egui
                .renderer_mut()
                .update_texture(gpu.device(), gpu.queue(), *tid, image_delta);
        }
        self.egui.renderer_mut().update_buffers(
            gpu.device(),
            gpu.queue(),
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            let mut render_pass = render_pass.forget_lifetime();
            self.egui
                .renderer()
                .render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        for id in &output.textures_delta.free {
            self.egui.renderer_mut().free_texture(id);
        }

        gpu.queue().submit(std::iter::once(encoder.finish()));
        gpu.window().pre_present_notify();
        output_frame.present();
        gpu.window().request_redraw();
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let resp = self.egui.state.on_window_event(&self.gpu.window, &event);
        if resp.repaint {
            self.gpu.window.request_redraw();
        }
        if resp.consumed {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let current_time = std::time::Instant::now();
                let do_frame = self.run_frame
                    || (self.clock_cpu
                        && (current_time - self.last_time)
                            > std::time::Duration::from_secs_f64(1.00 / 60.0));
                if do_frame {
                    self.last_time = current_time;
                    let ready = false;
                    self.frame_time_start = std::time::Instant::now();
                    // for _ in 0..29781 {
                    while !self.nes.clock(ready) {}
                    self.frame_time_end = std::time::Instant::now();
                    self.run_frame = false;
                }
                match self.draw() {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error: {e}");
                    }
                };
            }
            WindowEvent::Resized(winit::dpi::PhysicalSize { width, height }) => {
                self.gpu.resize(width, height);
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.repeat {
                    return;
                }
                let pressed = event.state == ElementState::Pressed;
                match event.key_without_modifiers().as_ref() {
                    Key::Character(s) => {
                        match s {
                            "p" if pressed => {
                                if !self.clock_cpu {
                                    self.nes.clock(false);
                                }
                            }
                            "o" if pressed => {
                                self.nes.dump_ppu();
                            }
                            "f" if pressed => {
                                self.run_frame = true;
                            }
                            // a
                            "l" => {
                                self.nes.set_controller_button(0, 7, pressed);
                            }
                            // b
                            "k" => {
                                self.nes.set_controller_button(0, 6, pressed);
                            }
                            // start
                            "j" => {
                                self.nes.set_controller_button(0, 5, pressed);
                            }
                            // select
                            "h" => {
                                self.nes.set_controller_button(0, 4, pressed);
                            }
                            // up
                            "w" => {
                                self.nes.set_controller_button(0, 3, pressed);
                            }
                            // down
                            "s" => {
                                self.nes.set_controller_button(0, 2, pressed);
                            }
                            // left
                            "a" => {
                                self.nes.set_controller_button(0, 1, pressed);
                            }
                            // right
                            "d" => {
                                self.nes.set_controller_button(0, 0, pressed);
                            }
                            _ => {}
                        }
                    }
                    Key::Named(n) => {
                        if n == NamedKey::Space && pressed {
                            self.clock_cpu = !self.clock_cpu;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

#[allow(dead_code)] // TODO: Handle suspended state
enum AppShell {
    Unintialized,
    Resumed(App),
    Suspended(App),
}

impl ApplicationHandler for AppShell {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        *self = match std::mem::take(self) {
            // Create a new application
            Self::Unintialized => Self::Resumed(App::new(event_loop)),
            // We are already a resumed application... do nothing?
            Self::Resumed(app) => Self::Resumed(app),
            // Return from a suspended state
            Self::Suspended(mut app) => {
                app.resumed(event_loop);
                Self::Resumed(app)
            }
        };
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Self::Resumed(app) = self {
            app.window_event(event_loop, window_id, event);
        } else {
            panic!("Attempted to do window event during an invalid state!");
        }
    }
}

impl Default for AppShell {
    fn default() -> Self {
        Self::Unintialized
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let mut app = AppShell::default();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    Ok(event_loop.run_app(&mut app)?)
}
