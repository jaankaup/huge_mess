use crate::texture::{
    Texture as Tex,
};
use bytemuck::Pod;
use bytemuck::Zeroable;
use std::borrow::Cow;
use std::mem::size_of;
use crate::pipelines::{
    ComputePipelineWrapper,
    BindGroupMapper,
};
use crate::bindgroups::{
    create_buffer_bindgroup_layout,
    create_uniform_bindgroup_layout,
};
use crate::draw_commands::draw_indirect;
use crate::common_structs::{
    DispatchIndirect,
    DrawIndirect,
};
use crate::buffer::to_vec;
use crate::buffer::buffer_from_data;
use crate::misc::Convert2Vec;
use crate::impl_convert;

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

impl_convert!{Arrow}

pub struct PrimitiveProcessor {

    aabb_pipeline_wrapper: ComputePipelineWrapper,
    aabb_bind_group: wgpu::BindGroup,
    arrow_buffer: wgpu::Buffer,
    aabb_buffer: wgpu::Buffer,
    aabb_wire_buffer: wgpu::Buffer,
    arrow_params_buffer: wgpu::Buffer,
}

impl PrimitiveProcessor {

    pub fn init(device: &wgpu::Device,
                render_buffer: &wgpu::Buffer,
                max_number_of_arrows: u32,
                max_number_of_aabbs: u32,
                max_number_of_aabb_wires: u32,
                max_number_of_vertices: u32) -> Self {

        let arrow_aabb_params = ArrowAabbParams {
            max_number_of_vertices: 5000 as u32,
            iterator_start_index: 0,
            iterator_end_index: 0,
            element_type: 0,
        };

        let arrow_buffer = device.create_buffer(&wgpu::BufferDescriptor{
                label: Some("array buffer"),
                size: (max_number_of_arrows * std::mem::size_of::<Arrow>() as u32) as u64,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        let aabb_buffer = device.create_buffer(&wgpu::BufferDescriptor{
                label: Some("aabb buffer"),
                size: (max_number_of_aabbs * std::mem::size_of::<AABB>() as u32) as u64,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        let aabb_wire_buffer = device.create_buffer(&wgpu::BufferDescriptor{
                label: Some("output_aabbs"),
                size: (max_number_of_aabb_wires * std::mem::size_of::<AABB>() as u32) as u64,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        let arrow_params_buffer = buffer_from_data::<ArrowAabbParams>(
                &device,
                &vec![arrow_aabb_params],
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                None);

        // Create here pipeline for arrow & aabb pipeline.
        //let aabb_pipeline_wrapper = ComputePipelineWrapper::init(
        //        device,
        //        &aabb_pipeline_layout,
        //        &wgsl_module,
        //        "main",
        //        bind_group_mapper,
        //        Some("Arrow aabb pipeline"));

        // Create here pipeline and bind group mapper for char preprocessor.
        let mut aabb_mapper = BindGroupMapper::init(device);
        aabb_mapper.insert(device, 0, &create_uniform_bindgroup_layout(0, wgpu::ShaderStages::COMPUTE));
        aabb_mapper.insert(device, 0, &create_buffer_bindgroup_layout(1, wgpu::ShaderStages::COMPUTE, false));
        aabb_mapper.insert(device, 0, &create_buffer_bindgroup_layout(2, wgpu::ShaderStages::COMPUTE, false));
        aabb_mapper.insert(device, 0, &create_buffer_bindgroup_layout(3, wgpu::ShaderStages::COMPUTE, false));
        aabb_mapper.insert(device, 0, &create_buffer_bindgroup_layout(4, wgpu::ShaderStages::COMPUTE, false));
        aabb_mapper.build_bind_group_layouts(device);

        // Create wgsl module.
        let aabb_wgsl_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Arrow aabb module"),
            source: wgpu::ShaderSource::Wgsl(
                Cow::Borrowed(include_str!("wgsl/char_preprocessor.wgsl"))),
        });

        // Create pipeline layout
        let aabb_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Preprocessor layout"),
            bind_group_layouts: &aabb_mapper.get_bind_group_layouts().iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        let aabb_pipeline_wrapper = ComputePipelineWrapper::init(
                device,
                &aabb_pipeline_layout,
                &aabb_wgsl_module,
                "main",
                aabb_mapper,
                Some("Arrow aabb pipeline"));

        // Create bindgroups.
        let aabb_bind_group = aabb_pipeline_wrapper.create_bind_group(
            device,
            &vec![
                &arrow_params_buffer.as_entire_binding(),
                &arrow_buffer.as_entire_binding(), // TODO: keep char params information in this struct.
                &aabb_buffer.as_entire_binding(),
                &aabb_wire_buffer.as_entire_binding(),
                &render_buffer.as_entire_binding(),
            ],
            0);

        Self {
            aabb_pipeline_wrapper: aabb_pipeline_wrapper,
            aabb_bind_group: aabb_bind_group,
            arrow_buffer: arrow_buffer,
            aabb_buffer: aabb_buffer,
            aabb_wire_buffer: aabb_wire_buffer,
            arrow_params_buffer: arrow_params_buffer,
        }
    }
}
