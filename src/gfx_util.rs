#![allow(clippy::redundant_field_names)]

extern crate pollster;
extern crate sdl2;
extern crate wgpu;
use egui_wgpu::ScreenDescriptor;
use sdl2::event::{Event, WindowEvent};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use std::time::Instant;

use egui::{Key, Modifiers, MouseWheelUnit, PointerButton, Pos2, RawInput, Rect};
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;
use sdl2::mouse::{Cursor, MouseButton, SystemCursor};
use sdl2::video::Window;

pub struct GfxContext<'a> {
    pub sdl_context: sdl2::Sdl,
    pub sdl_video: sdl2::VideoSubsystem,
    pub wgpu_instance: wgpu::Instance,
    pub wgpu_adapter: wgpu::Adapter,
    pub wgpu_device: wgpu::Device,
    pub wgpu_queue: wgpu::Queue,
    pipeline_cache: HashMap<&'a str, Rc<wgpu::RenderPipeline>>,
    windows: HashMap<u32, Rc<RefCell<GfxWindow<'a>>>>,
}
pub struct GfxWindow<'a> {
    pub egui_ctx: egui::Context,
    pub egui_renderer: egui_wgpu::Renderer,
    pub sdl_window: sdl2::video::Window,
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'a>,
    render_fn: Rc<RefCell<dyn FnMut(&mut GfxContext, &mut GfxWindow) -> Result<(), String>>>,
    pub running_time: f64,
    last_delta_instant: Instant,
    pub dpi_scale: f32,
    pub egui_state: EguiSDL2State,
    should_close: bool,
}

pub struct FusedCursor {
    pub cursor: sdl2::mouse::Cursor,
    pub icon: sdl2::mouse::SystemCursor,
}

impl FusedCursor {
    pub fn new() -> Self {
        Self {
            cursor: sdl2::mouse::Cursor::from_system(sdl2::mouse::SystemCursor::Arrow).unwrap(),
            icon: sdl2::mouse::SystemCursor::Arrow,
        }
    }
}

impl Default for FusedCursor {
    fn default() -> Self {
        Self::new()
    }
}

pub fn translate_virtual_key_code(key: Keycode) -> Option<egui::Key> {
    Some(match key {
        Keycode::Left => Key::ArrowLeft,
        Keycode::Up => Key::ArrowUp,
        Keycode::Right => Key::ArrowRight,
        Keycode::Down => Key::ArrowDown,

        Keycode::Escape => Key::Escape,
        Keycode::Tab => Key::Tab,
        Keycode::Backspace => Key::Backspace,
        Keycode::Space => Key::Space,
        Keycode::Return => Key::Enter,

        Keycode::Insert => Key::Insert,
        Keycode::Home => Key::Home,
        Keycode::Delete => Key::Delete,
        Keycode::End => Key::End,
        Keycode::PageDown => Key::PageDown,
        Keycode::PageUp => Key::PageUp,

        Keycode::Kp0 | Keycode::Num0 => Key::Num0,
        Keycode::Kp1 | Keycode::Num1 => Key::Num1,
        Keycode::Kp2 | Keycode::Num2 => Key::Num2,
        Keycode::Kp3 | Keycode::Num3 => Key::Num3,
        Keycode::Kp4 | Keycode::Num4 => Key::Num4,
        Keycode::Kp5 | Keycode::Num5 => Key::Num5,
        Keycode::Kp6 | Keycode::Num6 => Key::Num6,
        Keycode::Kp7 | Keycode::Num7 => Key::Num7,
        Keycode::Kp8 | Keycode::Num8 => Key::Num8,
        Keycode::Kp9 | Keycode::Num9 => Key::Num9,

        Keycode::A => Key::A,
        Keycode::B => Key::B,
        Keycode::C => Key::C,
        Keycode::D => Key::D,
        Keycode::E => Key::E,
        Keycode::F => Key::F,
        Keycode::G => Key::G,
        Keycode::H => Key::H,
        Keycode::I => Key::I,
        Keycode::J => Key::J,
        Keycode::K => Key::K,
        Keycode::L => Key::L,
        Keycode::M => Key::M,
        Keycode::N => Key::N,
        Keycode::O => Key::O,
        Keycode::P => Key::P,
        Keycode::Q => Key::Q,
        Keycode::R => Key::R,
        Keycode::S => Key::S,
        Keycode::T => Key::T,
        Keycode::U => Key::U,
        Keycode::V => Key::V,
        Keycode::W => Key::W,
        Keycode::X => Key::X,
        Keycode::Y => Key::Y,
        Keycode::Z => Key::Z,

        _ => {
            return None;
        }
    })
}

