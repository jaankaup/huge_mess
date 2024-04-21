use std::cmp::Reverse;
use engine::wfc_test::Direction;
use engine::wfc_test::SceneNode;
use engine::misc::index_to_uvec3;
use engine::misc::uvec3_to_index;
use engine::wfc_test::WfcTag;
use rand::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BinaryHeap;
use engine::wfc_test::WfcData;
use engine::gpu_debugger::primitive_processor::Arrow;
use std::mem::transmute;
use engine::gpu_debugger::primitive_processor::AABB;
use engine::gpu_debugger::GpuDebugger;
use engine::common_structs::DrawIndirect;
use engine::buffer::to_vec;
use engine::draw_commands::draw_indirect;
use crate::configuration::WfcTestFeatures;
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

use engine::logger::initialize_env_logger; 
use log::LevelFilter;
mod configuration; 

const x_dim: u32 = 2;
const y_dim: u32 = 2;
const negative: f32 = -1.0;

#[derive(PartialOrd, Ord, Eq, PartialEq, Debug)]
struct BandCell {
    pub alternatives: u32,
    pub index: u32,
}

struct WfcPart2App {
    depth_texture: Option<Tex>,
    camera: Camera,
    draw_buffer: wgpu::Buffer,
    gpu_debugger: GpuDebugger,
    some_counter: u32,
    y_counter: u32,
    visited: bool,
    available_blocks: HashMap<String, WfcData>,
    once: bool,
    once2: bool,
    scene_nodes: Vec<Option<SceneNode>>,
    band: BinaryHeap<Reverse<BandCell>>,
    temp_aabbs: Vec<AABB>,
}

fn create_aabbs(wfc_block: &WfcData, block_size: f32, base_position: [f32 ; 3], color: f32) -> Vec<AABB> {

    assert!(block_size > 0.0);

    let mut result: Vec<AABB> = Vec::new(); 

    let positions = wfc_block.get_inner_locations();
    log::info!("{:?}", positions);

    // Scale and tranlate aabbs.
    for [x,y,z] in positions {
        let factor = block_size / wfc_block.dimensionX as f32 ; //wfc_block.dimensionX as f32 / block_size; 
        let mut scaled = [x as f32 * factor, y as f32 * factor, z as f32 * factor];
        result.push(
            AABB {
                min: [scaled[0] + base_position[0],          (scaled[2] + base_position[2])      , negative * (base_position[1] + scaled[1])         , color],
                max: [scaled[0] + base_position[0] + factor, (scaled[2] + base_position[2]) + 5.0, negative * (base_position[1] + factor + scaled[1]), color],
            });
    }
    result
}

impl WfcPart2App {
    // fn satisfies(&self, 
    fn check_alternatives(&mut self, index: u32) -> Vec<WfcData> {
        let neighbors = self.find_neigbors(index).iter().map(|x| (self.scene_nodes[x.0 as usize].clone(), x.1)).collect::<Vec<_>>();
        let possible_cases = WfcData::resolve_cases(
            &neighbors,
            &self.available_blocks,
            5, 
            5);
        println!("CASES {:?}", possible_cases);
        possible_cases
    }

    fn find_neigbors(&self, index: u32) -> Vec<(u32, Direction)> {

        let mut result = Vec::new();

        // The coordinate of this cell.
        let coordinate = index_to_uvec3(index, x_dim, y_dim);

        let x_minus = coordinate[0] as i32 - 1;
        let x_plus = coordinate[0] as i32 + 1;
        let y_minus = coordinate[1] as i32 - 1;
        let y_plus = coordinate[1] as i32 + 1;

        // Left
        if x_minus >= 0 {
            //result.push((uvec3_to_index(x_minus as u32, coordinate[1], 0, x_dim, y_dim) , Direction::Left)); 
            result.push((uvec3_to_index(x_minus as u32, coordinate[1], 0, x_dim, y_dim) , Direction::Right)); 
        }
        // Right
        if x_plus < x_dim.try_into().unwrap() {
            //result.push((uvec3_to_index(x_plus as u32, coordinate[1], 0, x_dim, y_dim), Direction::Right));
            result.push((uvec3_to_index(x_plus as u32, coordinate[1], 0, x_dim, y_dim), Direction::Left));
        }
        // Up
        if y_minus >= 0 {
            result.push((uvec3_to_index(coordinate[0], y_minus as u32 , 0, x_dim, y_dim), Direction::Top));
            //result.push((uvec3_to_index(coordinate[0], y_minus as u32 , 0, x_dim, y_dim), Direction::Bottom));
        }
        // Down
        if y_plus < y_dim.try_into().unwrap() {
            result.push((uvec3_to_index(coordinate[0], y_plus as u32 , 0, x_dim, y_dim), Direction::Bottom));
            //result.push((uvec3_to_index(coordinate[0], y_plus as u32 , 0, x_dim, y_dim), Direction::Top));
        }
        result
    }

