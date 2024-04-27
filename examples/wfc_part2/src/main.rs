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
            voxels: HashMap::new(),
            wfc_engine: WfcScene::init(32, 32, 32),
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

            self.wfc_engine.insert_block_case(first_block);
            self.wfc_engine.insert_block_case(second_block);
            self.wfc_engine.insert_block_case(third_block);
            self.wfc_engine.insert_block_case(fourth_block);

            self.wfc_engine.add_seed_point(0, [5,5,5]);
            self.wfc_engine.expand_band_uvec3([5,5,5]);

            self.temp_aabbs.append(&mut self.wfc_engine.get_aabb_data());
            // let mut base_position = [0.0, 0.0, 0.0];
            // for x in first_block.get_connection_data().iter() { 
            //     self.temp_aabbs.push(
            //         AABB {
            //             min: [x[0],       x[1]       , x[2]      , color],
            //             max: [x[0] + 1.0, x[1] + 1.0 , x[2] - 1.0, color],
            //         });
            // }
            // base_position[0] += 5.0;
            // for x in second_block.get_connection_data().iter() { 
            //     self.temp_aabbs.push(
            //         AABB {
            //             min: [base_position[0] + x[0],       x[1]       , x[2]      , color],
            //             max: [base_position[0] + x[0] + 1.0, x[1] + 1.0 , x[2] - 1.0, color],
            //         });
            // }
            // base_position[0] += 5.0;
            // for x in third_block.get_connection_data().iter() { 
            //     self.temp_aabbs.push(
            //         AABB {
            //             min: [base_position[0] + x[0],       x[1]       , x[2]      , color],
            //             max: [base_position[0] + x[0] + 1.0, x[1] + 1.0 , x[2] - 1.0, color],
            //         });
            // }
            // base_position[0] += 5.0;
            // for x in fourth_block.get_connection_data().iter() { 
            //     self.temp_aabbs.push(
            //         AABB {
            //             min: [base_position[0] + x[0],       x[1]       , x[2]      , color],
            //             max: [base_position[0] + x[0] + 1.0, x[1] + 1.0 , x[2] - 1.0, color],
            //         });
            // }
            // // let all_rotations = create_rotations(0b111111111, &test);
            // let mut base_position = [0.0, 0.0, 0.0];

            // let connections = check_connections(&test, &test, 0b1111111111);
            // println!("connections = {:?}", connections);
            //++ let mut voxel = Voxel::init(0, 0b1111111111, 5, 0.0, &test, &vec![]);
            //++ let all_rotations = voxel.get_all_rotations();
            //++ // Create hash for all rotations.
            //++ println!("HASH");
            //++ // let mut rotation_table = Vec::new(); 
            //++ for (i, rotation) in all_rotations.iter().enumerate() {
            //++     // Create a rotation for all cases. 
            //++     for b in 0..10 {
            //++         // println!("Rotation data for rotation case {:?}", b);
            //++         let mut rotated = create_rotations(1 << b, rotation)[0].iter().map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
            //++         rotated.sort();
            //++         // Search for original index that has same content so we create mapping.
            //++         let mut found = false;
            //++         for (original_index, r) in all_rotations.iter().enumerate() {

            //++             let mut original_sorted = (*r).iter().map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
            //++             original_sorted.sort();
            //++             if original_sorted == rotated {
            //++                 println!("{:?} => {:?}", b,  original_index);
            //++                 found = true;
            //++                 break;
            //++             }
            //++         }
            //++         if !found {
            //++             println!("PRKL");
            //++         }
            //++         //rotation_table.push(create_rotations(1 << b, rotation));
            //++     }
            //++     // Check the new index for 
            //++ }
            //++ println!("TABLE:::::::::::::::::::::::");
            //++ // println!("{:?}", rotation_table);
            //++ // Fix the roration table.
            //++ for r in 0..10 {
            //++     print!("{:?}   ", r); 
            //++ }
            //++ println!(""); 
            //++ // for r in rotation_table.iter() {
            //++ //     // for (i, original) in all_rotations.iter().enumerate() {
            //++ //     //     if original == r {
            //++ //     //         print!("{:?}   ", i); 
            //++ //     //         break;
            //++ //     //     }
            //++ //     //     println!("");
            //++ //     // }
            //++ // }

            //++ println!("END HASH");
            //++ voxel.add_rules(&voxel.clone());
            //++ self.voxels.insert(voxel.id, voxel);

            //++ // Get the rotations for it self.
            //++ // HashMap<u32, [u32 ; 6]> 
            //++ // let neighbors = self.voxels.get(&0).as_ref().unwrap().get_possible_neighbors(0b1);
            //++ let neighbors = self.voxels.get(&0).as_ref().unwrap().get_possible_neighbors(0b1);
            //++ let current_rotation_data = self.voxels.get(&0).as_ref().unwrap().get_rotated_connection_data(0b1);
            //++ for vec in current_rotation_data.iter() { 
            //++     for x in vec.iter() { 
            //++         self.temp_aabbs.push(
            //++             AABB {
            //++                 min: [x[0],       x[1]       , x[2]      , color],
            //++                 max: [x[0] + 1.0, x[1] + 1.0 , x[2] - 1.0, color],
            //++             });
            //++         }
            //++ }

            //++ // Iterate over all neighbors.
            //++ for (k, v) in neighbors.iter() {
            //++     // Get the reference to voxel.
            //++     let neighbor = self.voxels.get(&k).as_ref().unwrap().clone();
            //++     // println!("neighbor == {:?}", neighbor);
            //++     // println!("v == {:?}", v);

            //++     // Get directions for rendering 0 :: x+  1 :: x-  y+ :: 2  y- :: 3  z+ :: 4  z- :: 5
            //++     for (direction_index, cases_per_dir) in v.iter().enumerate() {

            //++         let mut base_position = [0.0, 0.0, 0.0];
            //++         println!("{:?} :: {:?}", direction_index, cases_per_dir);

            //++         // Vector of rendering cubes.
            //++         let rotations = create_rotations(*cases_per_dir, &neighbor.connection_data);

            //++         const base_increment: [[f32; 3]; 6]
            //++             = [[ 5.0,  0.0,  0.0],
            //++                [-5.0,  0.0,  0.0],
            //++                [ 0.0,  5.0,  0.0],
            //++                [ 0.0, -5.0,  0.0],
            //++                [ 0.0,  0.0,  5.0],
            //++                [ 0.0,  0.0, -5.0]];


            //++          base_position[0] += base_increment[direction_index][0];
            //++          base_position[1] += base_increment[direction_index][1];
            //++          base_position[2] += base_increment[direction_index][2];
            //++          
            //++          for sub_case in rotations.iter() {
            //++              for x in sub_case.iter() {
            //++                  self.temp_aabbs.push(
            //++                      AABB {
            //++                          min: [x[0] + base_position[0],       x[1]       + base_position[1] , x[2] + base_position[2], color_red],
            //++                          max: [x[0] + base_position[0] + 1.0, x[1] + 1.0 + base_position[1] , x[2] + base_position[2] - 1.0, color_red],
            //++                      });
            //++              }
            //++              base_position[0] += base_increment[direction_index][0];
            //++              base_position[1] += base_increment[direction_index][1];
            //++              base_position[2] += base_increment[direction_index][2];
            //++          }
            //++     }


            //++     //++     for c in 0..*cases_per_dir {
            //++     //++         let rotations = create_rotations(c, &neighbor.connection_data);

            //++     //++         // x+
            //++     //++         if direction_index == 0 {

            //++     //++             for rotation in rotations.iter() {
            //++     //++                 base_position[0] += 5.0;
            //++     //++                 for x in rotation.iter() {
            //++     //++                     self.temp_aabbs.push(
            //++     //++                         AABB {
            //++     //++                             min: [x[0] + base_position[0],       x[1]       + base_position[1] , x[2] + base_position[2], color_red],
            //++     //++                             max: [x[0] + base_position[0] + 1.0, x[1] + 1.0 + base_position[1] , x[2] + base_position[2] - 1.0, color_red],
            //++     //++                         });
            //++     //++                 }
            //++     //++             }
            //++     //++         }
            //++     //++         // // x-
            //++     //++         // if direction_index == 1 {
            //++     //++         //     base_position[0] -= 5.0;
            //++     //++         // }
            //++     //++         // // y+
            //++     //++         // if direction_index == 0 {
            //++     //++         //     base_position[1] += 5.0;
            //++     //++         // }
            //++     //++         // // y-
            //++     //++         // if direction_index == 1 {
            //++     //++         //     base_position[1] -= 5.0;
            //++     //++         // }
            //++     //++         // // z+
            //++     //++         // if direction_index == 0 {
            //++     //++         //     base_position[2] += 5.0;
            //++     //++         // }
            //++     //++         // // z-
            //++     //++         // if direction_index == 1 {
            //++     //++         //     base_position[2] -= 5.0;
            //++     //++         // }

            //++     //++     }


            //++     //++ }
            //++     // let rotated_voxel_data = neighbor.get_rotated_connection_data(
            //++     // for (index, rot) in v.enumerate() {
            //++     //     let rotations = create_rotations(rot);
            //++     // }
            //++ }
            // let rotations_x_plus_dir = create_rotations(connections[0], &test);

            // for x in test.iter() {
            //     self.temp_aabbs.push(
            //         AABB {
            //             min: [x[0] + base_position[0],       x[1]       + base_position[1] , x[2] + base_position[2], color],
            //             max: [x[0] + base_position[0] + 1.0, x[1] + 1.0 + base_position[1] , x[2] + base_position[2] - 1.0, color],
            //         });
            // }
            // base_position[0] += 5.0;

            // // Check the x- direction.

            // // for rotation in all_rotations.iter() {
            // for rotation in rotations_x_plus_dir.iter() {
            //     for x in rotation.iter() {  
            //         self.temp_aabbs.push(
            //             AABB {
            //                 min: [x[0] + base_position[0],       x[1]       + base_position[1] , x[2] + base_position[2], color_red],
            //                 max: [x[0] + base_position[0] + 1.0, x[1] + 1.0 + base_position[1] , x[2] + base_position[2] - 1.0, color_red],
            //             });
            //     }
            //     base_position[0] += 5.0;
            // }
            // let mut base_position = [-5.0, 0.0, 0.0];

            // let rotations_x_minus_dir = create_rotations(connections[1], &test);
            // for rotation in rotations_x_minus_dir.iter() {
            //     for x in rotation.iter() {  
            //         self.temp_aabbs.push(
            //             AABB {
            //                 min: [x[0] + base_position[0],       x[1]       + base_position[1] , x[2] + base_position[2], color_red],
            //                 max: [x[0] + base_position[0] + 1.0, x[1] + 1.0 + base_position[1] , x[2] + base_position[2] - 1.0, color_red],
            //             });
            //     }
            //     base_position[0] -= 5.0;
            // }
            // let mut base_position = [0.0, 5.0, 0.0];

            // let rotations_y_plus_dir = create_rotations(connections[2], &test);
            // for rotation in rotations_y_plus_dir.iter() {
            //     for x in rotation.iter() {
            //         self.temp_aabbs.push(
            //             AABB {
            //                 min: [x[0] + base_position[0],       x[1]       + base_position[1] , x[2] + base_position[2], color_red],
            //                 max: [x[0] + base_position[0] + 1.0, x[1] + 1.0 + base_position[1] , x[2] + base_position[2] - 1.0, color_red],
            //             });
            //     }
            //     base_position[1] += 5.0;
            // }

            // let mut base_position = [0.0, -5.0, 0.0];

            // let rotations_y_minus_dir = create_rotations(connections[3], &test);
            // for rotation in rotations_y_minus_dir.iter() {
            //     for x in rotation.iter() {
            //         self.temp_aabbs.push(
            //             AABB {
            //                 min: [x[0] + base_position[0],       x[1]       + base_position[1] , x[2] + base_position[2], color_red],
            //                 max: [x[0] + base_position[0] + 1.0, x[1] + 1.0 + base_position[1] , x[2] + base_position[2] - 1.0, color_red],
            //             });
            //     }
            //     base_position[1] -= 5.0;
            // }

            // let mut base_position = [0.0, 0.0, 5.0];

            // let rotations_z_plus_dir = create_rotations(connections[4], &test);
            // for rotation in rotations_z_plus_dir.iter() {
            //     for x in rotation.iter() {
            //         self.temp_aabbs.push(
            //             AABB {
            //                 min: [x[0] + base_position[0],       x[1]       + base_position[1] , x[2] + base_position[2], color_red],
            //                 max: [x[0] + base_position[0] + 1.0, x[1] + 1.0 + base_position[1] , x[2] + base_position[2] - 1.0, color_red],
            //             });
            //     }
            //     base_position[2] += 5.0;
            // }

            // let mut base_position = [0.0, 0.0, -5.0];

            // let rotations_z_minus_dir = create_rotations(connections[5], &test);
            // for rotation in rotations_z_minus_dir.iter() {
            //     for x in rotation.iter() {
            //         self.temp_aabbs.push(
            //             AABB {
            //                 min: [x[0] + base_position[0],       x[1]       + base_position[1] , x[2] + base_position[2], color_red],
            //                 max: [x[0] + base_position[0] + 1.0, x[1] + 1.0 + base_position[1] , x[2] + base_position[2] - 1.0, color_red],
            //             });
            //     }
            //     base_position[2] += -5.0;
            // }

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
