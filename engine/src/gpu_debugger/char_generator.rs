use crate::histogram::Histogram;
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

use wgpu::TextureView;

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

pub struct CharProcessor {

    char_pipeline_wrapper: ComputePipelineWrapper,
    pre_processor_pipeline_wrapper: ComputePipelineWrapper,
    char_pipeline_bind_groups: wgpu::BindGroup,
    pre_processor_bind_groups: wgpu::BindGroup,
    indirect_dispatch_buffer: wgpu::Buffer,
    char_param_buffer: wgpu::Buffer,
    chars_buffer: wgpu::Buffer,
    indirect_draw_buffer: wgpu::Buffer,
    dispatch_counter_histogram: Histogram, 
}

impl CharProcessor {

    pub fn init(device: &wgpu::Device,
                render_buffer: &wgpu::Buffer,
                camera_buffer: &wgpu::Buffer,
                max_number_of_chars: u32,
                max_points_per_char: u32,
                max_number_of_vertices: u32) -> Self {

        // Create histogram for wgsl shader.
        let histogram = Histogram::init(device, &vec![0]);

        // Create render indirect buffer.
        let indirect_draw_buffer = buffer_from_data::<DrawIndirect>(
                &device,
                &vec![DrawIndirect{ vertex_count: 0, instance_count: 1, base_vertex: 0, base_instance: 0, } ; 1024],
                wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDIRECT,
                Some("Char processor: Indirect draw buffer")
                );

        // Create dispatch indirect buffer.
        let indirect_dispatch_buffer = 
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
                &histogram.get_histogram_buffer().as_entire_binding(),
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
                &indirect_dispatch_buffer.as_entire_binding(), // ADD histogram
                &indirect_draw_buffer.as_entire_binding(),
                &chars_buffer.as_entire_binding(),
            ],
            0);

        Self {
            char_pipeline_wrapper: char_pipeline_wrapper,
            pre_processor_pipeline_wrapper: pre_processor_pipeline_wrapper,
            char_pipeline_bind_groups: char_bind_group,
            pre_processor_bind_groups: pre_processor_bind_group,
            indirect_dispatch_buffer: indirect_dispatch_buffer,
            char_param_buffer: char_param_buffer,
            chars_buffer: chars_buffer,
            indirect_draw_buffer: indirect_draw_buffer,
            dispatch_counter_histogram: histogram,
        }
    }

    pub fn render(&self,
                  device: &wgpu::Device,
                  queue: &wgpu::Queue,
                  render_buffer: &wgpu::Buffer,
                  render_bindgroup: &wgpu::BindGroup,
                  render_pipeline: &wgpu::RenderPipeline,
                  view: &wgpu::TextureView,
                  depth_texture: &Tex,
                  number_of_chars: u32,
                  max_number_of_vertices: u32,
                  clear_color: Option<wgpu::Color>, 
                  clear: bool) {

        let charparams_result = to_vec::<CharParams>(
            &device,
            &queue,
            &self.char_param_buffer,
            0,
            (size_of::<CharParams>()) as wgpu::BufferAddress
            ).unwrap();

        // Can we avoid this. Should we use some other parameter than vertices_so_far (we have
        // padding for future usage)?
        if charparams_result[0].vertices_so_far > 0 {

            // Create char params for pre processor. TODO: replace vertices_so_far value with some
            // padding values.
            let cp = CharParams{
                vertices_so_far: 0,
                iterator_end: number_of_chars,
                draw_index: 0,
                max_points_per_char: 4000,
                max_number_of_vertices: max_number_of_vertices - 500000, // TODO: avoid this???
                padding: [1,2,3],
            };

            queue.write_buffer(
                &self.char_param_buffer,
                0,
                bytemuck::cast_slice(&[cp])
                );

            self.dispatch_counter_histogram.reset_all_cpu_version(queue, 0);

            // Dispatch char pre processor.
            let mut encoder_char_preprocessor = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("char preprocessor encoder") });

            self.pre_processor_pipeline_wrapper.dispatch(
                &vec![(0, &self.pre_processor_bind_groups)],
                &mut encoder_char_preprocessor,
                1, 1, 1, Some("char preprocessor dispatch")
                );

            queue.submit(Some(encoder_char_preprocessor.finish()));

            // Create point data from char elements and draw.
            let mut encoder_char = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("numbers encoder") });

            for i in 0..(charparams_result[0].draw_index + 1) {

                self.char_pipeline_wrapper.dispatch_indirect(
                    &vec![(0, &self.char_pipeline_bind_groups)],
                    &mut encoder_char,
                    &self.indirect_dispatch_buffer,
                    (i * std::mem::size_of::<DispatchIndirect>() as u32) as wgpu::BufferAddress,
                    Some("numbers dispatch")
                    );

                draw_indirect(
                    &mut encoder_char,
                    &view,
                    Some(depth_texture), // we need to get this
                    &vec![render_bindgroup], // we need to get this
                    render_pipeline, // we need to get this
                    render_buffer, // we need to get this
                    &self.indirect_draw_buffer, // Should we have a own draw_buffer?
                    (i * std::mem::size_of::<DrawIndirect>() as u32) as wgpu::BufferAddress,
                    if clear && clear_color.is_none() { &Some(wgpu::Color { r: 0.1, g: 0.0, b: 0.0, a: 1.0, }) } else { &clear_color },
                    clear
                    );
            }
            queue.submit(Some(encoder_char.finish()));
        }
    }
}
