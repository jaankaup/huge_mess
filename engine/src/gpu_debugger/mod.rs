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

/// GpuDebugger can be used to render some basic primitives like numbers, arrows, aabbs and aabb
/// wires. 
pub struct GpuDebugger {
    /// A sub system for generating primives to renderable triangles.
    primitive_processor: PrimitiveProcessor,
    /// A sub system for generating numbers to renderable points.
    char_processor: CharProcessor,
    /// Maximum number of vertices that can be used for rendering in one part. The actual
    /// rendering is done in multiple parts.
    max_number_of_vertices: u32,
    /// Thread count is given for shader (work group size). This is not used now. 
    thread_count: u32,
    /// A pipeline wrapper for rendering points (numbers).
    v3c1_pipeline_wrapper: RenderPipelineWrapper,
    /// A pipeline wrapper for rendering primitives.
    v4n4_pipeline_wrapper: RenderPipelineWrapper,
    /// Bind groups for number pipeline wrapper.
    v3c1_bind_group: wgpu::BindGroup,
    /// Bind groups for primitive pipeline wrapper.
    v4n4_bind_group: wgpu::BindGroup,
    /// Some parameters for rendering.
    render_params: RenderParamBuffer,
    /// Light buffer for rendering shaders. 
    light: LightBuffer,
    /// This histogram stores the count of all renderable elements. 
    /// TODO: impove this to use tags.
    histogram_element_counter: Histogram,
}
        // ADD these to gpu_debugger.
        // Get the total number of elements.
        //let elem_counter = self.histogram_element_counter.get_values(device, queue);

        // let total_number_of_arrows = elem_counter[1];
        // let total_number_of_aabbs = elem_counter[2];
        // let total_number_of_aabb_wires = elem_counter[3];


impl GpuDebugger {

    /// Initializes GpuDebugger. Could this fail? TODO: -> Result<Self, Err>.
    /// TODO: add a custom pipelines for rendering.
    /// TODO: make shading adjustable.
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
            [25.0, 25.0, 25.0], // pos
            [25, 25, 130],  // spec
            [255,200,255], // light
            155.0,
            0.35,
            0.00013
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

    /// Get reference to the buffer that holds all aabbs.
    pub fn get_aabb_buffer(&self) -> &wgpu::Buffer {
        &self.primitive_processor.get_aabb_buffer()
    }

    /// Append aabb. TODO: rename this function to append_aabb. Also check that the appending
    /// succeeded.
    pub fn add_aabb(&self, device: &wgpu::Device, queue: &wgpu::Queue, aabb: &AABB) {

        let mut histogram_values = self.histogram_element_counter.get_values(device, queue);
        histogram_values[2] += 1;
        self.histogram_element_counter.set_values_cpu_version(device, queue, &histogram_values);
        self.primitive_processor.append_aabb(device, queue, aabb, histogram_values[2]-1);
    }

    /// Append multiple aabbs at once. TODO: add checking. Rename function. 
    pub fn add_aabbs(&self, device: &wgpu::Device, queue: &wgpu::Queue, aabb: &Vec<AABB>) {
        let mut histogram_values = self.histogram_element_counter.get_values(device, queue);
        let count_now = histogram_values[2];
        histogram_values[2] += aabb.len() as u32;
        self.histogram_element_counter.set_values_cpu_version(device, queue, &histogram_values);
        self.primitive_processor.append_aabbs(device, queue, aabb, count_now);
    }

    /// Append arrow. TODO: rename this function to append_arrow. Also check that the appending
    /// succeeded.
    pub fn add_arrow(&self, device: &wgpu::Device, queue: &wgpu::Queue, arrow: &Arrow) {

        let mut histogram_values = self.histogram_element_counter.get_values(device, queue);
        histogram_values[1] += 1;
        self.histogram_element_counter.set_values_cpu_version(device, queue, &histogram_values);
        self.primitive_processor.insert_arrow(device, queue, arrow, histogram_values[1]-1);
    }

    /// Append multiple arros at once. TODO: add checking. Rename function. 
    pub fn add_arrows(&self, device: &wgpu::Device, queue: &wgpu::Queue, arrows: &Vec<Arrow>) {

        assert!(arrows.len() > 0);
        let mut histogram_values = self.histogram_element_counter.get_values(device, queue);
        let count_now = histogram_values[1];
        histogram_values[1] += arrows.len() as u32;
        self.histogram_element_counter.set_values_cpu_version(device, queue, &histogram_values);
        self.primitive_processor.insert_arrows(device, queue, arrows, count_now);
    }

    /// Render all primives and numbers. Make some validation for draw buffer. TODO: Should this function
    /// return a failure? TODO: finish the number rendering.
    pub fn render(&mut self,
                  device: &wgpu::Device,
                  queue: &wgpu::Queue,
                  view: &wgpu::TextureView,
                  draw_buffer: &wgpu::Buffer,
                  depth_texture: &Tex,
                  clear: &mut bool) {


        // log::info!("GpugDebugger::Rendering");
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

        self.char_processor.render(
                  device,
                  queue,
                  draw_buffer,
                  &self.v3c1_bind_group, //render_bindgroup: &wgpu::BindGroup,
                  &self.v3c1_pipeline_wrapper.get_pipeline(),
                  view,
                  depth_texture,
                  total_number_of_chars,
                  self.max_number_of_vertices,
                  Some(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0, }),
                  *clear);

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
