use crate::gpu_debugger::char_generator::CharProcessor;
use crate::gpu_debugger::primitive_processor::PrimitiveProcessor;
use crate::buffer::buffer_from_data;
use std::mem::size_of;
use std::collections::HashMap;
use crate::misc::Convert2Vec;
use crate::impl_convert;
use crate::common_structs::{
    DrawIndirect,
    DispatchIndirect,
};
use bytemuck::{Pod, Zeroable};

mod char_generator;
mod primitive_processor;


#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct OtherRenderParams {
    scale_factor: f32,
}

pub struct GpuDebugger {
    primitive_processor: PrimitiveProcessor,
    char_processor: CharProcessor,
    max_number_of_vertices: u32,
    thread_count: u32,
    // vvvc renderer.
}

impl GpuDebugger {

    pub fn init(device: &wgpu::Device,
                render_buffer: &wgpu::Buffer,
                camera_buffer: &wgpu::Buffer,
                max_number_of_arrows: u32,
                max_number_of_aabbs: u32,
                max_number_of_aabb_wires: u32,
                max_number_of_vertices: u32,
                max_number_of_chars: u32,
                max_points_per_char: u32,
                thread_count: u32) -> Self {

        let primitive_processor = PrimitiveProcessor::init(
                device,
                render_buffer,
                max_number_of_arrows,
                max_number_of_aabbs,
                max_number_of_aabb_wires,
                max_number_of_vertices);

        let char_processor = CharProcessor::init(
                device,
                render_buffer,
                camera_buffer,
                max_number_of_chars,
                max_points_per_char,
                max_number_of_vertices);

        // vvvc renderer.


        Self {
            primitive_processor: primitive_processor,
            char_processor: char_processor,
            max_number_of_vertices: max_number_of_vertices,
            thread_count: thread_count,
        }
    }
}