pub struct EguiSDL2State {
    pub raw_input: RawInput,
    pub modifiers: Modifiers,
    pub dpi_scaling: f32,
    pub mouse_pointer_position: egui::Pos2,
    pub fused_cursor: FusedCursor,
}

impl EguiSDL2State {
    pub fn sdl2_input_to_egui(&mut self, window: &sdl2::video::Window, event: &sdl2::event::Event) {
        fn sdl_button_to_egui(btn: &MouseButton) -> Option<PointerButton> {
            match btn {
                MouseButton::Left => Some(egui::PointerButton::Primary),
                MouseButton::Middle => Some(egui::PointerButton::Middle),
                MouseButton::Right => Some(egui::PointerButton::Secondary),
                _ => None,
            }
        }

        use sdl2::event::Event::*;
        if event.get_window_id() != Some(window.id()) {
            return;
        }
        match event {
            // handle when window Resized and SizeChanged.
            Window { win_event, .. } => match win_event {
                WindowEvent::Resized(_x, _y) | sdl2::event::WindowEvent::SizeChanged(_x, _y) => {
                    let (pix_w, pix_h) = window.drawable_size();
                    self.update_screen_rect(pix_w as u32, pix_h as u32);
                }
                _ => (),
            },
            MouseButtonDown { mouse_btn, .. } => {
                if let Some(pressed) = sdl_button_to_egui(mouse_btn) {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.mouse_pointer_position,
                        button: pressed,
                        pressed: true,
                        modifiers: self.modifiers,
                    });
                }
            }
            MouseButtonUp { mouse_btn, .. } => {
                if let Some(released) = sdl_button_to_egui(mouse_btn) {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.mouse_pointer_position,
                        button: released,
                        pressed: false,
                        modifiers: self.modifiers,
                    });
                }
            }

            MouseMotion { x, y, .. } => {
                self.mouse_pointer_position = egui::pos2(*x as f32, *y as f32);
                self.raw_input
                    .events
                    .push(egui::Event::PointerMoved(self.mouse_pointer_position));
            }

            KeyUp {
                keycode, keymod, ..
            } => {
                let key_code = match keycode {
                    Some(key_code) => key_code,
                    _ => return,
                };
                let key = match translate_virtual_key_code(*key_code) {
                    Some(key) => key,
                    _ => return,
                };
                self.modifiers = Modifiers {
                    alt: (*keymod & Mod::LALTMOD == Mod::LALTMOD)
                        || (*keymod & Mod::RALTMOD == Mod::RALTMOD),
                    ctrl: (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (*keymod & Mod::RCTRLMOD == Mod::RCTRLMOD),
                    shift: (*keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD)
                        || (*keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD),
                    mac_cmd: *keymod & Mod::LGUIMOD == Mod::LGUIMOD,

                    //TOD: Test on both windows and mac
                    command: (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (*keymod & Mod::LGUIMOD == Mod::LGUIMOD),
                };

                self.raw_input.events.push(egui::Event::Key {
                    key: key,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: self.modifiers,
                });
            }

            KeyDown {
                keycode, keymod, ..
            } => {
                let key_code = match keycode {
                    Some(key_code) => key_code,
                    _ => return,
                };

                let key = match translate_virtual_key_code(*key_code) {
                    Some(key) => key,
                    _ => return,
                };
                self.modifiers = Modifiers {
                    alt: (*keymod & Mod::LALTMOD == Mod::LALTMOD)
                        || (*keymod & Mod::RALTMOD == Mod::RALTMOD),
                    ctrl: (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (*keymod & Mod::RCTRLMOD == Mod::RCTRLMOD),
                    shift: (*keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD)
                        || (*keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD),
                    mac_cmd: *keymod & Mod::LGUIMOD == Mod::LGUIMOD,

                    //TOD: Test on both windows and mac
                    command: (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (*keymod & Mod::LGUIMOD == Mod::LGUIMOD),
                };

                self.raw_input.events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: self.modifiers,
                });

                if self.modifiers.command && key == Key::C {
                    println!("copy event");
                    self.raw_input.events.push(egui::Event::Copy);
                } else if self.modifiers.command && key == Key::X {
                    // println!("cut event");
                    self.raw_input.events.push(egui::Event::Cut);
                } else if self.modifiers.command && key == Key::V {
                    // println!("paste");
                    if let Ok(contents) = window.subsystem().clipboard().clipboard_text() {
                        self.raw_input.events.push(egui::Event::Text(contents));
                    }
                }
            }

            TextInput { text, .. } => {
                self.raw_input.events.push(egui::Event::Text(text.clone()));
            }
            MouseWheel { x, y, .. } => {
                let delta = egui::vec2(*x as f32 * 8.0, *y as f32 * 8.0);
                let sdl = window.subsystem().sdl();
                // zoom:
                if sdl.keyboard().mod_state() & Mod::LCTRLMOD == Mod::LCTRLMOD
                    || sdl.keyboard().mod_state() & Mod::RCTRLMOD == Mod::RCTRLMOD
                {
                    let zoom_delta = (delta.y / 125.0).exp();
                    self.raw_input.events.push(egui::Event::Zoom(zoom_delta));
                }
                // horizontal scroll:
                else if sdl.keyboard().mod_state() & Mod::LSHIFTMOD == Mod::LSHIFTMOD
                    || sdl.keyboard().mod_state() & Mod::RSHIFTMOD == Mod::RSHIFTMOD
                {
                    let e = egui::Event::MouseWheel {
                        unit: MouseWheelUnit::Point,
                        delta: egui::vec2(delta.x + delta.y, 0.0),
                        modifiers: Default::default(),
                    };
                    self.raw_input.events.push(e);
                    // regular scroll:
                } else {
                    let e = egui::Event::MouseWheel {
                        unit: MouseWheelUnit::Point,
                        delta: egui::vec2(delta.x, delta.y),
                        modifiers: Default::default(),
                    };
                    self.raw_input.events.push(e)
                }
            }
            _ => {}
        }
    }

    pub fn update_screen_rect(&mut self, width: u32, height: u32) {
        let inv_scale = 1.0 / self.dpi_scaling;
        let rect = egui::vec2(width as f32 * inv_scale, height as f32 * inv_scale);
        self.raw_input.screen_rect = Some(Rect::from_min_size(Pos2::new(0f32, 0f32), rect));
    }

    pub fn update_time(&mut self, running_time: Option<f64>, delta: f32) {
        self.raw_input.time = running_time;
        self.raw_input.predicted_dt = delta;
    }

    pub fn new(width: u32, height: u32, dpi_scaling: f32) -> Self {
        let inv_scale = 1.0 / dpi_scaling;
        let rect = egui::vec2(width as f32 * inv_scale, height as f32 * inv_scale);
        let screen_rect = Rect::from_min_size(Pos2::new(0f32, 0f32), rect);
        let raw_input = RawInput {
            screen_rect: Some(screen_rect),
            ..RawInput::default()
        };
        let modifiers = Modifiers::default();
        EguiSDL2State {
            raw_input,
            modifiers,
            dpi_scaling,
            mouse_pointer_position: egui::Pos2::new(0.0, 0.0),
            fused_cursor: FusedCursor::new(),
        }
    }

    pub fn process_output(&mut self, window: &Window, egui_output: &egui::PlatformOutput) {
        EguiSDL2State::translate_cursor(&mut self.fused_cursor, egui_output.cursor_icon);
        for cmd in egui_output.commands.iter() {
            match cmd {
                egui::OutputCommand::CopyText(copied_text) => {
                    let result = window
                        .subsystem()
                        .clipboard()
                        .set_clipboard_text(&copied_text);
                    if result.is_err() {
                        dbg!("Unable to set clipboard content to SDL clipboard.");
                    }
                }
                egui::OutputCommand::OpenUrl(url) => {
                    let result = sdl2::url::open_url(&url.url);
                    if result.is_err() {
                        dbg!("Failed to open URL {}", &url.url);
                    }
                }
                cmd => {
                    println!("Unhandled Output Command {:?}", cmd);
                }
            }
        }
    }

    fn translate_cursor(fused: &mut FusedCursor, cursor_icon: egui::CursorIcon) {
        let tmp_icon = match cursor_icon {
            egui::CursorIcon::Crosshair => SystemCursor::Crosshair,
            egui::CursorIcon::Default => SystemCursor::Arrow,
            egui::CursorIcon::Grab => SystemCursor::Hand,
            egui::CursorIcon::Grabbing => SystemCursor::SizeAll,
            egui::CursorIcon::Move => SystemCursor::SizeAll,
            egui::CursorIcon::PointingHand => SystemCursor::Hand,
            egui::CursorIcon::ResizeHorizontal => SystemCursor::SizeWE,
            egui::CursorIcon::ResizeNeSw => SystemCursor::SizeNESW,
            egui::CursorIcon::ResizeNwSe => SystemCursor::SizeNWSE,
            egui::CursorIcon::ResizeVertical => SystemCursor::SizeNS,
            egui::CursorIcon::Text => SystemCursor::IBeam,
            egui::CursorIcon::NotAllowed | egui::CursorIcon::NoDrop => SystemCursor::No,
            egui::CursorIcon::Wait => SystemCursor::Wait,
            //There doesn't seem to be a suitable SDL equivalent...
            _ => SystemCursor::Arrow,
        };

        if tmp_icon != fused.icon {
            fused.cursor = Cursor::from_system(tmp_icon).unwrap();
            fused.icon = tmp_icon;
            fused.cursor.set();
        }
    }
}

