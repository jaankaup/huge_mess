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

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Vertex {
    v: [f32; 4],
    n: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Triangle {
    a: Vertex,
    b: Vertex,
    c: Vertex,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct AABB {
    min: [f32; 4],
    max: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Arrow {
    start_pos: [f32 ; 4],
    end_pos: [f32 ; 4],
    color: u32,
    size: f32,
    _padding: [u32; 2]
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ArrowAabbParams{
    max_number_of_vertices: u32,
    iterator_start_index: u32,
    iterator_end_index: u32,
    element_type: u32, // 0 :: array, 1 :: aabb, 2 :: aabb wire
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct OtherRenderParams {
    scale_factor: f32,
}

impl_convert!{Arrow}

#[derive(Eq, Hash, PartialEq)]
enum GpuBuffer {
    CharBuffer,
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
}

impl GpuDebugger {

    fn create_buffers(&mut self, device: &wgpu::Device,
                      arrow_aabb_params: &ArrowAabbParams,
                      max_number_of_arrows: u32,
                      max_number_of_aabbs: u32,
                      max_number_of_aabb_wires: u32,
                      max_number_of_vertices: u32) {
        self.buffers.insert(
            GpuBuffer::DrawIndirectBuffer,
            buffer_from_data::<DrawIndirect>(
                &device,
                &vec![DrawIndirect{ vertex_count: 0, instance_count: 1, base_vertex: 0, base_instance: 0, } ; 1024],
                wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDIRECT,
                Some("Indirect draw buffer")
                ));

        self.buffers.insert(
            GpuBuffer::DispatchIndirectBuffer,
            buffer_from_data::<DispatchIndirect>(
                &device,
                &vec![DispatchIndirect{ x: 0, y: 0, z: 0, } ; 1024],
                wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDIRECT,
                Some("Indirect dispatch buffer")
                ));

        self.buffers.insert(
            GpuBuffer::ArrowBuffer,
            device.create_buffer(&wgpu::BufferDescriptor{
                label: Some("output_arrays buffer"),
                size: (max_number_of_arrows * std::mem::size_of::<Arrow>() as u32) as u64,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));

        self.buffers.insert(
            GpuBuffer::AabbBuffer,
            device.create_buffer(&wgpu::BufferDescriptor{
                label: Some("output_aabbs"),
                size: (max_number_of_aabbs * std::mem::size_of::<AABB>() as u32) as u64,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));

        self.buffers.insert(
            GpuBuffer::AabbWireBuffer,
            device.create_buffer(&wgpu::BufferDescriptor{
                label: Some("output_aabbs"),
                size: (max_number_of_aabb_wires * std::mem::size_of::<AABB>() as u32) as u64,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));

        self.buffers.insert(
            GpuBuffer::RenderBuffer,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("gpu_debug draw buffer"),
                size: (max_number_of_vertices * size_of::<Vertex>() as u32) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));

        self.buffers.insert(
            GpuBuffer::ArrowParamsBuffer,
            buffer_from_data::<ArrowAabbParams>(
                &device,
                &vec![*arrow_aabb_params],
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                None)
            );  
    }
}
