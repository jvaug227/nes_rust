use std::{error::Error, io::Write, sync::Arc};
use egui::{ Ui, Color32, RichText};
use nes_rust::{cpu::*, cartidge::CartridgeData };
use wgpu::Backends;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::EventLoop, keyboard::{Key, NamedKey},
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::{Window, WindowAttributes}
};
use anyhow::{anyhow, Result};

fn draw_ram(ui: & mut Ui, ram: &[u8], addr: u16, rows: u32, cols: usize) {
    ui.vertical_centered_justified(|ui| {
        for row in 0..rows {
            let row_addr = addr as usize + (cols * row as usize);
            let end = row_addr + cols;
            //let byte_vec: Vec<u8> = vec![];
            let a = &ram[row_addr..end];
            
            ui.label(format!("\t${:04X?}:\t{:02X?}", row_addr, a));
        }
    });
    
}

fn draw_cpu_flag(ui: &mut Ui, flag: Flags6502, cpu: &Cpu) {
    ui.label(RichText::new(flag.to_string()).color(if cpu.get_flag(flag) { Color32::GREEN } else { Color32::RED }));
}

fn draw_cpu(ui: & mut Ui, cpu: &Cpu) {
    
    ui.label("Status:");
    ui.horizontal(|ui: &mut Ui| {
        let flags = [Flags6502::C, Flags6502::Z, Flags6502::I, Flags6502::D, Flags6502::B, Flags6502::U, Flags6502::V, Flags6502::N];
        for flag in flags {
            draw_cpu_flag(ui, flag, cpu);
        }
    });
    ui.horizontal(|ui: &mut Ui| {
        ui.label(RichText::new(&format!("PC: ${:#02x}", cpu.pc)));
        ui.label(RichText::new(&format!("A: ${:#x}", cpu.a)));
        ui.label(RichText::new(&format!("X: ${:#x}", cpu.x)));
        ui.label(RichText::new(&format!("Y: ${:#x}", cpu.y)));
        ui.label(RichText::new(&format!("Stack Ptr: ${:#x}", cpu.stkpt)));
        ui.label(RichText::new(&format!("Fetched: ${:#x}", cpu.fetched)));
        ui.label(RichText::new(&format!("Addr_data: ${:#x}", cpu.addr_data)));
    });
    ui.horizontal(|ui: &mut Ui| {
        ui.label(RichText::new(&format!("Opcode: ${:#x}", cpu.opcode)));
        ui.label(RichText::new(&format!("Pipeline Status: ${:#x?}", cpu.pipeline_status)));
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
    pub fn surface(&self) -> &wgpu::Surface {
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

    pub fn on_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn state(&self) -> &egui_winit::State { &self.state }
    pub fn state_mut(&mut self) -> &mut egui_winit::State { &mut self.state }
    pub fn renderer(&self) -> &egui_wgpu::Renderer { &self.renderer }
    pub fn renderer_mut(&mut self) -> &mut egui_wgpu::Renderer { &mut self.renderer }
}
struct App {
    nes: NESBoard,
    egui: Option<EguiIntegrator>,
    gpu: Option<Gpu>,
    clock_cpu: bool,
    last_time: std::time::Instant,
}

impl App {
    fn new(cpu: Cpu, ram: Vec<u8>) -> Self {
        let nes = NESBoard::new(cpu, ram);
        Self {
            nes, gpu: None, egui: None, clock_cpu: false, last_time: std::time::Instant::now(),
        }
    }

    async fn create_gpu_struct(event_loop: &winit::event_loop::ActiveEventLoop) -> Result<Gpu> {
        let window = Arc::new(event_loop.create_window(WindowAttributes::default())?);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { backends: Backends::all(), ..Default::default() });
        let surface = instance.create_surface(window.clone())?;

        let adapter = match instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await {
            Some(adapter) => adapter,
            None => return Err(anyhow!("No Available Adapters!")),
        };

        let (device, queue) = adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
        ).await?;

        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        // let _surface_capabilities = surface.get_capabilities(&adapter);
        let surface_config = 
            match surface.get_default_config(&adapter, size.width, size.height) {
                Some(config) => config,
                None => return Err(anyhow!("No formats or present mode available for surface!")),
            };
        surface.configure(&device, &surface_config);
        let gpu = Gpu { window, surface, surface_config, device, queue };
        Ok(gpu)
    }

    fn draw(&mut self) -> Result<()> {

        let (Some(gpu), Some(egui), cpu, ram) = (self.gpu.as_ref(), self.egui.as_mut(), &self.nes.cpu, &self.nes.ram) else { return Err(anyhow!("Can't draw yet...")) };

        let mut encoder = gpu.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder"),
        });

        let output_frame = gpu.surface().get_current_texture()?;
        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin to draw the UI frame.
        let input = egui.state.take_egui_input(gpu.window());
        let ctx = egui.state.egui_ctx();
        ctx.begin_pass(input);
        egui::SidePanel::right("CPU").resizable(true).show(ctx,|ui| {

            ui.label("SPACE = Step Instruction    R = RESET    I = IRQ    N = NMI");
            draw_cpu(ui, cpu);
            ui.separator();
            // draw code here
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            //egui::SidePanel::left("RAM").show(&context, |ui| {

            ui.label("NES (6502) Emulator");

            draw_ram(ui, ram, 0x0000, 16, 16);
            ui.separator();
            draw_ram(ui, ram, 0xC000, 16, 16);
            ui.separator();
            draw_ram(ui, ram, 0xC500, 16, 16);
            ui.separator();
            draw_ram(ui, ram, 0xC700, 16, 16);
        });


        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let output = ctx.end_pass();
        let paint_jobs = ctx.tessellate(output.shapes, output.pixels_per_point);

        egui.state.handle_platform_output(gpu.window(), output.platform_output);

        // Upload all resources for the GPU.
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [gpu.surface_config().width, gpu.surface_config().height],
            pixels_per_point: gpu.window().scale_factor() as f32,
        };

        for (tid, image_delta) in &output.textures_delta.set {
            egui.renderer_mut().update_texture(gpu.device(), gpu.queue(), *tid, image_delta);
        }
        egui.renderer_mut().update_buffers(gpu.device(), gpu.queue(), &mut encoder, &paint_jobs, &screen_descriptor);

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
            egui.renderer().render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        for id in &output.textures_delta.free {
            egui.renderer_mut().free_texture(id);
        }

        gpu.queue().submit(std::iter::once(encoder.finish()));
        gpu.window().pre_present_notify();
        output_frame.present();
        gpu.window().request_redraw();
        Ok(())

        }
    }

    impl ApplicationHandler for App {
        fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            let gpu = self.gpu.get_or_insert_with(|| pollster::block_on(App::create_gpu_struct(event_loop)).unwrap());

            _ = self.egui.replace(EguiIntegrator::new(gpu));
        }

        fn window_event(
            &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let (Some(egui), Some(gpu)) = (self.egui.as_mut(), self.gpu.as_ref()) {
            let resp = egui.state.on_window_event(&gpu.window, &event);
            if resp.repaint { gpu.window.request_redraw(); }
            if resp.consumed { return; }
        }

        match event {
            WindowEvent::CloseRequested => { event_loop.exit(); },
            WindowEvent::RedrawRequested => {
                match self.draw() {
                    Ok(_) => {},
                    Err(e) => { eprintln!("Error: {e}"); }
                };
                let current_time = std::time::Instant::now();
                if self.clock_cpu && (current_time - self.last_time) > std::time::Duration::from_secs_f64(0.00) {
                    self.last_time = current_time;
                    self.nes.clock();
                    self.nes.clock();
                    self.nes.clock();
                }
            },
            WindowEvent::Resized(winit::dpi::PhysicalSize{ width, height }) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize( width, height );
                }
            },
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                if event.repeat { return; }
                if event.state != ElementState::Pressed { return; }
                match event.key_without_modifiers().as_ref() {
                    Key::Character(s) => {
                        match s {
                            "r" => { self.nes.cpu.reset(); },
                            "i" => { self.nes.cpu.irq(); },
                            "n" => { self.nes.cpu.nmi(); },
                            "p" => if !self.clock_cpu { self.nes.clock(); },
                            _ => {}
                        }
                    }
                    Key::Named(n) => {
                        if n == NamedKey::Space {
                            self.clock_cpu = !self.clock_cpu;
                        }
                    },
                    _ => {},
                }

            },
            _ => {}

        }

    }
}

