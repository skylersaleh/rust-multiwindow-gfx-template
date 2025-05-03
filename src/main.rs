extern crate pollster;
extern crate sdl2;
extern crate wgpu;
use std::borrow::Cow;
mod gfx_util;
use gfx_util::{GfxContext, GfxWindow};
use std::cell::RefCell;
use std::rc::Rc;

struct AppState {
    should_run: bool,
    slider_value: f32,
    checkbox: bool,
}
impl AppState {
    fn new() -> AppState {
        return AppState {
            should_run: true,
            slider_value: 0.0,
            checkbox: false,
        };
    }
}
fn get_triangle_render_pipeline(gfx: &mut GfxContext) -> Rc<wgpu::RenderPipeline> {
    return gfx.cache_render_pipeline("triangle", |gfx: &mut GfxContext| {
        let shader = gfx
            .wgpu_device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            });

        let pipeline_layout =
            gfx.wgpu_device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[],
                    label: None,
                    push_constant_ranges: &[],
                });

        let render_pipeline = Rc::new(gfx.wgpu_device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    buffers: &[],
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Front),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                label: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            },
        ));
        return render_pipeline;
    });
}
fn second_window_function(
    gfx: &mut GfxContext,
    window: &mut GfxWindow,
    state_ptr: Rc<RefCell<AppState>>,
) -> Result<(), String> {
    let mut close_window = false;
    let full_output = window
        .egui_ctx
        .run(window.egui_state.raw_input.take(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Hello from another window!");
                ui.label(format!(
                    "This window has been open for {}",
                    window.running_time
                ));
                ui.label(format!(
                    "w: {} h: {} dpi: {}",
                    window.config.width, window.config.height, window.dpi_scale
                ));
                ui.checkbox(
                    &mut state_ptr.borrow_mut().checkbox,
                    "This is the same checkbox",
                );
                if ui.button("Close Window").clicked() {
                    close_window = true;
                }
                ui.end_row();
            });
        });

    if close_window {
        window.close();
    }
    window.draw_egui(gfx, full_output)?;
    return Ok(());
}

fn main_render_function(
    gfx: &mut GfxContext,
    window: &mut GfxWindow,
    state_ptr: Rc<RefCell<AppState>>,
) -> Result<(), String> {
    let mut state = state_ptr.borrow_mut();
    let full_output = window
        .egui_ctx
        .run(window.egui_state.raw_input.take(), |ctx| {
            egui::Window::new("EGUI Test Window")
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.label("This");
                    ui.label("is");
                    ui.label("a");
                    ui.label("long");
                    ui.label("list");
                    ui.label("of");
                    ui.label("labels");
                    ui.label("to");
                    ui.label("demonstrate");
                    ui.label("scrolling!");
                    ui.label("Here is a spinner");
                    ui.spinner();
                    ui.add(egui::Slider::new(&mut state.slider_value, 0.0..=100.0).text("Slider"));
                    ui.label("Here is a progress bar");
                    ui.add(egui::ProgressBar::new(state.slider_value / 100.));
                    ui.label(format!(
                        "This window's dimensions are: {} h: {}",
                        window.config.width, window.config.height
                    ));
                    ui.hyperlink("https://github.com/emilk/egui");
                    ui.checkbox(&mut state.checkbox, "This is a checkbox");
                    let state_ptr_copy = state_ptr.clone();
                    if ui.button("Press me to open a new window").clicked() {
                        gfx.open_window("I'm a new window", 400, 200, move |gfx, window| {
                            second_window_function(gfx, window, state_ptr_copy.clone())
                        })
                        .unwrap();
                    }
                    ui.end_row();
                });
        });

    let frame = window
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
        let render_pipeline = get_triangle_render_pipeline(gfx);
        rpass.set_pipeline(&render_pipeline);
        rpass.draw(0..3, 0..1);
        window.draw_egui_inline(gfx, &mut rpass, &mut encoder, full_output);
    }

    gfx.wgpu_queue.submit([encoder.finish()]);

    frame.present();

    return Ok(());
}
fn main_loop(state: &mut Rc<RefCell<AppState>>, gfx: &mut GfxContext) -> Result<(), String> {
    let state_copy = state.clone();
    gfx.open_window("Rust + SDL2 + WGPU 🦊", 800, 600, move |gfx, window| {
        main_render_function(gfx, window, state_copy.clone())
    })?;

    while state.borrow().should_run {
        gfx.poll_events()?;
        state.borrow_mut().should_run &= gfx.has_windows_open();
    }
    return Ok(());
}
fn main() -> Result<(), String> {
    // Show logs from wgpu
    env_logger::init();
    let mut state = Rc::new(RefCell::new(AppState::new()));
    while state.borrow().should_run {
        let mut gfx = GfxContext::new().unwrap();
        if let Err(string) = main_loop(&mut state, &mut gfx) {
            println!("Main render loop hit an error:");
            println!("{}", string);
            println!("Recreating all Gfx resources");
        }
    }
    return Ok(());
}