    fn find_next_known(&mut self, context: &WGPUContext) -> Option<u32> {
        //if self.band.len() > 0 {
        //
        let color: f32 = if (self.some_counter & 1 == 0) ^ (self.y_counter & 1 == 0) { unsafe {transmute::<u32, f32>(0xFFFF00FF)} } else { unsafe {transmute::<u32, f32>(0x1100FFFF)}};
        let color_other: f32 = if (self.some_counter & 1 == 0) ^ (self.y_counter & 1 == 0) { unsafe {transmute::<u32, f32>(0x111110FF)} } else { unsafe {transmute::<u32, f32>(0x05100FFF)}};
        let mut next_id: Option<u32> = None;

        if let Some(next_known) = self.band.pop() {
            println!("next_known = {:?}", next_known);
            let mut next_cell = &mut self.scene_nodes[next_known.0.index as usize].as_mut().unwrap();
            next_id = Some(next_known.0.index);
            if next_cell.tag == WfcTag::Band { 
                let how_many_alternatives = next_cell.alternatives.len();

                let coordinate = index_to_uvec3(next_known.0.index, x_dim, y_dim); 

                let mut rng = rand::thread_rng();
                let r: u32 = rng.gen_range(0..how_many_alternatives).try_into().unwrap();
                // ERKKI
                next_cell.wfc_data.data = next_cell.alternatives[r as usize].data.clone();

                // self.scene_nodes[next_known.0.index as usize].as_mut().unwrap().wfc_data = next_cell.wfc_data.clone();
                // next_cell.wfc_data.data = next_cell.alternatives[r as usize].data.clone();
                let aabbs = create_aabbs(&next_cell.wfc_data, 2.0, [coordinate[0] as f32 * 2.0, coordinate[1] as f32 * 2.0, 0.0], color);
                for a in aabbs.iter() {
                    self.gpu_debugger.add_aabb(&context.device, &context.queue, &a);
                }

                let aabb = AABB {
                    // min: [coordinate[0] as f32 * 5.0 , 2.2, coordinate[1] as f32 * 5.0, color_other],
                    // max: [coordinate[0] as f32 * 5.0 , 2.2, coordinate[1] as f32 * 5.0 + 2.0, color_other],
                    min: [coordinate[0] as f32 * 2.0      , 0.2, negative * (coordinate[1] as f32 * 2.0), color_other],
                    max: [coordinate[0] as f32 * 2.0 + 2.0, 0.2, negative * (coordinate[1] as f32 * 2.0 + 2.0), color_other],
                };
                self.gpu_debugger.add_aabb(&context.device, &context.queue, &aabb);
                // let aabb = AABB {
                //     // min: [coordinate[0] as f32 * 5.0 , 2.2, coordinate[1] as f32 * 5.0, color_other],
                //     // max: [coordinate[0] as f32 * 5.0 , 2.2, coordinate[1] as f32 * 5.0 + 2.0, color_other],
                //     min: [10.0 as f32 * 5.0 , 2.2, 10.0 as f32 * 5.0, color_other],
                //     max: [10.0 as f32 * 5.0 , 2.2, 10.0 as f32 * 5.0 + 2.0, color_other],
                // };
                self.gpu_debugger.add_aabb(&context.device, &context.queue, &aabb);
                println!("coordinate[0] = {:?}, coordinate[1] = {:?}", coordinate[0] as f32, coordinate[1] as f32);
            }

            // println!("Next cell: {:?}", next_cell);


            // self.gpu_debugger.add_aabb(&context.device, &context.queue, &AABB {
            //     min: [self.some_counter as f32 * 4.0 + 0.1, 1.0, self.y_counter as f32 * 4.0 + 0.1, color],
            //     max: [self.some_counter as f32 * 4.0 + 4.1, 1.2, self.y_counter as f32 * 4.0 + 4.1, color],
            // });

        }
        //}
        next_id
    }

