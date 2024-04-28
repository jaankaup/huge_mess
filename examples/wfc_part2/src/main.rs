use crate::wfc_misc::corner2;
use crate::wfc_misc::corner;
use crate::wfc_misc::test_data_ceiling_corner_2;
use crate::wfc_misc::test_data_ceiling_corner;
use crate::wfc_misc::test_data_floor_corner_3;
use crate::wfc_misc::test_data_floor_corner_2;
use crate::wfc_misc::test_data_floor_corner;
use crate::wfc_misc::test_data_empty;
use crate::wfc_misc::test_data_2x_ceiling_floor;
use crate::wfc_misc::test_data_ceiling;
use crate::wfc_misc::test_data_floor;
use crate::wfc_misc::WfcScene;
use crate::wfc_misc::test_data_v3;
use crate::wfc_misc::WfcBlock;
use crate::wfc_misc::Voxel;
use crate::wfc_misc::check_connections;
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
    voxels: HashMap<u32, Voxel>,
    wfc_engine: WfcScene,
}

impl WfcPart2App {
}

impl Application for WfcPart2App {

    /// Initialize application.
    fn init(context: &WGPUContext, surface: &SurfaceWrapper) -> Self {

        log::info!("Initializing WfcPart2App");

        log::info!("Creating camera.");

        // Create camera.
        let mut camera = Camera::new(surface.config().width as f32,
                                     surface.config().height as f32,
                                     (-45.0, 132.0, 38.0),
                                     (50.0, 0.0, 50.0)
        );
        camera.set_rotation_sensitivity(1.0);
        camera.set_movement_sensitivity(0.1);

        let draw_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("WfcPart2DrawBuffer"),
            size: 256*256*256*16 as u64,
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
            voxels: HashMap::new(),
            wfc_engine: WfcScene::init(32, 8, 32),
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

        if self.once {

            self.temp_arrows.push(Arrow {
                start_pos: [-40.0, 10.0, 0.0, 1.0],
                end_pos: [-40.0, 15.0, 0.0, 1.0],
                color: 0x00FF00FF,
                size: 0.5,
                _padding: [0,0],
            });
            self.temp_arrows.push(Arrow {
                start_pos: [-40.0, 10.0, 0.0, 1.0],
                end_pos: [-35.0, 10.0, 0.0, 1.0],
                color: 0xFF0000FF,
                size: 0.5,
                _padding: [0,0],
            });
            self.temp_arrows.push(Arrow {
                start_pos: [-40.0, 10.0, 0.0, 1.0],
                end_pos: [-40.0, 10.0, 5.0, 1.0],
                color: 0x0000FFFF,
                size: 0.5,
                _padding: [0,0],
            });


            let color: f32 = unsafe {transmute::<u32, f32>(0xFFFF00FF)};
            let color_red: f32 = unsafe {transmute::<u32, f32>(0xFF0000FF)};
            let color_90: f32 = unsafe {transmute::<u32, f32>(0x00FFFFFF)};
            let color_180: f32 = unsafe {transmute::<u32, f32>(0xFFFFFFFF)};
            let color_270: f32 = unsafe {transmute::<u32, f32>(0x0F0FFFFF)};

            let test = test_data_v3();
            let first_block = WfcBlock::init(0, 5, test.clone(), vec![]); 
            let second_block = first_block.create_rotation(0b10000, 1);
            let third_block = first_block.create_rotation( 0b100000, 2);
            let fourth_block = first_block.create_rotation(0b1000000, 3);
            let block_5 = first_block.create_rotation(0b10, 3);
            let block_6 = first_block.create_rotation(0b100, 3);
            let block_7 = first_block.create_rotation(0b1000, 3);
            let block_8 = first_block.create_rotation(0b1000000, 3);
            let block_9 = first_block.create_rotation(0b10000000, 3);
            let block_10 = first_block.create_rotation(0b100000000, 3);

            self.wfc_engine.insert_block_case(first_block);
            self.wfc_engine.insert_block_case(second_block);
            self.wfc_engine.insert_block_case(third_block);
            self.wfc_engine.insert_block_case(fourth_block);
            self.wfc_engine.insert_block_case(block_5);
            self.wfc_engine.insert_block_case(block_6);
            self.wfc_engine.insert_block_case(block_7);
            self.wfc_engine.insert_block_case(block_8);
            self.wfc_engine.insert_block_case(block_9);
            self.wfc_engine.insert_block_case(block_10);

            // Floor
            let floor = test_data_floor();
            self.wfc_engine.insert_block_case(WfcBlock::init(0, 5, floor.clone(), vec![]));

            // Ceiling
            let ceiling = test_data_ceiling(); 
            let ceiling_block = WfcBlock::init(0, 5, ceiling.clone(), vec![]);
            let ceiling_second = ceiling_block.create_rotation(0b10000, 1);
            let ceiling_third = ceiling_block.create_rotation(0b100000, 1);
            let ceiling_fourth = ceiling_block.create_rotation(0b1000000, 1);
            let ceiling_5 = ceiling_block.create_rotation(0b10, 1);
            let ceiling_6 = ceiling_block.create_rotation(0b100, 1);
            let ceiling_7 = ceiling_block.create_rotation(0b1000, 1);
            let ceiling_8 = ceiling_block.create_rotation(0b10000000, 1);
            let ceiling_9 = ceiling_block.create_rotation(0b10000000, 1);
            let ceiling_10 = ceiling_block.create_rotation(0b100000000, 1);

            self.wfc_engine.insert_block_case(ceiling_block);
            self.wfc_engine.insert_block_case(ceiling_second);
            self.wfc_engine.insert_block_case(ceiling_third);
            self.wfc_engine.insert_block_case(ceiling_fourth);
            self.wfc_engine.insert_block_case(ceiling_5);
            self.wfc_engine.insert_block_case(ceiling_6);
            self.wfc_engine.insert_block_case(ceiling_7);
            self.wfc_engine.insert_block_case(ceiling_8);
            self.wfc_engine.insert_block_case(ceiling_9);
            self.wfc_engine.insert_block_case(ceiling_10);

            // Ceiling x 2 + floor
            let c2f = test_data_2x_ceiling_floor();
            let c2f_0 = WfcBlock::init(0, 5, c2f.clone(), vec![]);
            let c2f_1 = c2f_0.create_rotation(0b10, 1);
            let c2f_2 = c2f_0.create_rotation(0b100, 1);
            let c2f_3 = c2f_0.create_rotation(0b1000, 1);
            let c2f_4 = c2f_0.create_rotation(0b10000, 1);
            let c2f_5 = c2f_0.create_rotation(0b100000, 1);
            let c2f_6 = c2f_0.create_rotation(0b1000000, 1);
            let c2f_7 = c2f_0.create_rotation(0b10000000, 1);
            let c2f_8 = c2f_0.create_rotation(0b10000000, 1);
            let c2f_9 = c2f_0.create_rotation(0b100000000, 1);

            self.wfc_engine.insert_block_case(c2f_0);
            self.wfc_engine.insert_block_case(c2f_1);
            self.wfc_engine.insert_block_case(c2f_2);
            self.wfc_engine.insert_block_case(c2f_3);
            self.wfc_engine.insert_block_case(c2f_4);
            self.wfc_engine.insert_block_case(c2f_5);
            self.wfc_engine.insert_block_case(c2f_6);
            self.wfc_engine.insert_block_case(c2f_7);
            self.wfc_engine.insert_block_case(c2f_8);
            self.wfc_engine.insert_block_case(c2f_9);

            let floor_corner = test_data_floor_corner();
            let floor_corner_0 = WfcBlock::init(0, 5, floor_corner.clone(), vec![]);
            let floor_corner_1 = floor_corner_0.create_rotation(0b10, 1);
            let floor_corner_2 = floor_corner_0.create_rotation(0b100, 1);
            let floor_corner_3 = floor_corner_0.create_rotation(0b1000, 1);
            let floor_corner_4 = floor_corner_0.create_rotation(0b10000, 1);
            let floor_corner_5 = floor_corner_0.create_rotation(0b100000, 1);
            let floor_corner_6 = floor_corner_0.create_rotation(0b1000000, 1);
            let floor_corner_7 = floor_corner_0.create_rotation(0b10000000, 1);
            let floor_corner_8 = floor_corner_0.create_rotation(0b100000000, 1);
            let floor_corner_9 = floor_corner_0.create_rotation(0b1000000000, 1);

            self.wfc_engine.insert_block_case(floor_corner_0);
            self.wfc_engine.insert_block_case(floor_corner_1);
            self.wfc_engine.insert_block_case(floor_corner_2);
            self.wfc_engine.insert_block_case(floor_corner_3);
            self.wfc_engine.insert_block_case(floor_corner_4);
            self.wfc_engine.insert_block_case(floor_corner_5);
            self.wfc_engine.insert_block_case(floor_corner_6);
            self.wfc_engine.insert_block_case(floor_corner_7);
            self.wfc_engine.insert_block_case(floor_corner_8);
            self.wfc_engine.insert_block_case(floor_corner_9);

            let floor_corner_2 = test_data_floor_corner_2();
            let floor_corner_2_0 = WfcBlock::init(0, 5, floor_corner_2.clone(), vec![]);
            let floor_corner_2_1 = floor_corner_2_0.create_rotation(0b10, 1);
            let floor_corner_2_2 = floor_corner_2_0.create_rotation(0b100, 1);
            let floor_corner_2_3 = floor_corner_2_0.create_rotation(0b1000, 1);
            let floor_corner_2_4 = floor_corner_2_0.create_rotation(0b10000, 1);
            let floor_corner_2_5 = floor_corner_2_0.create_rotation(0b100000, 1);
            let floor_corner_2_6 = floor_corner_2_0.create_rotation(0b1000000, 1);
            let floor_corner_2_7 = floor_corner_2_0.create_rotation(0b10000000, 1);
            let floor_corner_2_8 = floor_corner_2_0.create_rotation(0b100000000, 1);
            let floor_corner_2_9 = floor_corner_2_0.create_rotation(0b1000000000, 1);

            self.wfc_engine.insert_block_case(floor_corner_2_0);
            self.wfc_engine.insert_block_case(floor_corner_2_1);
            self.wfc_engine.insert_block_case(floor_corner_2_2);
            self.wfc_engine.insert_block_case(floor_corner_2_3);
            self.wfc_engine.insert_block_case(floor_corner_2_4);
            self.wfc_engine.insert_block_case(floor_corner_2_5);
            self.wfc_engine.insert_block_case(floor_corner_2_6);
            self.wfc_engine.insert_block_case(floor_corner_2_7);
            self.wfc_engine.insert_block_case(floor_corner_2_8);
            self.wfc_engine.insert_block_case(floor_corner_2_9);

            let floor_corner_3 = test_data_floor_corner_3();
            let floor_corner_3_0 = WfcBlock::init(0, 5, floor_corner_3.clone(), vec![]);
            let floor_corner_3_1 = floor_corner_3_0.create_rotation(0b10, 1);
            let floor_corner_3_2 = floor_corner_3_0.create_rotation(0b100, 1);
            let floor_corner_3_3 = floor_corner_3_0.create_rotation(0b1000, 1);
            let floor_corner_3_4 = floor_corner_3_0.create_rotation(0b10000, 1);
            let floor_corner_3_5 = floor_corner_3_0.create_rotation(0b100000, 1);
            let floor_corner_3_6 = floor_corner_3_0.create_rotation(0b1000000, 1);
            let floor_corner_3_7 = floor_corner_3_0.create_rotation(0b10000000, 1);
            let floor_corner_3_8 = floor_corner_3_0.create_rotation(0b100000000, 1);
            let floor_corner_3_9 = floor_corner_3_0.create_rotation(0b1000000000, 1);

            self.wfc_engine.insert_block_case(floor_corner_3_0);
            self.wfc_engine.insert_block_case(floor_corner_3_1);
            self.wfc_engine.insert_block_case(floor_corner_3_2);
            self.wfc_engine.insert_block_case(floor_corner_3_3);
            self.wfc_engine.insert_block_case(floor_corner_3_4);
            self.wfc_engine.insert_block_case(floor_corner_3_5);
            self.wfc_engine.insert_block_case(floor_corner_3_6);
            self.wfc_engine.insert_block_case(floor_corner_3_7);
            self.wfc_engine.insert_block_case(floor_corner_3_8);
            self.wfc_engine.insert_block_case(floor_corner_3_9);

            let ceiling_corner = test_data_ceiling_corner();
            let ceiling_corner_0 = WfcBlock::init(0, 5, ceiling_corner.clone(), vec![]);
            let ceiling_corner_1 = ceiling_corner_0.create_rotation(0b10, 1);
            let ceiling_corner_2 = ceiling_corner_0.create_rotation(0b100, 1);
            let ceiling_corner_3 = ceiling_corner_0.create_rotation(0b1000, 1);
            let ceiling_corner_4 = ceiling_corner_0.create_rotation(0b10000, 1);
            let ceiling_corner_5 = ceiling_corner_0.create_rotation(0b100000, 1);
            let ceiling_corner_6 = ceiling_corner_0.create_rotation(0b1000000, 1);
            let ceiling_corner_7 = ceiling_corner_0.create_rotation(0b10000000, 1);
            let ceiling_corner_8 = ceiling_corner_0.create_rotation(0b100000000, 1);
            let ceiling_corner_9 = ceiling_corner_0.create_rotation(0b1000000000, 1);

            self.wfc_engine.insert_block_case(ceiling_corner_0);
            self.wfc_engine.insert_block_case(ceiling_corner_1);
            self.wfc_engine.insert_block_case(ceiling_corner_2);
            self.wfc_engine.insert_block_case(ceiling_corner_3);
            self.wfc_engine.insert_block_case(ceiling_corner_4);
            self.wfc_engine.insert_block_case(ceiling_corner_5);
            self.wfc_engine.insert_block_case(ceiling_corner_6);
            self.wfc_engine.insert_block_case(ceiling_corner_7);
            self.wfc_engine.insert_block_case(ceiling_corner_8);
            self.wfc_engine.insert_block_case(ceiling_corner_9);

            let ceiling_corner_2 = test_data_ceiling_corner_2();
            let ceiling_corner_2_0 = WfcBlock::init(0, 5, ceiling_corner_2.clone(), vec![]);
            let ceiling_corner_2_1 = ceiling_corner_2_0.create_rotation(0b10, 1);
            let ceiling_corner_2_2 = ceiling_corner_2_0.create_rotation(0b100, 1);
            let ceiling_corner_2_3 = ceiling_corner_2_0.create_rotation(0b1000, 1);
            let ceiling_corner_2_4 = ceiling_corner_2_0.create_rotation(0b10000, 1);
            let ceiling_corner_2_5 = ceiling_corner_2_0.create_rotation(0b100000, 1);
            let ceiling_corner_2_6 = ceiling_corner_2_0.create_rotation(0b1000000, 1);
            let ceiling_corner_2_7 = ceiling_corner_2_0.create_rotation(0b10000000, 1);
            let ceiling_corner_2_8 = ceiling_corner_2_0.create_rotation(0b100000000, 1);
            let ceiling_corner_2_9 = ceiling_corner_2_0.create_rotation(0b1000000000, 1);

            self.wfc_engine.insert_block_case(ceiling_corner_2_0);
            self.wfc_engine.insert_block_case(ceiling_corner_2_1);
            self.wfc_engine.insert_block_case(ceiling_corner_2_2);
            self.wfc_engine.insert_block_case(ceiling_corner_2_3);
            self.wfc_engine.insert_block_case(ceiling_corner_2_4);
            self.wfc_engine.insert_block_case(ceiling_corner_2_5);
            self.wfc_engine.insert_block_case(ceiling_corner_2_6);
            self.wfc_engine.insert_block_case(ceiling_corner_2_7);
            self.wfc_engine.insert_block_case(ceiling_corner_2_8);
            self.wfc_engine.insert_block_case(ceiling_corner_2_9);

            let corner = corner();
            let corner_0 = WfcBlock::init(0, 5, corner.clone(), vec![]);
            let corner_1 = corner_0.create_rotation(0b10, 1);
            let corner_2 = corner_0.create_rotation(0b100, 1);
            let corner_3 = corner_0.create_rotation(0b1000, 1);
            let corner_4 = corner_0.create_rotation(0b10000, 1);
            let corner_5 = corner_0.create_rotation(0b100000, 1);
            let corner_6 = corner_0.create_rotation(0b1000000, 1);
            let corner_7 = corner_0.create_rotation(0b10000000, 1);
            let corner_8 = corner_0.create_rotation(0b100000000, 1);
            let corner_9 = corner_0.create_rotation(0b1000000000, 1);

            self.wfc_engine.insert_block_case(corner_0);
            self.wfc_engine.insert_block_case(corner_1);
            self.wfc_engine.insert_block_case(corner_2);
            self.wfc_engine.insert_block_case(corner_3);
            self.wfc_engine.insert_block_case(corner_4);
            self.wfc_engine.insert_block_case(corner_5);
            self.wfc_engine.insert_block_case(corner_6);
            self.wfc_engine.insert_block_case(corner_7);
            self.wfc_engine.insert_block_case(corner_8);
            self.wfc_engine.insert_block_case(corner_9);

            let corner2 = corner2();
            let corner2_0 = WfcBlock::init(0, 5, corner2.clone(), vec![]);
            let corner2_1 = corner2_0.create_rotation(0b10, 1);
            let corner2_2 = corner2_0.create_rotation(0b100, 1);
            let corner2_3 = corner2_0.create_rotation(0b1000, 1);
            let corner2_4 = corner2_0.create_rotation(0b10000, 1);
            let corner2_5 = corner2_0.create_rotation(0b100000, 1);
            let corner2_6 = corner2_0.create_rotation(0b1000000, 1);
            let corner2_7 = corner2_0.create_rotation(0b10000000, 1);
            let corner2_8 = corner2_0.create_rotation(0b100000000, 1);
            let corner2_9 = corner2_0.create_rotation(0b1000000000, 1);

            self.wfc_engine.insert_block_case(corner2_0);
            self.wfc_engine.insert_block_case(corner2_1);
            self.wfc_engine.insert_block_case(corner2_2);
            self.wfc_engine.insert_block_case(corner2_3);
            self.wfc_engine.insert_block_case(corner2_4);
            self.wfc_engine.insert_block_case(corner2_5);
            self.wfc_engine.insert_block_case(corner2_6);
            self.wfc_engine.insert_block_case(corner2_7);
            self.wfc_engine.insert_block_case(corner2_8);
            self.wfc_engine.insert_block_case(corner2_9);
            // Empty
            let empty = test_data_empty();
            let empty_block = WfcBlock::init(0, 5, empty.clone(), vec![]);
            self.wfc_engine.insert_block_case(empty_block);

            self.wfc_engine.add_seed_point(0, [5,5,5]);
            self.wfc_engine.expand_band_uvec3([5,5,5]);

            for i in 0..6000 {

            let candidates = self.wfc_engine.find_next_known_candidates().unwrap();
            // println!("CANDIDATES : {:?}", candidates);

            let mut rng = rand::thread_rng();
            let r: u32 = rng.gen_range(0..candidates.len()).try_into().unwrap();

            let next_candidate = candidates[r as usize];
            // println!("The next candidate is {:?}", next_candidate);
            self.wfc_engine.make_known(next_candidate);
            self.wfc_engine.expand_band(next_candidate);

            self.temp_aabbs.append(&mut self.wfc_engine.get_aabb_data());
            self.gpu_debugger.add_aabbs(&context.device, &context.queue, &self.temp_aabbs);
            self.temp_aabbs.clear();
                // self.some_counter += 1;
            }

            self.gpu_debugger.add_aabbs(&context.device, &context.queue, &self.temp_aabbs);
            self.gpu_debugger.add_arrows(&context.device, &context.queue, &self.temp_arrows);

            self.temp_arrows.clear();
            self.temp_aabbs.clear();

        }

        self.once = false;

        // if self.some_counter < 6000 {

        //     let candidates = self.wfc_engine.find_next_known_candidates().unwrap();
        //     // println!("CANDIDATES : {:?}", candidates);

        //     let mut rng = rand::thread_rng();
        //     let r: u32 = rng.gen_range(0..candidates.len()).try_into().unwrap();

        //     let next_candidate = candidates[r as usize];
        //     // println!("The next candidate is {:?}", next_candidate);
        //     self.wfc_engine.make_known(next_candidate);
        //     self.wfc_engine.expand_band(next_candidate);

        //     self.temp_aabbs.append(&mut self.wfc_engine.get_aabb_data());
        //     self.gpu_debugger.add_aabbs(&context.device, &context.queue, &self.temp_aabbs);
        //     self.temp_aabbs.clear();
        // }
        self.some_counter += 1;
    }

    fn close(&mut self, _wgpu_context: &WGPUContext){ 
    }
}

fn main() {

    initialize_env_logger(&vec![("mc".to_string(), LevelFilter::Info)]);
    run::<WfcPart2Features, BasicLoop, WfcPart2App>("yeah");
}