struct NESBoard {
    cpu: Cpu,
    cpu_copy: Cpu,
    debug_buffer: Vec<u8>,
    ram: Vec<u8>,
    addr_rw: bool,
    addr_bus: u16,
    data_bus: u8,
}

impl NESBoard {
    fn new(cpu: Cpu, ram: Vec<u8>) -> NESBoard {
        let debug_buffer = b"XXXX  XX XX XX  XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX  A:XX X:XX Y:XX P:XX SP:XX PPU:XXX,XXX CYC:XXXXX\n".to_vec();
        let mut cpu_copy = cpu;
        cpu_copy.pc -= 1;
        NESBoard { ram, cpu, cpu_copy, debug_buffer, addr_rw: true, addr_bus: 0, data_bus: 0 }
    }

    // Emulate one master clok cycle
    fn clock(&mut self) {
        self.cpu.clock(&mut self.addr_bus, &mut self.data_bus, &mut self.addr_rw, false);
        if self.addr_rw {
            let addr = self.addr_bus as usize;
            self.data_bus = self.ram[addr];
        }
        if self.cpu.clock(&mut self.addr_bus, &mut self.data_bus, &mut self.addr_rw, true) {
            self.print_log();
            self.cpu_copy = self.cpu; // update the copy
            self.cpu_copy.pc -= 1; // pc increments before we can copy it
        }
        if !self.addr_rw {
            let addr = self.addr_bus as usize;
            self.ram[addr] = self.data_bus;
        }
    }