    fn add_neighbors_to_band(&mut self, index: u32, context: &WGPUContext) {
        // Find neighbors. Add far nodes to the narrow band.         

        let mut hash: HashSet<[u32; 2]> = HashSet::new();

        let mut neigbors = self.find_neigbors(index);

        for n in neigbors.iter_mut() {
            let neighbor = self.scene_nodes[n.0 as usize].clone();
            let alternatives = self.check_alternatives(n.0);
            let alternatives_count = alternatives.len();
            if neighbor.is_none() {
                self.scene_nodes[n.0 as usize] = Some(SceneNode {
                    tag: WfcTag::Band,
                    index: n.0,
                    wfc_data: WfcData::init(5,5,1),
                    alternatives: alternatives,
                });
                self.band.push(Reverse(BandCell { alternatives: alternatives_count as u32, index: n.0, }));
            }
        }
        println!("BANDI SIZE {:?}", self.band.len());
    }
}

impl Application for WfcPart2App {

    /// Initialize application.
    fn init(context: &WGPUContext, surface: &SurfaceWrapper) -> Self {

        let mut available_blocks: HashMap<String, WfcData> = HashMap::new();
        let empty = WfcData::init(5,5,1);
        available_blocks.insert(empty.toString(), empty.clone());
        let mut test = WfcData::init(5,5,1);
        test.write(0,0,0,1);
        test.write(1,0,0,1);
        test.write(2,0,0,1);
        test.write(3,0,0,1);
        test.write(4,0,0,1);
        test.write(4,1,0,1);
        test.write(4,2,0,1);
        test.write(4,3,0,1);
        test.write(4,4,0,1);

        available_blocks.insert(test.toString(), test.clone());

        let test_rotated = test.rotate90();
        available_blocks.insert(test_rotated.toString(), test_rotated.clone());

        let test_rotated_again = test_rotated.rotate90();
        available_blocks.insert(test_rotated_again.toString(), test_rotated_again.clone());

        let test_rotated_again_again = test_rotated_again.rotate90();
        available_blocks.insert(test_rotated_again_again.toString(), test_rotated_again_again.clone());

        // test.print();
        // test_rotated.print();
        // test_rotated_again.print();
        // test_rotated_again_again.print();

        // let mut line = WfcData::init(5,5,1);
        // line.write(0,0,0,1);
        // line.write(1,0,0,1);
        // line.write(2,0,0,1);
        // line.write(3,0,0,1);
        // line.write(4,0,0,1);

        // let line_2 = line.rotate90();
        // let line_3 = line_2.rotate90();
        // let line_4 = line_3.rotate90();

        // available_blocks.insert(line.toString(), line.clone());
        // available_blocks.insert(line_2.toString(), line_2.clone());
        // available_blocks.insert(line_3.toString(), line_3.clone());
        // available_blocks.insert(line_4.toString(), line_4.clone());

        // let mut line_pah = WfcData::init(5,5,1);
        // line.write(0,0,0,1);
        // line.write(1,0,0,1);
        // line.write(2,0,0,1);
        // line.write(3,0,0,1);
        // line.write(4,0,0,1);
        // line.write(2,0,0,1);
        // line.write(2,1,0,1);
        // line.write(2,2,0,1);
        // line.write(2,3,0,1);
        // line.write(2,4,0,1);

        // let line_pah_2 = line.rotate90();
        // let line_pah_3 = line_pah_2.rotate90();
        // let line_pah_4 = line_pah_3.rotate90();

        // available_blocks.insert(line_pah.toString(), line_pah.clone());
        // available_blocks.insert(line_pah_2.toString(), line_pah_2.clone());
        // available_blocks.insert(line_pah_3.toString(), line_pah_3.clone());
        // available_blocks.insert(line_pah_4.toString(), line_pah_4.clone());

        log::info!("{:?}", available_blocks);

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
            label: Some("Aabb draw buffer"),
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

        let mut scene_nodes: Vec<Option<SceneNode>> = vec![None; (x_dim * y_dim).try_into().unwrap()];

        // let mut aabb_grid: Vec<AABB> = Vec::new();

        // for i in 0..300 {
        //     for j in 0..300 {
        //         let color: f32 = if (i & 1 == 0) ^ (j & 1 == 0) { unsafe {transmute::<u32, f32>(0xFFFF00FF)} } else { unsafe {transmute::<u32, f32>(0x1100FFFF)}};
        //         aabb_grid.push(
        //         AABB {
        //             min: [i as f32 * 4.0 + 0.1, 1.0, j as f32 * 4.0 + 0.1, color],
        //             max: [i as f32 * 4.0 + 4.1, 1.2, j as f32 * 4.0 + 4.1, color],
        //         });
        //     }
        // }

        // gpu_debugger.add_aabbs(&context.device, &context.queue, &aabb_grid);

        log::info!("Finished initialization.");

        Self {
            depth_texture: Some(Tex::create_depth_texture(&context, surface.config(), None)),
            camera: camera,
            draw_buffer: draw_buffer,
            gpu_debugger: gpu_debugger,
            some_counter: 0,
            y_counter: 2,
            visited: false,
            available_blocks: available_blocks,
            once: true,
            once2: true,
            scene_nodes: scene_nodes, 
            band: BinaryHeap::new(),
            temp_aabbs: Vec::new(),
        }
    }