impl<'a> GfxWindow<'a> {
    fn render(&mut self, gfx: &mut GfxContext) -> Result<(), String> {
        let now = Instant::now();
        let delta = now
            .saturating_duration_since(self.last_delta_instant)
            .as_secs_f64();
        self.last_delta_instant = now;
        self.running_time += delta as f64;
        self.egui_state
            .update_time(Some(self.running_time), delta as f32);

        let render_fn = self.render_fn.clone();
        (render_fn.borrow_mut())(gfx, self)?;
        Ok(())
    }
    fn proc_sdl_event(&mut self, event: &Event) {
        self.egui_state.sdl2_input_to_egui(&self.sdl_window, &event);
    }
    //Make sure to call rpass.forget_lifetime before providing the render pass here
    pub fn draw_egui_inline(
        &mut self,
        gfx: &mut GfxContext,
        rpass: &mut wgpu::RenderPass<'static>,
        encoder: &mut wgpu::CommandEncoder,
        full_output: egui::FullOutput,
    ) {
        self.egui_state
            .process_output(&self.sdl_window, &full_output.platform_output);
        let tris = self
            .egui_ctx
            .tessellate(full_output.shapes, self.egui_state.dpi_scaling);

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.egui_state.dpi_scaling,
        };

        {
            for (id, image_delta) in &full_output.textures_delta.set {
                self.egui_renderer.update_texture(
                    &gfx.wgpu_device,
                    &gfx.wgpu_queue,
                    *id,
                    image_delta,
                );
            }

            self.egui_renderer.update_buffers(
                &gfx.wgpu_device,
                &gfx.wgpu_queue,
                encoder,
                &tris[..],
                &screen_descriptor,
            );
        }
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        {
            self.egui_renderer
                .render(rpass, &tris[..], &screen_descriptor);
        }
    }
    pub fn draw_egui(
        &mut self,
        gfx: &mut GfxContext,
        full_output: egui::FullOutput,
    ) -> Result<(), String> {
        let frame = self
            .surface
            .get_current_texture()
            .map_err(|x| format!("Error creating texture {x}"))?;
        let output = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gfx
            .wgpu_device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command_encoder"),
            });
        {
            let mut rpass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    label: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
                .forget_lifetime();

            self.draw_egui_inline(gfx, &mut rpass, &mut encoder, full_output);
        }

        gfx.wgpu_queue.submit([encoder.finish()]);

        frame.present();
        Ok(())
    }
    pub fn close(&mut self) {
        self.should_close = true;
    }
}
impl<'a> GfxContext<'a> {
    pub fn new() -> Result<GfxContext<'a>, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let adapter_opt =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            }));
        let adapter = adapter_opt.expect("No adapter found");

