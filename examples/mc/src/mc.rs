use engine::common_structs::DrawIndirect;
use engine::buffer::to_vec;
use engine::algorithms::mc::McParams;
use engine::algorithms::mc::MarchingCubes;
use engine::draw_commands::draw_indirect;
use crate::configuration::McFeatures;
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
use engine::core::WGPUContext;
use engine::core::Application;
use engine::input_cache::InputCache;
use engine::meshes::create_cube;
use engine::pipeline_stuff::custom_pipelines::default_render_shader_v4n4_camera_light_tex2;
use engine::noise_maker::NoiseMaker;

use engine::logger::initialize_env_logger; 
use log::LevelFilter;
mod configuration; 

/// Marching cubes application.
struct McApp {
    depth_texture: Option<Tex>,
    camera: Camera,
    buffer: wgpu::Buffer,
    render_pipeline_wrapper: RenderPipelineWrapper,
    #[allow(dead_code)] 
    light: LightBuffer,
    bind_group1: wgpu::BindGroup,
    bind_group2: wgpu::BindGroup,
    noise_maker: NoiseMaker,
    marching_cubes: MarchingCubes,
    output_buffer: wgpu::Buffer,
    calculator: f32,  
}

impl Application for McApp {

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
                                                    &camera.get_camera_uniform(&context.device).as_entire_binding(),
                                                    &light.get_buffer().as_entire_binding(),
                                                  ],
                                                  0);
        let bind_group2 = render_pipeline_wrapper.create_bind_group(&context.device,
                         &vec![
                         &wgpu::BindingResource::TextureView(&grass_texture.view.unwrap()),
                         &wgpu::BindingResource::Sampler(&grass_texture.sampler.unwrap()),
                         &wgpu::BindingResource::TextureView(&rock_texture.view.unwrap()),
                         &wgpu::BindingResource::Sampler(&rock_texture.sampler.unwrap())
                        ],
                                                  1);

        let noise_maker = NoiseMaker::init(
                  &context.device,
                  &"main".to_string(),
                  [256,128,256],
                  [1, 1, 1],
                  [1.0,1.0,1.0],
                  0.5,
                  2.0,
                  1.5);


        let output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mc output buffer"),
            size: 256*128*256*16 as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mc_params = McParams {
            base_position: [0.0, 0.0, 0.0, 1.0],
            isovalue: -15.0,
            cube_length: 1.0,
            future_usage1: 0.0,
            future_usage2: 0.0,
            noise_global_dimension: [256,
            128,
            256,
            0
            ],
            noise_local_dimension: [1,
            1,
            1,
            0
            ],
        };

        let marching_cubes = MarchingCubes::init_with_noise_buffer(&context.device,
                                                                   &mc_params,
                                                                   noise_maker.get_buffer(),
                                                                   &output_buffer);
        log::info!("Finished initialization.");

        Self {
            depth_texture: Some(Tex::create_depth_texture(&context, surface.config(), None)),
            camera: camera,
            buffer: create_cube(&context.device, 18.0, false),
            render_pipeline_wrapper: render_pipeline_wrapper,
            light: light,
            bind_group1: bind_group1,
            bind_group2: bind_group2,
            noise_maker: noise_maker,
            marching_cubes: marching_cubes,
            output_buffer: output_buffer,
            calculator: 0.0,
        }
    }

    /// Render application.
    fn render(&mut self, context: &WGPUContext, view: &TextureView, _surface: &SurfaceWrapper ) {

        let clear_color = Some(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0, });


        // If there is nothing to draw, this must be executed.
        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Dummy encoder") });

        draw_indirect(
            &mut encoder,
            &view,
            self.depth_texture.as_ref(),
            &vec![&self.bind_group1, &self.bind_group2],
            self.render_pipeline_wrapper.get_pipeline(),
            &self.output_buffer, // TODO: create this!
            self.marching_cubes.get_draw_indirect_buffer(),
            0,
            &clear_color,
            true
            );

        draw(&mut encoder,
             view,
             self.depth_texture.as_ref(),
             &vec![&self.bind_group1, &self.bind_group2],
             self.render_pipeline_wrapper.get_pipeline(),
             &self.buffer,
             0..36,
             &clear_color,
             false);


        context.queue.submit(Some(encoder.finish()));

        // Reset counter.
        self.marching_cubes.reset_counter_value(&context.queue);
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

        let mut encoder_command = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Noise & Mc encoder.") });

        self.noise_maker.dispatch(&mut encoder_command);
        self.marching_cubes.dispatch(&mut encoder_command, total_grid_count / 256, 1, 1);
        context.queue.submit(Some(encoder_command.finish()));

        self.calculator += 1.0;
        self.noise_maker.update_param_a(&context.queue, 5.0 * (self.calculator * 0.005).sin()); 
        self.noise_maker.update_param_b(&context.queue, 5.0 * (self.calculator * 0.001).cos()); 
    }

    fn close(&mut self, _wgpu_context: &WGPUContext){ 
    }
}

fn main() {

    initialize_env_logger(&vec![("mc".to_string(), LevelFilter::Info)]);
    run::<McFeatures, BasicLoop, McApp>("yeah");
}
