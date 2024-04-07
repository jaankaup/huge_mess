use engine::gpu_debugger::primitive_processor::Arrow;
use std::mem::transmute;
use engine::gpu_debugger::primitive_processor::AABB;
use engine::gpu_debugger::GpuDebugger;
use engine::common_structs::DrawIndirect;
use engine::buffer::to_vec;
use engine::draw_commands::draw_indirect;
use crate::configuration::AabbFeatures;
use engine::draw_commands::draw;
use engine::texture::Texture;
use engine::lights::LightBuffer;
use engine::pipelines::RenderPipelineWrapper;
use engine::texture::{
    Texture as Tex,
};
use engine::camera::Camera;

use wgpu::TextureView;

use engine::core::SurfaceWrapper;
use engine::basic_loop::BasicLoop;
use engine::core::run;
// fn vs_main(@location(0) pos: vec3<f32>, @location(1) col: u32) -> FragmentInput {
use engine::core::WGPUContext;
use engine::core::Application;
use engine::input_cache::InputCache;
use engine::meshes::create_cube;

use engine::logger::initialize_env_logger; 
use log::LevelFilter;
mod configuration; 

/// Marching cubes application.
struct AabbApp {
    depth_texture: Option<Tex>,
    camera: Camera,
    draw_buffer: wgpu::Buffer,
    gpu_debugger: GpuDebugger,
    some_counter: u32,
    y_counter: u32,
    visited: bool,
}

impl Application for AabbApp {

    /// Initialize application.
    fn init(context: &WGPUContext, surface: &SurfaceWrapper) -> Self {

        log::info!("Initializing AabbApp");

        log::info!("Creating camera.");

        // Create camera.
        let mut camera = Camera::new(surface.config().width as f32,
                                     surface.config().height as f32,
                                     (-15.0, 12.0, 28.0),
                                     (0.0, 0.0, 0.0)
        );
        camera.set_rotation_sensitivity(1.0);
        camera.set_movement_sensitivity(0.1);

        let draw_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Aabb draw buffer"),
            size: 256*128*256*16 as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        const MAX_NUMBER_OF_ARROWS:     u32 = 40960;
        const MAX_NUMBER_OF_AABBS:      u32 = 1000000; // 262144;
        const MAX_NUMBER_OF_AABB_WIRES: u32 = 40960;
        const MAX_NUMBER_OF_CHARS:      u32 = 262144;

        log::info!("Creating GpuDebugger.");
        let gpu_debugger = GpuDebugger::init(
                &context.device,
                &surface.config(),
                &draw_buffer,
                &camera.get_camera_uniform(&context.device),
                MAX_NUMBER_OF_ARROWS,
                MAX_NUMBER_OF_AABBS,
                MAX_NUMBER_OF_AABB_WIRES,
                MAX_NUMBER_OF_CHARS,
                2000000,
                4000,
                64);

        // let mut aabb_grid: Vec<AABB> = Vec::new();

        // for i in 0..300 {
        //     for j in 0..300 {
        //         let color: f32 = if (i & 1 == 0) ^ (j & 1 == 0) { unsafe {transmute::<u32, f32>(0xFFFF00FF)} } else { unsafe {transmute::<u32, f32>(0x1100FFFF)}};
        //         aabb_grid.push(
        //         AABB {
        //             min: [i as f32 * 4.0 + 0.1, 1.0, j as f32 * 4.0 + 0.1, color],
        //             max: [i as f32 * 4.0 + 4.1, 1.2, j as f32 * 4.0 + 4.1, color],
        //         });
        //     }
        // }

        // gpu_debugger.add_aabbs(&context.device, &context.queue, &aabb_grid);

        log::info!("Finished initialization.");

        Self {
            depth_texture: Some(Tex::create_depth_texture(&context, surface.config(), None)),
            camera: camera,
            draw_buffer: draw_buffer,
            gpu_debugger: gpu_debugger,
            some_counter: 0,
            y_counter: 2,
            visited: false,
        }
    }

    /// Render application.
    fn render(&mut self, context: &WGPUContext, view: &TextureView, _surface: &SurfaceWrapper ) {

        // let aabb01 = AABB {
        //     min: [1.0, 1.0, 1.0, 23423321.0],
        //     max: [5.0, 5.0, 5.0, 23423321.0],
        // };

        let clear_color = Some(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0, });

