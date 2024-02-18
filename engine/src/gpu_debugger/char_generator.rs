use bytemuck::Pod;
use bytemuck::Zeroable;
use std::borrow::Cow;
use crate::pipelines::{
    ComputePipelineWrapper,
    BindGroupMapper,
};
use crate::bindgroups::{
    create_buffer_bindgroup_layout,
    create_uniform_bindgroup_layout,
};
use crate::common_structs::DispatchIndirect;
use crate::buffer::buffer_from_data;
use crate::misc::Convert2Vec;
use crate::impl_convert;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct CharParams{
    vertices_so_far: u32,
    iterator_end: u32,
    draw_index: u32,
    max_points_per_char: u32,
    max_number_of_vertices: u32,
    padding: [u32 ; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Char {
    start_pos: [f32 ; 3],
    font_size: f32,
    value: [f32 ; 4],
    vec_dim_count: u32,
    color: u32,
    decimal_count: u32,
    auxiliary_data: u32,
}

impl_convert!{CharParams}
impl_convert!{Char}

struct CharProcessor {

    char_pipeline_wrapper: ComputePipelineWrapper,
    pre_processor_pipeline_wrapper: ComputePipelineWrapper,
    char_pipeline_bind_groups: wgpu::BindGroup,
    pre_processor_bind_groups: wgpu::BindGroup,
    direct_dispatch_buffer: wgpu::Buffer,
    char_param_buffer: wgpu::Buffer,
    chars_buffer: wgpu::Buffer,
}

impl CharProcessor {

    pub fn init(device: &wgpu::Device,
                indirect_draw_buffer: &wgpu::Buffer,
                dispatch_counter_buffer: &wgpu::Buffer,
                render_buffer: &wgpu::Buffer,
                camera_buffer: &wgpu::Buffer,
                max_number_of_chars: u32,
                max_points_per_char: u32,
                max_number_of_vertices: u32) -> Self {

        // Create dispatch indirect buffer.
        let direct_dispatch_buffer = 
                buffer_from_data::<DispatchIndirect>(
                    &device,
                    &vec![DispatchIndirect{ x: 0, y: 0, z: 0, } ; 1024],
                    wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDIRECT,
                    Some("Pre processor indirect dispatch buffer")
        );

        // Create char params buffer here.
        let char_param_buffer = buffer_from_data::<CharParams>(
            &device,
            &vec![
                CharParams{ vertices_so_far: 0,
                iterator_end: 0,
                draw_index: 0,
                max_points_per_char: 4000,
                max_number_of_vertices: max_number_of_vertices,
                padding: [1,2,3], },],
            wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            None
            );

        // Create char buffer.
        let chars_buffer = 
            device.create_buffer(&wgpu::BufferDescriptor{
                label: Some("char buffer"),
                size: (max_number_of_chars * std::mem::size_of::<Char>() as u32) as u64,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });



        // Create here pipeline and bind group mapper for numbers.
        let mut bind_group_mapper = BindGroupMapper::init(device);
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(0, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(1, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(2, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(3, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.build_bind_group_layouts(device);

        // Create wgsl module.
        let wgsl_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Numbers module"),
            source: wgpu::ShaderSource::Wgsl(
                Cow::Borrowed(include_str!("wgsl/numbers.wgsl"))),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Numbers layout"),
            bind_group_layouts: &bind_group_mapper.get_bind_group_layouts().iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        let char_pipeline_wrapper = ComputePipelineWrapper::init(
                device,
                &pipeline_layout,
                &wgsl_module,
                "main",
                bind_group_mapper,
                Some("Char pipeline"));

        // Create here pipeline and bind group mapper for char preprocessor.
        let mut pre_processor_mapper = BindGroupMapper::init(device);
        pre_processor_mapper.insert(device, 0, &create_uniform_bindgroup_layout(0, wgpu::ShaderStages::COMPUTE));
        pre_processor_mapper.insert(device, 0, &create_buffer_bindgroup_layout(1, wgpu::ShaderStages::COMPUTE, false));
        pre_processor_mapper.insert(device, 0, &create_buffer_bindgroup_layout(2, wgpu::ShaderStages::COMPUTE, false));
        pre_processor_mapper.insert(device, 0, &create_buffer_bindgroup_layout(3, wgpu::ShaderStages::COMPUTE, false));
        pre_processor_mapper.insert(device, 0, &create_buffer_bindgroup_layout(4, wgpu::ShaderStages::COMPUTE, false));
        pre_processor_mapper.build_bind_group_layouts(device);

        // Create wgsl module.
        let pre_processor_wgsl_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Char preprocessor module"),
            source: wgpu::ShaderSource::Wgsl(
                Cow::Borrowed(include_str!("wgsl/char_preprocessor.wgsl"))),
        });

        // Create pipeline layout
        let pre_processor_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Preprocessor layout"),
            bind_group_layouts: &pre_processor_mapper.get_bind_group_layouts().iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        let pre_processor_pipeline_wrapper = ComputePipelineWrapper::init(
                device,
                &pre_processor_pipeline_layout,
                &pre_processor_wgsl_module,
                "main",
                pre_processor_mapper,
                Some("Char pipeline"));

        // Create bindgroups.
        let char_bind_group = char_pipeline_wrapper.create_bind_group(
            device,
            &vec![
                &indirect_draw_buffer.as_entire_binding(),
                &dispatch_counter_buffer.as_entire_binding(),
                &chars_buffer.as_entire_binding(),
                &render_buffer.as_entire_binding(),
            ],
            0);

        // Create bindgroups.
        let pre_processor_bind_group = pre_processor_pipeline_wrapper.create_bind_group(
            device,
            &vec![
                &camera_buffer.as_entire_binding(),
                &char_param_buffer.as_entire_binding(), // TODO: keep char params information in this struct.
                &direct_dispatch_buffer.as_entire_binding(),
                &indirect_draw_buffer.as_entire_binding(),
                &chars_buffer.as_entire_binding(),
            ],
            0);

        Self {
            char_pipeline_wrapper: char_pipeline_wrapper,
            pre_processor_pipeline_wrapper: pre_processor_pipeline_wrapper,
            char_pipeline_bind_groups: char_bind_group,
            pre_processor_bind_groups: pre_processor_bind_group,
            direct_dispatch_buffer: direct_dispatch_buffer,
            char_param_buffer: char_param_buffer,
            chars_buffer: chars_buffer,
        }
    }
}
