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
}

impl Application for AabbApp {

    /// Initialize application.
    fn init(context: &WGPUContext, surface: &SurfaceWrapper) -> Self {

        log::info!("Initializing AabbApp");

        log::info!("Creating camera.");

        // Create camera.
        let mut camera = Camera::new(surface.config().width as f32,
                                     surface.config().height as f32,
                                     (15.0, 12.0, -18.0),
                                     (0.0, 0.0, 0.0)
        );
        camera.set_rotation_sensitivity(1.0);
        camera.set_movement_sensitivity(0.1);

        let draw_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mc output buffer"),
            size: 256*128*256*16 as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        const MAX_NUMBER_OF_ARROWS:     u32 = 40960;
        const MAX_NUMBER_OF_AABBS:      u32 = 262144;
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

        log::info!("Finished initialization.");

        Self {
            depth_texture: Some(Tex::create_depth_texture(&context, surface.config(), None)),
            camera: camera,
            draw_buffer: draw_buffer,
            gpu_debugger: gpu_debugger,
        }
    }

    /// Render application.
    fn render(&mut self, context: &WGPUContext, view: &TextureView, _surface: &SurfaceWrapper ) {

        let clear_color = Some(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0, });


        // If there is nothing to draw, this must be executed.
        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Dummy encoder") });
        // draw_indirect(
        //     &mut encoder,
        //     &view,
        //     self.depth_texture.as_ref(),
        //     &vec![&self.bind_group1, &self.bind_group2],
        //     self.render_pipeline_wrapper.get_pipeline(),
        //     &self.output_buffer, // TODO: create this!
        //     self.marching_cubes.get_draw_indirect_buffer(),
        //     0,
        //     &clear_color,
        //     true
        //     );

        // draw(&mut encoder,
        //      view,
        //      self.depth_texture.as_ref(),
        //      &vec![&self.bind_group1, &self.bind_group2],
        //      self.render_pipeline_wrapper.get_pipeline(),
        //      &self.buffer,
        //      0..36,
        //      &clear_color,
        //      false);

        // context.queue.submit(Some(encoder.finish()));

    }

    /// Resize window.
    fn resize(&mut self, context: &WGPUContext, surface_configuration: &wgpu::SurfaceConfiguration, _new_size: winit::dpi::PhysicalSize<u32>) {

        self.depth_texture = Some(Texture::create_depth_texture(context, &surface_configuration, Some("depth-texture")));
        self.camera.resize(surface_configuration.width as f32, surface_configuration.height as f32);
    }

    /// Application update.
    fn update(&mut self, context: &WGPUContext, input_cache: &InputCache) {
        self.camera.update_from_input(&context.queue, &input_cache);

        let total_grid_count = 256 * 128 * 256;

    }

    fn close(&mut self, _wgpu_context: &WGPUContext){ 
    }
}

fn main() {

    initialize_env_logger(&vec![("mc".to_string(), LevelFilter::Info)]);
    run::<AabbFeatures, BasicLoop, AabbApp>("yeah");
}
