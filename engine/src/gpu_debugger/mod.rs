use crate::histogram::Histogram;
use crate::texture::Texture as Tex;
use crate::lights::LightBuffer;
use crate::gpu_debugger::char_generator::CharProcessor;
use crate::gpu_debugger::char_generator::Char;
use crate::gpu_debugger::primitive_processor::PrimitiveProcessor;
use crate::gpu_debugger::primitive_processor::Arrow;
use crate::gpu_debugger::primitive_processor::AABB;
use crate::buffer::buffer_from_data;
use std::mem::size_of;
use std::collections::HashMap;
use crate::misc::Convert2Vec;
use crate::impl_convert;
use crate::common_structs::{
    DrawIndirect,
    DispatchIndirect,
};
use crate::pipelines::RenderPipelineWrapper;
use crate::pipeline_stuff::custom_pipelines::{
    RenderParamBuffer,
    default_render_shader_v3c1,
    render_v4n4_camera_light_other_params,
};

use bytemuck::{Pod, Zeroable};

pub mod char_generator;
pub mod primitive_processor;

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
    v3c1_pipeline_wrapper: RenderPipelineWrapper,
    v4n4_pipeline_wrapper: RenderPipelineWrapper,
    v3c1_bind_group: wgpu::BindGroup,
    v4n4_bind_group: wgpu::BindGroup,
    render_params: RenderParamBuffer,
    light: LightBuffer,
    histogram_element_counter: Histogram,
}
        // ADD these to gpu_debugger.
        // Get the total number of elements.
        //let elem_counter = self.histogram_element_counter.get_values(device, queue);

        // let total_number_of_arrows = elem_counter[1];
        // let total_number_of_aabbs = elem_counter[2];
        // let total_number_of_aabb_wires = elem_counter[3];


impl GpuDebugger {

    pub fn init(device: &wgpu::Device,
                sc_desc: &wgpu::SurfaceConfiguration,
                render_buffer: &wgpu::Buffer,
                camera_buffer: &wgpu::Buffer,
                max_number_of_arrows: u32,
                max_number_of_aabbs: u32,
                max_number_of_aabb_wires: u32,
                max_number_of_chars: u32,
                max_number_of_vertices: u32,
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

        let v3c1 = default_render_shader_v3c1(device, sc_desc);
        let v4n4 = render_v4n4_camera_light_other_params(device, sc_desc);
        let render_param_buffer = RenderParamBuffer::create(device, 1.0);
        let light = LightBuffer::create(
            device,
            [25.0, 55.0, 25.0], // pos
            [25, 25, 130],  // spec
            [255,200,255], // light
            155.0,
            0.35,
            0.0013
            );

        let v3c1_bind_group = v3c1.create_bind_group(device, &vec![&camera_buffer.as_entire_binding()], 0);
        let v4n4_bind_group = v4n4.create_bind_group(
                device,
                &vec![&camera_buffer.as_entire_binding(), &light.get_buffer().as_entire_binding(), &render_param_buffer.get_buffer().as_entire_binding()],
                0);

        Self {
            primitive_processor: primitive_processor,
            char_processor: char_processor,
            max_number_of_vertices: max_number_of_vertices,
            thread_count: thread_count,
            v3c1_pipeline_wrapper: v3c1,
            v4n4_pipeline_wrapper: v4n4,
            v3c1_bind_group: v3c1_bind_group,
            v4n4_bind_group: v4n4_bind_group,
            render_params: render_param_buffer,
            light: light,
            histogram_element_counter: Histogram::init(device, &vec![0; 4]),
        }
    }

    pub fn get_aabb_buffer(&self) -> &wgpu::Buffer {
        &self.primitive_processor.get_aabb_buffer()
    }

    pub fn add_aabb(&self, device: &wgpu::Device, queue: &wgpu::Queue, aabb: &AABB) {

        let mut histogram_values = self.histogram_element_counter.get_values(device, queue);
        histogram_values[2] += 1;

        self.primitive_processor.append_aabb(device, queue, aabb, histogram_values[2]);

        self.histogram_element_counter.set_values_cpu_version(queue, &histogram_values);
    }

    pub fn render(&mut self,
                  device: &wgpu::Device,
                  queue: &wgpu::Queue,
                  view: &wgpu::TextureView,
                  draw_buffer: &wgpu::Buffer,
                  // draw_bind_group: &wgpu::BindGroup,
                  // draw_pipeline: &wgpu::RenderPipeline,
                  depth_texture: &Tex,
                  clear: &mut bool) {


        log::info!("GpugDebugger::Rendering");
        // Check the total number of elements.
        let elem_counter = self.histogram_element_counter.get_values(device, queue);

        let total_number_of_chars = elem_counter[0];
        let total_number_of_arrows = elem_counter[1];
        let total_number_of_aabbs = elem_counter[2];
        let total_number_of_aabb_wires = elem_counter[3];

        self.primitive_processor.render(
            device,
            queue,
            view,
            depth_texture,
            draw_buffer,
            &self.v4n4_bind_group, //: &wgpu::BindGroup,
            self.v4n4_pipeline_wrapper.get_pipeline(), //: &wgpu::RenderPipeline,
            total_number_of_arrows,
            total_number_of_aabbs,
            total_number_of_aabb_wires,
            self.max_number_of_vertices,
            64,
            Some(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0, }),
            clear);

        // Reset element counters.
        // let elem_counter = self.histogram_element_counter.reset_all_cpu_version(queue, 0);
    }

    // pub fn get_output_chars_buffer(&self) -> &wgpu::Buffer {
    //     self.buffers.get(&"output_chars".to_string()).unwrap()
    // }
    // pub fn get_output_arrows_buffer(&self) -> &wgpu::Buffer {
    //     self.buffers.get(&"output_arrows".to_string()).unwrap()
    // }
    // pub fn get_output_aabbs_buffer(&self) -> &wgpu::Buffer {
    //     self.buffers.get(&"output_aabbs".to_string()).unwrap()
    // }
    // pub fn get_output_aabb_wires_buffer(&self) -> &wgpu::Buffer {
    //     self.buffers.get(&"output_aabb_wires".to_string()).unwrap()
    // }
}
