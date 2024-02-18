use crate::gpu_debugger::char_generator::CharProcessor;
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


#[derive(Eq, Hash, PartialEq)]
enum GpuBuffer {
    ArrowBuffer,
    ArrowParamsBuffer,
    AabbBuffer,
    AabbWireBuffer,
    DrawIndirectBuffer,
    DispatchIndirectBuffer,
    RenderBuffer,
}

pub struct GpuDebugger {
    buffers: HashMap<GpuBuffer, wgpu::Buffer>,
    char_processor: CharProcessor,
}

impl GpuDebugger {

    fn create_buffers(&mut self, device: &wgpu::Device,
                      max_number_of_arrows: u32,
                      max_number_of_aabbs: u32,
                      max_number_of_aabb_wires: u32,
                      max_number_of_vertices: u32) {


        // self.buffers.insert(
        //     GpuBuffer::RenderBuffer,
        //     device.create_buffer(&wgpu::BufferDescriptor {
        //         label: Some("gpu_debug draw buffer"),
        //         size: (max_number_of_vertices * size_of::<Vertex>() as u32) as u64,
        //         usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        //         mapped_at_creation: false,
        //     }));

    }
}
