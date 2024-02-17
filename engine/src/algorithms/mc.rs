use std::borrow::Cow;
use crate::pipelines::BindGroupMapper;
use crate::pipelines::ComputePipelineWrapper;
use bytemuck::{Zeroable, Pod};
use crate::buffer::buffer_from_data;
use crate::common_structs::{DrawIndirect};
use crate::histogram::Histogram;
use crate::bindgroups::{
    create_uniform_bindgroup_layout,
    create_buffer_bindgroup_layout,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct McParams {
    pub base_position: [f32; 4],
    pub isovalue: f32,
    pub cube_length: f32,
    pub future_usage1: f32, // Remove
    pub future_usage2: f32, // Remove
    pub noise_global_dimension: [u32; 4], 
    pub noise_local_dimension: [u32; 4], 
}

pub struct MarchingCubes {
    // The mc pipeline.
    compute_pipeline_wrapper: ComputePipelineWrapper,
    mc_params: McParams,
    mc_params_buffer: wgpu::Buffer,
    buffer_counter: Histogram,
    indirect_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl MarchingCubes {

    pub fn get_counter_value(&self, device:&wgpu::Device, queue: &wgpu::Queue) -> u32 {
        self.buffer_counter.get_values(device, queue)[0]
    }
    pub fn get_draw_indirect_buffer(&self) -> &wgpu::Buffer {
        &self.indirect_buffer
    }
    pub fn reset_counter_value(&self, queue: &wgpu::Queue) {
        self.buffer_counter.set_values_cpu_version(queue, &vec![0]);
        
        queue.write_buffer(
            &self.indirect_buffer,
            0,
            bytemuck::cast_slice(&[
                DrawIndirect {
                    vertex_count: 0,
                    instance_count: 1,
                    base_vertex: 0,
                    base_instance: 0,
                }
            ])
        );
    }

    pub fn update_mc_params(&mut self, queue: &wgpu::Queue, mc_params: McParams) {

        // self.mc_params.isovalue = isovalue;
        self.mc_params = mc_params;

        queue.write_buffer(
            &self.mc_params_buffer,
            0,
            bytemuck::cast_slice(&[self.mc_params ])
        );
    }

    pub fn get_mc_params(&self) -> McParams {
        self.mc_params
    }

    pub fn init_with_noise_buffer(device: &wgpu::Device,
                                  mc_params: &McParams,
                                  noise_buffer: &wgpu::Buffer,
                                  output_buffer: &wgpu::Buffer,
                                  ) -> Self {

        let histogram = Histogram::init(device, &vec![0]);

        // Initialize the draw indirect data.
        let indirect_data =  
            DrawIndirect {
                vertex_count: 0,
                instance_count: 1,
                base_vertex: 0,
                base_instance: 0,
            };

        let indirect_buffer = 
            buffer_from_data::<DrawIndirect>(
            &device,
            &[indirect_data],
            // wgpu::BufferUsages::VERTEX |
            wgpu::BufferUsages::COPY_SRC |
            wgpu::BufferUsages::COPY_DST |
            wgpu::BufferUsages::INDIRECT |
            wgpu::BufferUsages::STORAGE,
            None
        );

        // Create here pipeline and bind group mapper.
        let mut bind_group_mapper = BindGroupMapper::init(device);
        bind_group_mapper.insert(device, 0, &create_uniform_bindgroup_layout(0, wgpu::ShaderStages::COMPUTE));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(1, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(2, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(3, wgpu::ShaderStages::COMPUTE, true));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(4, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.build_bind_group_layouts(device);

        // Create wgsl module. TODO: from function parameter.
        let wgsl_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Default render shader"),
            source: wgpu::ShaderSource::Wgsl(
                Cow::Borrowed(include_str!("wgsl/marching_cubes_indirect.wgsl"))),

        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mc pipeline layout"),
            bind_group_layouts: &bind_group_mapper.get_bind_group_layouts().iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        let pipeline_wrapper = ComputePipelineWrapper::init(
                device,
                &pipeline_layout,
                &wgsl_module,
                "main",
                bind_group_mapper,
                Some("Mc pipeline"));

        let mc_params_buffer = buffer_from_data::<McParams>(
                &device,
                &[*mc_params],
                wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                None
        );

        let bind_group = pipeline_wrapper.create_bind_group(
            device,
            &vec![
                &mc_params_buffer.as_entire_binding(),
                &indirect_buffer.as_entire_binding(),
                &histogram.get_histogram_buffer().as_entire_binding(),
                &noise_buffer.as_entire_binding(),
                &output_buffer.as_entire_binding()
            ],
            0);

        Self {
            compute_pipeline_wrapper: pipeline_wrapper,
            mc_params: *mc_params,
            mc_params_buffer: mc_params_buffer,
            buffer_counter: histogram,
            indirect_buffer: indirect_buffer,
            bind_group: bind_group,
        }
    }

    pub fn dispatch(&self,
                    encoder: &mut wgpu::CommandEncoder,
                    x: u32, y: u32, z: u32) {

        self.compute_pipeline_wrapper.dispatch(
            &vec![(0, &self.bind_group)],
            encoder,
            x, y, z, Some("mc dispatch"));
    }
}