        let (device, queue) = match pollster::block_on(
            adapter.request_device(&wgpu::DeviceDescriptor::default(), None),
        ) {
            Ok(a) => a,
            Err(e) => return Err(e.to_string()),
        };
        println!("Initialized GFXContext");

        let gfx_context = GfxContext {
            sdl_context: sdl_context,
            sdl_video: video_subsystem,
            wgpu_instance: instance,
            wgpu_adapter: adapter,
            wgpu_device: device,
            wgpu_queue: queue,
            pipeline_cache: HashMap::new(),
            windows: HashMap::new(),
        };

        return Ok(gfx_context);
    }
    pub fn has_windows_open(&self) -> bool {
        return self.windows.is_empty() == false;
    }
    pub fn open_window(
        &mut self,
        title: &str,
        w: u32,
        h: u32,
        render_fn: impl FnMut(&mut GfxContext<'_>, &mut GfxWindow<'_>) -> Result<(), String> + 'static,
    ) -> Result<(), String> {
        let sdl_window = self
            .sdl_video
            .window(title, w, h)
            .position_centered()
            .allow_highdpi()
            .resizable()
            .metal_view()
            .build()
            .map_err(|e| e.to_string())?;

        let (width, height) = sdl_window.size();
        println!("Create window {} {}", width, height);
        let surface = unsafe {
            match self
                .wgpu_instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&sdl_window).unwrap())
            {
                Ok(s) => s,
                Err(e) => return Err(e.to_string()),
            }
        };

        let surface_caps = surface.get_capabilities(&self.wgpu_adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: sdl_window.drawable_size().0,
            height: sdl_window.drawable_size().1,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::default(),
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&self.wgpu_device, &config);
        let egui_ctx = egui::Context::default();
        let dpi_scale = (config.width as f32) / (width as f32);
        let egui_renderer =
            egui_wgpu::Renderer::new(&self.wgpu_device, config.format, None, 1, true);
        let pix_w = config.width;
        let pix_h = config.height;

        egui_ctx.set_zoom_factor(dpi_scale);

        let window = GfxWindow {
            sdl_window: sdl_window,
            surface: surface,
            config: config,
            render_fn: Rc::new(RefCell::new(render_fn)),
            egui_ctx: egui_ctx,
            egui_renderer: egui_renderer,
            running_time: 0.0,
            egui_state: EguiSDL2State::new(pix_w, pix_h, dpi_scale),
            dpi_scale: dpi_scale,
            last_delta_instant: Instant::now(),
            should_close: false,
        };

        self.windows
            .insert(window.sdl_window.id(), Rc::new(RefCell::new(window)));
        return Ok(());
    }
    fn forward_egui_event(&mut self, window_id: u32, event: Event) {
        if let Some(gfx_window_ptr) = self.windows.get(&window_id) {
            let mut gfx_window = gfx_window_ptr.borrow_mut();
            gfx_window.proc_sdl_event(&event);
        }
    }
    pub fn poll_events(&mut self) -> Result<(), String> {
        let mut wins_to_close = Vec::new();
        for event in self.sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Window {
                    window_id,
                    win_event: WindowEvent::SizeChanged(width, _height),
                    ..
                } if self.windows.contains_key(&window_id) => {
                    {
                        let mut gfx_window = self.windows.get_mut(&window_id).unwrap().borrow_mut();
                        let pix_w = gfx_window.sdl_window.drawable_size().0 as u32;
                        let pix_h = gfx_window.sdl_window.drawable_size().1 as u32;
                        gfx_window.config.width = pix_w;
                        gfx_window.config.height = pix_h;
                        gfx_window
                            .surface
                            .configure(&self.wgpu_device, &gfx_window.config);

                        let dpi_scale = (pix_w as f32) / (width as f32);
                        gfx_window.egui_ctx.set_zoom_factor(dpi_scale);
                        gfx_window.egui_state.update_screen_rect(pix_w, pix_h);
                    }
                    self.forward_egui_event(window_id, event);
                }
                Event::Window {
                    window_id,
                    win_event: WindowEvent::Close,
                    ..
                } => {
                    self.forward_egui_event(window_id, event);
                    wins_to_close.push(window_id.clone());
                }
                Event::KeyDown { window_id, .. } => {
                    self.forward_egui_event(window_id, event);
                }
                Event::Window { window_id, .. }
                | Event::KeyUp { window_id, .. }
                | Event::TextEditing { window_id, .. }
                | Event::TextInput { window_id, .. }
                | Event::MouseMotion { window_id, .. }
                | Event::MouseButtonDown { window_id, .. }
                | Event::MouseButtonUp { window_id, .. }
                | Event::MouseWheel { window_id, .. }
                | Event::DropFile { window_id, .. }
                | Event::DropText { window_id, .. }
                | Event::DropBegin { window_id, .. }
                | Event::DropComplete { window_id, .. }
                | Event::User { window_id, .. } => {
                    self.forward_egui_event(window_id, event);
                }
                e => {
                    for (_key, w) in self.windows.iter() {
                        w.borrow_mut().proc_sdl_event(&e);
                    }
                    dbg!(e);
                }
            }
        }
        for (win_id, win) in self.windows.iter() {
            if win.borrow().should_close {
                wins_to_close.push(*win_id);
            }
        }
        for win_id in wins_to_close {
            self.windows.remove(&win_id);
        }
        if self.windows.is_empty() {}

        for (_key, window) in self.windows.clone() {
            window.borrow_mut().render(self)?;
        }
        return Ok(());
    }
    pub fn cache_render_pipeline(
        &mut self,
        key: &'a str,
        create_fn: fn(&mut GfxContext) -> Rc<wgpu::RenderPipeline>,
    ) -> Rc<wgpu::RenderPipeline> {
        if self.pipeline_cache.contains_key(key) == false {
            let render_pipeline = create_fn(self);
            println!("Finished making gfx pipeline for: {}", key);
            self.pipeline_cache.insert(key, render_pipeline);
        }
        return self.pipeline_cache[key].clone();
    }
}