    fn write_hex_to_buffer(mut value: u16, buffer: &mut [u8], start: usize, digits: usize) {
        for digit in (0..digits).rev() {
            let a = b'0' + (value % 16) as u8;
            let a = if a > b'9' { a + 7 } else { a };
            value /= 16;
            buffer[start+digit] = a;
        }
    }

    fn write_decimal_to_buffer(mut value: usize, buffer: &mut [u8], start: usize, digits: usize) {
        for digit in (0..digits).rev() {
            let a = if value > 0 { b'0' + (value % 10) as u8 } else { b' ' };
            value /= 10;
            buffer[start+digit] = a;
        }
    }

    fn print_log(&mut self) {
        let byte1: u8 = 0;
        let byte2: u8 = 0;
        // let ins_str: String = stringify_ins_from_log(&self.cpu_copy);
        let cycles: usize = self.cpu_copy.cycles;
        let ppucycles = cycles * 3;
        let ppu1: usize = ppucycles / 340;
        let ppu2: usize = ppucycles % 340;
        // 0         1         2         3         4         5         6         7         8         9
        // 01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234
        //"XXXX  XX XX XX  XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX  A:XX X:XX Y:XX P:XX SP:XX PPU:XXX,XXX CYC:XXXXX"
        let buffer = &mut self.debug_buffer;
        Self::write_hex_to_buffer(self.cpu_copy.pc, buffer, 0, 4);
        Self::write_hex_to_buffer(self.cpu_copy.opcode as u16, buffer, 6, 2);
        Self::write_hex_to_buffer(byte1 as u16, buffer, 9, 2);
        Self::write_hex_to_buffer(byte2 as u16, buffer, 12, 2);
        Self::write_hex_to_buffer(self.cpu_copy.a as u16, buffer, 50, 2);
        Self::write_hex_to_buffer(self.cpu_copy.x as u16, buffer, 55, 2);
        Self::write_hex_to_buffer(self.cpu_copy.y as u16, buffer, 60, 2);
        Self::write_hex_to_buffer(self.cpu_copy.get_status().bits() as u16, buffer, 65, 2);
        Self::write_hex_to_buffer(self.cpu_copy.stkpt as u16, buffer, 71, 2);
        Self::write_decimal_to_buffer(ppu1, buffer, 78, 3);
        Self::write_decimal_to_buffer(ppu2, buffer, 82, 3);
        Self::write_decimal_to_buffer(cycles, buffer, 90, 5);
        let mut stdout = std::io::stdout().lock();
        let _e = stdout.write_all(&self.debug_buffer);
        let _e = stdout.flush();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    let mut cpu = Cpu::new();

    //let program = vec![0xA2, 0x0A, 0x8E, 0x00, 0x00, 0xA2, 0x03, 0x8E, 0x01, 0x00, 0xAC, 0x00, 0x00, 0xA9, 0x00, 0x18, 0x6D, 0x01, 0x00, 0x88, 0xD0, 0xFA, 0x8D, 0x02, 0x00, 0xEA, 0xEA, 0xEA];
    //let program = include_bytes!("official_only.nes");
    let program = include_bytes!("nestest.nes");
    let cartridge_data = CartridgeData::decode(program);
    println!("Read Catridge: (Maybe Named) {:?}", cartridge_data.title);
    println!("Program is {} bytes", program.len());
    println!("Trainer Block: {:?} at {} bytes", cartridge_data.trainer_range, cartridge_data.trainer_range.clone().map(|r| r.len()).unwrap_or(0));
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
        println!("Needs to mirror");
        for i in 0..mirror_count {
            let mirror_start = 0x8000 + mirror_length*i;
            let mirror_end = mirror_start + mirror_length;
            println!("Mirror {i} from {mirror_start:0>4X} to {mirror_end:0>4X}");
            ram[mirror_start .. mirror_end].copy_from_slice(program_range);   
        }
    } else {
        ram[0x8000 ..=0xFFFF].copy_from_slice(&program[cartridge_data.prg_rom_range.clone()]);
    }

    // cpu.bus_mut().ram[0xFFFA] = 0x00;
    // cpu.bus_mut().ram[0xFFFB] = 0x80;
    // cpu.bus_mut().ram[0xFFFC] = 0x00;
    // cpu.bus_mut().ram[0xFFFD] = 0x80;
    // cpu.bus_mut().ram[0xFFFE] = 0x00;
    // cpu.bus_mut().ram[0xFFFF] = 0x80;
    cpu.reset();
    cpu.pc = 0xC001;
    let a = ram[0xC000];
    cpu.opcode = a;
    cpu.cycles = 7;
    cpu.stkpt = 0xFD;
    cpu.set_flags(Flags6502::I | Flags6502::U);

    let mut app = App::new(cpu, ram);
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut app)?;

    Ok(())
}

