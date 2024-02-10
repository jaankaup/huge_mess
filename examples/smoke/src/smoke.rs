use engine::pipelines::RenderPipelineWrapper;
use engine::texture::{
    Texture as Tex,
};
use engine::camera::Camera;
use engine::render_pass::create_render_pass;
use wgpu::TextureView;
use engine::core::SurfaceWrapper;
use engine::basic_loop::BasicLoop;
use crate::configuration::SmokeFeatures;
use crate::smoke_pipelines::{
    // create_default_render_pipeline,
    DefaultBindGroups,
    default_render_shader_v4n4_camera_light_tex2
};
use engine::core::run;

use engine::core::WGPUContext;
use engine::core::Application;
use engine::input_cache::InputCache;
use engine::meshes::create_cube;
 
use engine::logger::initialize_env_logger; 
use log::LevelFilter;
mod configuration; 
mod smoke_pipelines; 

struct SmokeApp {
    _depth_texture: Option<Tex>, 
    _camera: Camera,
    _buffer: wgpu::Buffer,
    render_pipeline_wrapper: RenderPipelineWrapper,
    // render_pipeline_wrapper: RenderPipelineWrapper<DefaultBindGroups>,
}

impl Application for SmokeApp {

    /// Initialize application.
    fn init(context: &WGPUContext, surface: &SurfaceWrapper) -> Self {

        log::info!("Initializing SmokeApp");

        // Create camera.
        let mut camera = Camera::new(surface.config().width as f32,
                                     surface.config().height as f32,
                                     (180.0, 130.0, 480.0),
                                     -89.0,
                                     -4.0
        );
        camera.set_rotation_sensitivity(0.4);
        camera.set_movement_sensitivity(0.2);

        Self {
            _depth_texture: Some(Tex::create_depth_texture(&context, surface.config(), None)),
            _camera: camera,
            _buffer: create_cube(&context.device, true),
            render_pipeline_wrapper: default_render_shader_v4n4_camera_light_tex2(&context.device, &surface.config()),
        }
    }

    /// Render application.
    fn render(&mut self, context: &WGPUContext, view: &TextureView, _surface: &SurfaceWrapper ) {

            let clear_color = Some(wgpu::Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0, });

            // If there is nothing to draw, this must be executed.
            let mut dummy_encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Dummy encoder") });
            {
                create_render_pass(
                    &mut dummy_encoder,
                    view,
                    &None, // depth
                    true, // clear
                    &clear_color,
                    &None);// label
            }
            context.queue.submit(Some(dummy_encoder.finish()));
    }

    /// Resize window.
    fn resize(&mut self, _wgpu_context: &WGPUContext, _surface_configuration: &wgpu::SurfaceConfiguration, _new_size: winit::dpi::PhysicalSize<u32>) {

    }

    /// Application update.
    fn update(&mut self, _wgpu_context: &WGPUContext, _input_cache: &InputCache) {

    }

    fn close(&mut self, _wgpu_context: &WGPUContext){ 
    }
}

fn main() {

    initialize_env_logger(&vec![("smoke".to_string(), LevelFilter::Info)]);
    run::<SmokeFeatures, BasicLoop, SmokeApp>("yeah");
}
