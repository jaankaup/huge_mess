use wgpu::StoreOp;
use wgpu::TextureView;
use engine::core::SurfaceWrapper;
use engine::basic_loop::BasicLoop;
use crate::configuration::SmokeFeatures;
use engine::core::run;
use winit::dpi::PhysicalSize;
use engine::core::WGPUContext;
use engine::core::Application;
use engine::input_cache::InputCache;
use engine::core; 
use engine::logger::initialize_env_logger; 
use log::LevelFilter;
mod configuration; 

// TODO: drop renderpass if there is nothing to draw.

struct SmokeApp {
    // screen: ScreenTexture,
    // camera: Camera,
    render: bool,
}

impl Application for SmokeApp {

    /// Initialize application.
    fn init(configuration: &WGPUContext) -> Self {

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
            // screen: ScreenTexture::init(&configuration.device, &configuration.sc_desc, true),
            // camera,
            render: false,
        }
    }

    /// Render application.
    fn render(&mut self, context: &WGPUContext, view: &TextureView, surface: &SurfaceWrapper ) {

            let clear_color = Some(wgpu::Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0, });

            // If there is nothing to draw, this must be executed.
            let mut dummy_encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Dummy encoder") });
            {
                let render_pass = dummy_encoder.begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: Some("Render pass descriptor"),
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(clear_color.unwrap()),
                                    store: StoreOp::Store,
                                },
                            }),
                        ],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
            }
            context.queue.submit(Some(dummy_encoder.finish()));
    }

    /// Resize window.
    fn resize(&mut self, wgpu_context: &WGPUContext, surface_configuration: &wgpu::SurfaceConfiguration, new_size: winit::dpi::PhysicalSize<u32>) {

    }

    /// Application update.
    fn update(&mut self, wgpu_context: &WGPUContext, input_cache: &InputCache) {

    }

    fn close(&mut self, wgpu_context: &WGPUContext){ 
    }
}

fn main() {

    initialize_env_logger(&vec![("smoke".to_string(), LevelFilter::Info)]);
    run::<SmokeFeatures, BasicLoop, SmokeApp>("yeah");
}