    /// Render application.
    fn render(&mut self, context: &WGPUContext, view: &TextureView, _surface: &SurfaceWrapper ) {

        // let aabb01 = AABB {
        //     min: [1.0, 1.0, 1.0, 23423321.0],
        //     max: [5.0, 5.0, 5.0, 23423321.0],
        // };

        let clear_color = Some(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0, });

        let mut clear = true;

        // log::info!("Rendering");
        self.gpu_debugger.render(
                  &context.device,
                  &context.queue,
                  view,
                  &self.draw_buffer,
                  &self.depth_texture.as_ref().unwrap(), //depth_texture: &Tex,
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
        if self.once2 {
            let mut rng = rand::thread_rng();
            let starting_index: u32 = rng.gen_range(0..(x_dim*y_dim)).try_into().unwrap();

            self.scene_nodes[starting_index as usize] = Some(SceneNode { tag: WfcTag::Known, index: starting_index, wfc_data: WfcData::init(5,5,1), alternatives: Vec::new(), }); 
            self.add_neighbors_to_band(starting_index, context);
            let color_starting_point: f32 = unsafe {transmute::<u32, f32>(0x1100FFFF)};
            let color: f32 = unsafe {transmute::<u32, f32>(0x050011FF)};
            let coordinate = index_to_uvec3(starting_index, x_dim, y_dim); 
            let aabb = AABB {
                min: [coordinate[0] as f32 * 2.0, 0.2,       negative * (coordinate[1] as f32 * 2.0), color_starting_point],
                max: [coordinate[0] as f32 * 2.0 + 2.0, 0.2, negative * (coordinate[1] as f32 * 2.0 + 2.0), color_starting_point],
            };
            self.gpu_debugger.add_aabb(&context.device, &context.queue, &aabb);

            let r: usize = rng.gen_range(0..self.available_blocks.len()).try_into().unwrap();
            let mut random_data: Option<WfcData> = None;
            for (i, data) in self.available_blocks.iter().enumerate() {
                if i == r {
                    random_data = Some(data.1.clone());
                    break;
                }
            }
            self.scene_nodes[starting_index as usize].as_mut().unwrap().wfc_data = random_data.unwrap(); // next_cell.alternatives[r as usize].data.clone();
            // next_cell.wfc_data.data = next_cell.alternatives[r as usize].data.clone();
            let aabbs = create_aabbs(&self.scene_nodes[starting_index as usize].as_ref().unwrap().wfc_data, 2.0, [coordinate[0] as f32 * 2.0, coordinate[1] as f32 * 2.0, 0.0], color);
            for a in aabbs.iter() {
                self.gpu_debugger.add_aabb(&context.device, &context.queue, &a);
            }
        }

        self.once2 = false;

        let next_id = self.find_next_known(context);
        if next_id.is_some() {
            self.add_neighbors_to_band(next_id.unwrap(), context);
            //self.add_neighbors_to_band(next_id.unwrap(), context);
        }
        // Check the narrow band.
        //let narrow_band = self.scene_nodes.iter().filter(|x| x.is_some() && x.as_ref().unwrap().tag == WfcTag::Band).collect::<Vec<_>>();
        //let neighbors: HashSet<u32> = HashSet::new(); 

        // for node in narrow_band.iter() {
        //     // let coordinate = index_to_uvec3(index: u32, dim_x: u32, dim_y: u32) -> [u32 ; 3]; 
        // }

        // The active list is empty.
        // if narrow_band.len() == 0 {
        //     self.once = true; 
        // }
        // else {
        //     self.once = false;    

        // }

        if self.once {

        }

        // log::info!("Add aabb");
        // if self.once2 {
        //     let color: f32 = if (self.some_counter & 1 == 0) ^ (self.y_counter & 1 == 0) { unsafe {transmute::<u32, f32>(0xFFFF00FF)} } else { unsafe {transmute::<u32, f32>(0x1100FFFF)}};
        //     for (i, block) in self.available_blocks.iter().enumerate() {
        //         let aabbs = create_aabbs(&block.1, 2.0, [(2*i) as f32, 0.0, 0.0], color);
        //         println!("aabbs...");
        //         println!("{:?}", aabbs);
        //         for a in aabbs.iter() {
        //             self.gpu_debugger.add_aabb(&context.device, &context.queue, &a);
        //         }
        //     }
        // }
        // self.once2 = false;
        // log::info!("Add arrow");
        // self.gpu_debugger.add_arrow(&context.device, &context.queue, &Arrow {
        //     start_pos: [self.some_counter as f32 * 4.0 + 1.0, 8.0, self.y_counter as f32 * 4.0 + 1.0, 1.0],
        //     end_pos: [self.some_counter as f32 * 4.0 + 4.0, 122.0, self.y_counter as f32 * 4.0 + 4.0, 1.0],
        //     //color: 0xFF0000FF,
        //     color: if (self.some_counter & 1 == 0) ^ (self.y_counter & 1 == 0) {0xFFFF00FF} else {0x1100FFFF},
        //     size: 0.5,
        //     _padding: [0,0],
        // });
        // self.some_counter += 1;
        // if self.some_counter == 128 { self.some_counter = 0; self.y_counter += 1;  }
        // if self.some_counter < 16 { 
        //     if (self.y_counter < 16) {
        //         log::info!("Add arrow");
        //         self.gpu_debugger.add_arrow(&context.device, &context.queue, &Arrow {
        //             start_pos: [self.some_counter as f32 * 5.0 + 1.0, 8.0, self.y_counter as f32 * 5.0 + 1.0, 1.0],
        //             end_pos: [self.some_counter as f32 * 5.0 + 4.0, 122.0, self.y_counter as f32 * 5.0 + 4.0, 1.0],
        //             color: 0xFF0000FF,
        //             size: 0.5,
        //             _padding: [0,0],
        //         });
        //     }
        //     else {
        //         log::info!("Add aabb");
        //         let color: f32 = if (self.some_counter & 1 == 0) ^ (self.y_counter & 1 == 0) { unsafe {transmute::<u32, f32>(0xFFFF00FF)} } else { unsafe {transmute::<u32, f32>(0x1100FFFF)}};
        //         self.gpu_debugger.add_aabb(&context.device, &context.queue, &AABB {
        //             min: [self.some_counter as f32 * 4.0 + 0.1, 1.0, self.y_counter as f32 * 4.0 + 0.1, color],
        //             max: [self.some_counter as f32 * 4.0 + 4.1, 1.2, self.y_counter as f32 * 4.0 + 4.1, color],
        //         });
        //     }

        self.some_counter += 1;
        //     if self.some_counter == 2 { self.some_counter = 0; self.y_counter += 1; if self.y_counter > 2 { self.y_counter = 0; }  }
        // }
    }

    fn close(&mut self, _wgpu_context: &WGPUContext){ 
    }
}

fn main() {

    initialize_env_logger(&vec![("mc".to_string(), LevelFilter::Info)]);
    run::<WfcTestFeatures, BasicLoop, WfcPart2App>("yeah");
}
