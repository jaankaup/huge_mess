use engine::texture::{
    Texture as Tex,
};
use engine::render_pass::create_render_pass;
use wgpu::StoreOp;
use wgpu::TextureView;
use engine::core::SurfaceWrapper;
use engine::basic_loop::BasicLoop;
use crate::configuration::SmokeFeatures;
use engine::core::run;

use engine::core::WGPUContext;
use engine::core::Application;
use engine::input_cache::InputCache;
 
use engine::logger::initialize_env_logger; 
use log::LevelFilter;
mod configuration; 

// TODO: drop renderpass if there is nothing to draw.

struct SmokeApp {
    depth_texture: Option<Tex> 
    // screen: ScreenTexture,
    // camera: Camera,
    // render: bool,
}

impl Application for SmokeApp {

    /// Initialize application.
    fn init(context: &WGPUContext, surface: &SurfaceWrapper) -> Self {

        log::info!("Initializing SmokeApp");
        
        // Create camera.
        // let mut camera = Camera::new(configuration.size.width as f32,
        //                              configuration.size.height as f32,
        //                              (180.0, 130.0, 480.0),
        //                              -89.0,
        //                              -4.0
        // );
        // camera.set_rotation_sensitivity(0.4);
        // camera.set_movement_sensitivity(0.2);

        Self {
            depth_texture: Some(Tex::create_depth_texture(context, surface.config(), None)),
            // screen: ScreenTexture::init(&configuration.device, &configuration.sc_desc, true),
            // camera,
            // render: false,
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
