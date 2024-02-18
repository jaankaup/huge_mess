use crate::misc::udiv_up_safe32;
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
use crate::draw_commands::draw;
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
    arrow_aabb_params: ArrowAabbParams,
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
            arrow_aabb_params: arrow_aabb_params,
        }
    }

        pub fn render(&mut self,
                  device: &wgpu::Device,
                  queue: &wgpu::Queue,
                  view: &wgpu::TextureView,
                  depth_texture: &Tex,
                  draw_buffer: &wgpu::Buffer,
                  draw_bind_group: &wgpu::BindGroup,
                  draw_pipeline: &wgpu::RenderPipeline,
                  total_number_of_arrows: u32,
                  total_number_of_aabbs: u32,
                  total_number_of_aabb_wires: u32,
                  max_number_of_vertices: u32,
                  thread_count: u32,
                  clear_color: Option<wgpu::Color>,
                  clear: &mut bool
                  ) {

        // ADD these to gpu_debugger.
        // Get the total number of elements.
        //let elem_counter = self.histogram_element_counter.get_values(device, queue);

        // let total_number_of_arrows = elem_counter[1];
        // let total_number_of_aabbs = elem_counter[2];
        // let total_number_of_aabb_wires = elem_counter[3];

        const vertices_per_element_arrow: u32 = 72;
        const vertices_per_element_aabb: u32 = 36;
        const vertices_per_element_aabb_wire: u32 = 432;

        // The number of vertices created with one dispatch.
        let vertices_per_dispatch_arrow = thread_count * vertices_per_element_arrow;
        let vertices_per_dispatch_aabb = thread_count * vertices_per_element_aabb;
        let vertices_per_dispatch_aabb_wire = thread_count * vertices_per_element_aabb_wire;

        // [(element_type, total number of elements, number of vercies per dispatch, vertices_per_element)]
        let draw_params = [(0, total_number_of_arrows,     vertices_per_dispatch_arrow, vertices_per_element_arrow),
                           (1, total_number_of_aabbs,      vertices_per_dispatch_aabb, vertices_per_element_aabb), // !!!
                           (2, total_number_of_aabb_wires, vertices_per_dispatch_aabb_wire, vertices_per_element_aabb_wire)];

        // For each element type, create triangle meshes and render with respect of draw buffer size.
        for (e_type, e_size, v_per_dispatch, vertices_per_elem) in draw_params.iter() {

            // The number of safe dispathes. This ensures the draw buffer doesn't over flow.
            let safe_number_of_dispatches = max_number_of_vertices as u32 / v_per_dispatch;

            // The number of items to create and draw.
            let mut items_to_process = *e_size;

            // Nothing to process.
            if *e_size == 0 { continue; }

            // Create the initial params.
            self.arrow_aabb_params.iterator_start_index = 0;
            self.arrow_aabb_params.iterator_end_index = std::cmp::min(*e_size, safe_number_of_dispatches * v_per_dispatch);
            self.arrow_aabb_params.element_type = *e_type;

            queue.write_buffer(
                &self.arrow_params_buffer,
                0,
                bytemuck::cast_slice(&[self.arrow_aabb_params])
            );

            // Continue process until all element are rendered.
            while items_to_process > 0 {

                // The number of remaining dispatches to complete the triangle mesh creation and
                // rendering.
                let total_number_of_dispatches = udiv_up_safe32(items_to_process, thread_count);

                // Calculate the number of dispatches for this run.
                let local_dispatch = std::cmp::min(total_number_of_dispatches, safe_number_of_dispatches);

                // Then number of elements that are going to be rendered.
                let number_of_elements = std::cmp::min(local_dispatch * thread_count, items_to_process);

                let mut encoder_arrow_aabb = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("arrow_aabb ... ") });

                self.arrow_aabb_params.iterator_end_index = self.arrow_aabb_params.iterator_start_index + std::cmp::min(number_of_elements, safe_number_of_dispatches * v_per_dispatch);

                queue.write_buffer(
                    &self.arrow_params_buffer,
                    0,
                    bytemuck::cast_slice(&[self.arrow_aabb_params])
                );

                self.aabb_pipeline_wrapper.dispatch(
                    &vec![(0, &self.aabb_bind_group)],
                    &mut encoder_arrow_aabb,
                    local_dispatch, 1, 1, Some("arrow local dispatch")
                );

                // println!("local_dispatch == {}", local_dispatch);

                queue.submit(Some(encoder_arrow_aabb.finish()));


                let draw_count = number_of_elements * vertices_per_elem;

                let mut encoder_arrow_rendering = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("arrow rendering ... ") });

                draw(&mut encoder_arrow_rendering,
                     &view,
                     Some(depth_texture),
                     &vec![draw_bind_group],
                     draw_pipeline,
                     draw_buffer,
                     0..draw_count,
                     if *clear && clear_color.is_none() { &Some(wgpu::Color { r: 0.1, g: 0.0, b: 0.0, a: 1.0, }) } else { &clear_color }, // Wrong place for this. Add to draw.
                     *clear
                );

                if *clear { *clear = false; }

                // Decrease the total count of elements.
                items_to_process = items_to_process - number_of_elements;

                queue.submit(Some(encoder_arrow_rendering.finish()));

                self.arrow_aabb_params.iterator_start_index = self.arrow_aabb_params.iterator_end_index; // + items_to_process;
            }
        }
    }
}
