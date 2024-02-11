use engine::draw_commands::draw;
use engine::texture::Texture;
use engine::lights::LightBuffer;
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
    // DefaultBindGroups,
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
    depth_texture: Option<Tex>,
    camera: Camera,
    buffer: wgpu::Buffer,
    render_pipeline_wrapper: RenderPipelineWrapper,
    light: LightBuffer,
    bind_group1: wgpu::BindGroup,
    bind_group2: wgpu::BindGroup,
}

impl Application for SmokeApp {

    /// Initialize application.
    fn init(context: &WGPUContext, surface: &SurfaceWrapper) -> Self {

        log::info!("Initializing SmokeApp");

        log::info!("Creating camera.");

        // Create camera.
        let mut camera = Camera::new(surface.config().width as f32,
                                     surface.config().height as f32,
                                     (15.0, 12.0, -18.0),
                                     (0.0, 0.0, 0.0)
        );
        camera.set_rotation_sensitivity(1.0);
        camera.set_movement_sensitivity(0.1);

        log::info!("Creating light.");

        let light = LightBuffer::create(
            &context.device,
            [25.0, 55.0, 25.0], // pos
            [25, 25, 130],  // spec
            [255,200,255], // light
            155.0,
            0.35,
            0.0013
            );

        log::info!("Creating textures.");

        let grass_texture = Texture::create_from_bytes(
            &context.queue,
            &context.device,
            &surface.config(),
            1,
            &include_bytes!("../../../textures/grass_flowers.png")[..],
            None);

        let rock_texture = Texture::create_from_bytes(
            &context.queue,
            &context.device,
            &surface.config(),
            1,
            &include_bytes!("../../../textures/rock.png")[..],
            None);

        log::info!("Creating pipeline wrapper.");
        let render_pipeline_wrapper = default_render_shader_v4n4_camera_light_tex2(&context.device, &surface.config());
        log::info!("Creating bind groups.");
        let bind_group1 = render_pipeline_wrapper.create_bind_group(&context.device,
                                                  &vec![
                                                    camera.get_camera_uniform(&context.device).as_entire_binding(),
                                                    light.get_buffer().as_entire_binding(),
                                                  ],
                                                  0);
        let bind_group2 = render_pipeline_wrapper.create_bind_group(&context.device,
                         &vec![
                         wgpu::BindingResource::TextureView(&grass_texture.view.unwrap()),
                         wgpu::BindingResource::Sampler(&grass_texture.sampler.unwrap()),
                         wgpu::BindingResource::TextureView(&rock_texture.view.unwrap()),
                         wgpu::BindingResource::Sampler(&rock_texture.sampler.unwrap())
                        ],
                                                  1);

        log::info!("Finished initialization.");

        Self {
            depth_texture: Some(Tex::create_depth_texture(&context, surface.config(), None)),
            camera: camera,
            buffer: create_cube(&context.device, 18.0, false),
            render_pipeline_wrapper: render_pipeline_wrapper,
            light: light,
            bind_group1: bind_group1, 
            bind_group2: bind_group2, 
        }
    }

    /// Render application.
    fn render(&mut self, context: &WGPUContext, view: &TextureView, _surface: &SurfaceWrapper ) {

            let clear_color = Some(wgpu::Color { r: 0.1, g: 0.0, b: 0.0, a: 1.0, });


            // If there is nothing to draw, this must be executed.
            let mut dummy_encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Dummy encoder") });
            {

                draw(&mut dummy_encoder,
                     view,
                     self.depth_texture.as_ref(),
                     &vec![&self.bind_group1, &self.bind_group2],
                     self.render_pipeline_wrapper.get_pipeline(),
                     &self.buffer,
                     0..36,
                     &clear_color,
                     true);
            }
            context.queue.submit(Some(dummy_encoder.finish()));
    }

    /// Resize window.
    fn resize(&mut self, context: &WGPUContext, surface_configuration: &wgpu::SurfaceConfiguration, new_size: winit::dpi::PhysicalSize<u32>) {

        self.depth_texture = Some(Texture::create_depth_texture(context, &surface_configuration, Some("depth-texture")));
        self.camera.resize(surface_configuration.width as f32, surface_configuration.height as f32);
    }

    /// Application update.
    fn update(&mut self, context: &WGPUContext, input_cache: &InputCache) {
        self.camera.update_from_input(&context.queue, &input_cache);
        // log::info!("{:?}", self.camera.get_view());
        // log::info!("{:?}", self.camera.get_position());
    }

    fn close(&mut self, _wgpu_context: &WGPUContext){ 
    }
}

fn main() {

    initialize_env_logger(&vec![("smoke".to_string(), LevelFilter::Info)]);
    run::<SmokeFeatures, BasicLoop, SmokeApp>("yeah");
}
