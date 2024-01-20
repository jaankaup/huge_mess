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

    /// Handle user input.
    // fn input(&mut self, _queue: &wgpu::Queue, _input: &InputCache) {

    // }

    /// Resize window.
    fn resize(&mut self, wgpu_context: &WGPUContext, surface_configuration: &wgpu::SurfaceConfiguration, new_size: winit::dpi::PhysicalSize<u32>) {
    // fn resize(&mut self, context: &WGPUContext, size: PhysicalSize<u32>) {
    // fn resize(&mut self, &WGPUContext, sc_desc: &wgpu::SurfaceConfiguration, _new_size: winit::dpi::PhysicalSize<u32>) {
    //fn resize(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SurfaceConfiguration, _new_size: winit::dpi::PhysicalSize<u32>) {

        // TODO: add this functionality to the Screen.
        // self.screen.depth_texture = Some(ATexture::create_depth_texture(device, sc_desc, Some("depth-texture")));
        // self.camera.resize(sc_desc.width as f32, sc_desc.height as f32);
    }

    /// Application update.
    fn update(&mut self, wgpu_context: &WGPUContext, input_cache: &InputCache) {
    //fn update(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue, _input: &InputCache, _spawner: &Spawner) {

    }

    // Exit.
    //fn exit(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue, _input: &InputCache, _spawner: &Spawner) {
    //    // log::info!("Exit.");
    //}
     fn close(&mut self, wgpu_context: &WGPUContext){ 
     }
}

fn main() {

    initialize_env_logger(&vec![("smoke".to_string(), LevelFilter::Info)]);
    run::<SmokeFeatures, BasicLoop, SmokeApp>("yeah");
    // Initialize logging.
    // initialize_simple_logger(&vec![("dummy_example".to_string(), LevelFilter::Info)]);

    // log::info!("Hekotus from smoke");

    // Execute application.
    // run_loop::<SmokeApp, BasicLoop, dummy_features::SmokeFeatures>();

}
