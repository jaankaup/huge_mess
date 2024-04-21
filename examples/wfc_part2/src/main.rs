use std::cmp::Reverse;
use engine::wfc_test::Direction;
use engine::wfc_test::SceneNode;
use engine::misc::index_to_uvec3;
use engine::misc::uvec3_to_index;
use rand::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BinaryHeap;
use std::mem::transmute;
use engine::gpu_debugger::primitive_processor::Arrow;
use engine::gpu_debugger::primitive_processor::AABB;
use engine::gpu_debugger::GpuDebugger;
use engine::common_structs::DrawIndirect;
use engine::buffer::to_vec;
use engine::draw_commands::draw_indirect;
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

use crate::configuration::WfcPart2Features;
use crate::wfc_misc::{test_data, rotate_90z, rotate_180z, rotate_270z, rotate_90x, rotate_180x, rotate_270x, rotate_90y, rotate_180y, rotate_270y, create_rotations};

use engine::logger::initialize_env_logger; 
use log::LevelFilter;
mod configuration; 
mod wfc_misc; 

const x_dim: u32 = 2;
const y_dim: u32 = 2;
const negative: f32 = -1.0;

struct WfcPart2App {
    depth_texture: Option<Tex>,
    camera: Camera,
    draw_buffer: wgpu::Buffer,
    gpu_debugger: GpuDebugger,
    some_counter: u32,
    once: bool,
    // band: BinaryHeap<Reverse<BandCell>>,
    temp_aabbs: Vec<AABB>,
    temp_arrows: Vec<Arrow>,
}

// fn create_aabbs(wfc_block: &WfcData, block_size: f32, base_position: [f32 ; 3], color: f32) -> Vec<AABB> {
// 
//     assert!(block_size > 0.0);
// 
//     let mut result: Vec<AABB> = Vec::new(); 
// 
//     let positions = wfc_block.get_inner_locations();
//     log::info!("{:?}", positions);
// 
//     // Scale and tranlate aabbs.
//     for [x,y,z] in positions {
//         let factor = block_size / wfc_block.dimensionX as f32 ; //wfc_block.dimensionX as f32 / block_size; 
//         let mut scaled = [x as f32 * factor, y as f32 * factor, z as f32 * factor];
//         result.push(
//             AABB {
//                 min: [scaled[0] + base_position[0],          (scaled[2] + base_position[2])      , negative * (base_position[1] + scaled[1])         , color],
//                 max: [scaled[0] + base_position[0] + factor, (scaled[2] + base_position[2]) + 5.0, negative * (base_position[1] + factor + scaled[1]), color],
//             });
//     }
//     result
// }

impl WfcPart2App {
    // fn create_test_data(&self, base_position: &[f32 ; 4]) -> Vec<[f32; 4]> {
    //     let mut result = Vec::new();
    //     // let color: f32 = unsafe {transmute::<u32, f32>(0xFFFF00FF)};
    //     for x in test_data().iter() {
    //         result.push(*x);
    //         // self.temp_aabbs.push(
    //         //     AABB {
    //         //         min: [x[0] + base_position[0],       x[1]       , x[2], color],
    //         //         max: [x[0] + base_position[0] + 1.0, x[1] + 1.0 , x[2] - 1.0, color],
    //         //     });
    //     }
    //     result
    // }
}

impl Application for WfcPart2App {

    /// Initialize application.
    fn init(context: &WGPUContext, surface: &SurfaceWrapper) -> Self {

        log::info!("Initializing WfcPart2App");

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
            label: Some("WfcPart2DrawBuffer"),
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

        log::info!("Finished initialization.");

        Self {
            depth_texture: Some(Tex::create_depth_texture(&context, surface.config(), None)),
            camera: camera,
            draw_buffer: draw_buffer,
            gpu_debugger: gpu_debugger,
            some_counter: 0,
            once: true,
            // band: BinaryHeap::new(),
            temp_aabbs: Vec::new(),
            temp_arrows: Vec::new(),
        }
    }

    /// Render application.
    fn render(&mut self, context: &WGPUContext, view: &TextureView, _surface: &SurfaceWrapper ) {


        let clear_color = Some(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0, });

        let mut clear = true;

        self.gpu_debugger.render(
                  &context.device,
                  &context.queue,
                  view,
                  &self.draw_buffer,
                  &self.depth_texture.as_ref().unwrap(),
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

        // Once, generate a random seed point.
        // if self.once2 {
        //     let mut rng = rand::thread_rng();
        //     let starting_index: u32 = rng.gen_range(0..(x_dim*y_dim)).try_into().unwrap();

        // }

        if self.once {

            self.temp_arrows.push(Arrow {
                start_pos: [0.0, 10.0, 0.0, 1.0],
                end_pos: [0.0, 15.0, 0.0, 1.0],
                color: 0x00FF00FF,
                size: 0.5,
                _padding: [0,0],
            });
            self.temp_arrows.push(Arrow {
                start_pos: [0.0, 10.0, 0.0, 1.0],
                end_pos: [5.0, 10.0, 0.0, 1.0],
                color: 0xFF0000FF,
                size: 0.5,
                _padding: [0,0],
            });
            self.temp_arrows.push(Arrow {
                start_pos: [0.0, 10.0, 0.0, 1.0],
                end_pos: [0.0, 10.0, 5.0, 1.0],
                color: 0x0000FFFF,
                size: 0.5,
                _padding: [0,0],
            });


            let color: f32 = unsafe {transmute::<u32, f32>(0xFFFF00FF)};
            let color_90: f32 = unsafe {transmute::<u32, f32>(0x00FFFFFF)};
            let color_180: f32 = unsafe {transmute::<u32, f32>(0xFFFFFFFF)};
            let color_270: f32 = unsafe {transmute::<u32, f32>(0x0F0FFFFF)};

            let test   = test_data(0xFFFF00FF);
            let all_rotations = create_rotations(0b111111111, &test);
            let mut base_position    =   [0.0, 0.0, 0.0];

            for rotation in all_rotations.iter() {
                for x in rotation.iter() {  
                    self.temp_aabbs.push(
                        AABB {
                            min: [x[0] + base_position[0],       x[1]       + base_position[1] , x[2] + base_position[2], color],
                            max: [x[0] + base_position[0] + 1.0, x[1] + 1.0 + base_position[1] , x[2] + base_position[2] - 1.0, color],
                        });
                }
                base_position[0] += 5.0;
            }
            self.gpu_debugger.add_aabbs(&context.device, &context.queue, &self.temp_aabbs);
            self.gpu_debugger.add_arrows(&context.device, &context.queue, &self.temp_arrows);
        }

        self.once = false;

        self.some_counter += 1;
    }

    fn close(&mut self, _wgpu_context: &WGPUContext){ 
    }
}

fn main() {

    initialize_env_logger(&vec![("mc".to_string(), LevelFilter::Info)]);
    run::<WfcPart2Features, BasicLoop, WfcPart2App>("yeah");
}