        let mut clear = true;

        log::info!("Rendering");
        self.gpu_debugger.render(
                  &context.device,
                  &context.queue,
                  view,
                  &self.draw_buffer,
                  &self.depth_texture.as_ref().unwrap(), //depth_texture: &Tex,
                  &mut clear
            );
    }

    /// Resize window.
    fn resize(&mut self, context: &WGPUContext, surface_configuration: &wgpu::SurfaceConfiguration, _new_size: winit::dpi::PhysicalSize<u32>) {

        self.depth_texture = Some(Texture::create_depth_texture(context, &surface_configuration, Some("depth-texture")));
        self.camera.resize(surface_configuration.width as f32, surface_configuration.height as f32);
    }

    /// Application update.
    fn update(&mut self, context: &WGPUContext, input_cache: &InputCache) {
        self.camera.update_from_input(&context.queue, &input_cache);

        log::info!("Add aabb");
        let color: f32 = if (self.some_counter & 1 == 0) ^ (self.y_counter & 1 == 0) { unsafe {transmute::<u32, f32>(0xFFFF00FF)} } else { unsafe {transmute::<u32, f32>(0x1100FFFF)}};
        self.gpu_debugger.add_aabb(&context.device, &context.queue, &AABB {
            min: [self.some_counter as f32 * 4.0 + 0.1, 1.0, self.y_counter as f32 * 4.0 + 0.1, color],
            max: [self.some_counter as f32 * 4.0 + 4.1, 1.2, self.y_counter as f32 * 4.0 + 4.1, color],
        });
        log::info!("Add arrow");
        self.gpu_debugger.add_arrow(&context.device, &context.queue, &Arrow {
            start_pos: [self.some_counter as f32 * 5.0 + 1.0, 8.0, self.y_counter as f32 * 5.0 + 1.0, 1.0],
            end_pos: [self.some_counter as f32 * 5.0 + 4.0, 122.0, self.y_counter as f32 * 5.0 + 4.0, 1.0],
            color: 0xFF0000FF,
            size: 0.5,
            _padding: [0,0],
        });
        self.some_counter += 1;
        // if self.some_counter < 2 { 
        //     if (self.y_counter < 1) {
        //         log::info!("Add arrow");
        //         self.gpu_debugger.add_arrow(&context.device, &context.queue, &Arrow {
        //             start_pos: [self.some_counter as f32 * 5.0 + 1.0, 8.0, self.y_counter as f32 * 5.0 + 1.0, 1.0],
        //             end_pos: [self.some_counter as f32 * 5.0 + 4.0, 122.0, self.y_counter as f32 * 5.0 + 4.0, 1.0],
        //             color: 0xFF0000FF,
        //             size: 0.5,
        //             _padding: [0,0],
        //         });
        //     }
        //     // else {
        //     //     log::info!("Add aabb");
        //     //     let color: f32 = if (self.some_counter & 1 == 0) ^ (self.y_counter & 1 == 0) { unsafe {transmute::<u32, f32>(0xFFFF00FF)} } else { unsafe {transmute::<u32, f32>(0x1100FFFF)}};
        //     //     self.gpu_debugger.add_aabb(&context.device, &context.queue, &AABB {
        //     //         min: [self.some_counter as f32 * 4.0 + 0.1, 1.0, self.y_counter as f32 * 4.0 + 0.1, color],
        //     //         max: [self.some_counter as f32 * 4.0 + 4.1, 1.2, self.y_counter as f32 * 4.0 + 4.1, color],
        //     //     });
        //     // }

        //     self.some_counter += 1;
        //     if self.some_counter == 2 { self.some_counter = 0; self.y_counter += 1; if self.y_counter > 2 { self.y_counter = 0; }  }
        // }
    }

    fn close(&mut self, _wgpu_context: &WGPUContext){ 
    }
}

fn main() {

    initialize_env_logger(&vec![("mc".to_string(), LevelFilter::Info)]);
    run::<AabbFeatures, BasicLoop, AabbApp>("yeah");
}
