use bytemuck::{Pod, Zeroable};
// use std::marker::PhantomData;
use std::mem::size_of;
use engine::misc::index_to_uvec3;
use engine::misc::uvec3_to_index;
use engine::subgroup_test::MultiLevelScan;
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

use crate::configuration::RadixFeatures;

use engine::logger::initialize_env_logger; 
use log::LevelFilter;
mod configuration; 

const x_dim: u32 = 2;
const y_dim: u32 = 2;
const negative: f32 = -1.0;

struct RadixApp {
    depth_texture: Option<Tex>,
    camera: Camera,
    draw_buffer: wgpu::Buffer,
    gpu_debugger: GpuDebugger,
    some_counter: u32,
    once: bool,
    temp_aabbs: Vec<AABB>,
    temp_arrows: Vec<Arrow>,
    multi_level_scan: MultiLevelScan, 
    // scan_params: wgpu::Buffer,
    // input_data: wgpu::Buffer,
}

#[derive(Debug)]
pub struct RadixRequirements {
    input_and_auxiliary_memory: u32,
    bucket_histograms: u32,
    block_histograms: u32,
    block_assignments: u32,
    local_sort_bucket_assigments: u32,
}

impl RadixRequirements {

    /// Calculate radix sort memory requirements in bytes.
    /// n = number of keys
    /// k = number of bits per key
    /// r = radix
    /// kpb = keys per block
    /// lst = local sort threshold
    pub fn init(n: u32, k: u32, r: u32, kpb: u32, lst: u32, merge_threshold: u32) -> Self {
        Self {
            input_and_auxiliary_memory: 2 * n * k / 8,
            bucket_histograms: 4 * r * n / merge_threshold,
            block_histograms:  4 * r * n / kpb + n / merge_threshold,
            block_assignments: 2 * 16 * (n / kpb + n / merge_threshold),
            local_sort_bucket_assigments: 12 * (2 * n / lst + n/merge_threshold).min(r * n/merge_threshold), 
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct InputData {
    test_data: f32,
}

impl RadixApp {
}

impl Application for RadixApp {

    /// Initialize application.
    fn init(context: &WGPUContext, surface: &SurfaceWrapper) -> Self {

        log::info!("Initializing RadixApp");

        log::info!("Creating camera.");
        println!("min_subgroup_size == {:?}", context.adapter.limits().min_subgroup_size);
        println!("max_subgroup_size == {:?}", context.adapter.limits().max_subgroup_size);

        let warp_test = MultiLevelScan::init(&context.device);

        // Create camera.
        let mut camera = Camera::new(surface.config().width as f32,
                                     surface.config().height as f32,
                                     (-45.0, 132.0, 38.0),
                                     (50.0, 0.0, 50.0)
        );
        camera.set_rotation_sensitivity(1.0);
        camera.set_movement_sensitivity(0.1);

        let draw_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RadixDrawBuffer"),
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
        let multi_level_scan = MultiLevelScan::init(&context.device);

        log::info!("Finished initialization.");


        Self {
            depth_texture: Some(Tex::create_depth_texture(&context, surface.config(), None)),
            camera: camera,
            draw_buffer: draw_buffer,
            gpu_debugger: gpu_debugger,
            some_counter: 0,
            once: true,
            temp_aabbs: Vec::new(),
            temp_arrows: Vec::new(),
            multi_level_scan: multi_level_scan, 
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

            self.gpu_debugger.add_aabbs(&context.device, &context.queue, &self.temp_aabbs);
            self.gpu_debugger.add_arrows(&context.device, &context.queue, &self.temp_arrows);

            self.temp_arrows.clear();
            self.temp_aabbs.clear();

        let req = RadixRequirements::init(100000, 32, 256, 6912, 9216, 3000);
        println!("req == {:?}", req);


        self.once = false;

        self.some_counter += 1;
    }
}

    fn close(&mut self, _wgpu_context: &WGPUContext){ 
    }
}

fn main() {

    initialize_env_logger(&vec![("mc".to_string(), LevelFilter::Info)]);
    run::<RadixFeatures, BasicLoop, RadixApp>("yeah");
}
