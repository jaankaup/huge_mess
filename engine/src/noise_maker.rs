use crate::pipelines::ComputePipelineWrapper;
use std::mem::size_of;
use crate::pipelines::BindGroupMapper;
use bytemuck::Pod;
use bytemuck::Zeroable;
use crate::misc::Convert2Vec;
use crate::impl_convert;
use crate::buffer::buffer_from_data;
use crate::bindgroups::{
    create_uniform_bindgroup_layout,
    create_buffer_bindgroup_layout,
};
use std::borrow::Cow;


#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct NoiseParams {
    pub global_dim: [u32; 3],
    pub param_a: f32,
    pub local_dim: [u32; 3],
    pub param_b: f32,
    pub position: [f32; 3],
    pub param_c: f32,
}

impl_convert!{NoiseParams}

pub struct NoiseParamBuffer {
    pub noise_params: NoiseParams, // TODO: getter
    buffer: wgpu::Buffer,
}

impl NoiseParamBuffer {

    pub fn create(device: &wgpu::Device,
                  global_dim: [u32; 3],
                  param_a: f32,
                  local_dim: [u32; 3],
                  param_b: f32,
                  position: [f32; 3],
                  param_c: f32,
        ) -> Self {

        let params = NoiseParams {
            global_dim: global_dim,
            param_a,
            local_dim,
            param_b,
            position,
            param_c,
        };

        let buf = buffer_from_data::<NoiseParams>(
                  &device,
                  &vec![params],
                  wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
                  Some("noise params buffer.")
        );

        Self {
            noise_params: params,
            buffer: buf,
        }
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn get_noise_params(&self) -> &NoiseParams {
        &self.noise_params
    }

    pub fn update(&self, queue: &wgpu::Queue) {

        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.noise_params])
        );
    }
}

/// Noise maker.
pub struct NoiseMaker {
    compute_pipeline_wrapper: ComputePipelineWrapper,
    pub noise_params: NoiseParamBuffer, // TODO: getter
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl NoiseMaker {

    pub fn init(device: &wgpu::Device,
                entry_point: &String,
                global_dimension: [u32; 3],
                local_dimension: [u32; 3],
                position: [f32; 3],
                param_a:   f32,
                param_b:  f32,
                param_c: f32) -> Self {

        // Validation.
        let buf_size = (global_dimension[0] *
                        global_dimension[1] *
                        global_dimension[2] *
                        local_dimension[0] *
                        local_dimension[1] *
                        local_dimension[2] *
                        size_of::<f32>() as u32) as u64;

        // TODO: get buffer outside of struct.
        let buf = device.create_buffer(&wgpu::BufferDescriptor{
                      label: Some("noise buffer"), // Get label from function parameter
                      size: buf_size,
                      usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                      mapped_at_creation: false,
                      }
        );

        // Create here pipeline and bind group mapper.
        let mut bind_group_mapper = BindGroupMapper::init(device);
        bind_group_mapper.insert(device, 0, &create_uniform_bindgroup_layout(0, wgpu::ShaderStages::COMPUTE));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(1, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.build_bind_group_layouts(device);

        // Create wgsl module. TODO: from function parameter.
        let wgsl_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Default render shader"),
            source: wgpu::ShaderSource::Wgsl(
                Cow::Borrowed(include_str!("shaders/basic_noise.wgsl"))),

        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Noise pipeline layout"),
            bind_group_layouts: &bind_group_mapper.get_bind_group_layouts().iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        let pipeline_wrapper = ComputePipelineWrapper::init(
                device,
                &pipeline_layout,
                &wgsl_module,
                "main",
                bind_group_mapper,
                Some("Noise pipeline"));


        let params = NoiseParamBuffer::create(
                        device,
                        global_dimension,
                        param_a,
                        local_dimension,
                        param_b,
                        position,
                        param_c,
        );

        let bind_group = pipeline_wrapper.create_bind_group(
            device,
            &vec![
            &params.get_buffer().as_entire_binding(),
            &buf.as_entire_binding(),
            ],
            0);

        Self {
            compute_pipeline_wrapper: pipeline_wrapper,
            noise_params: params,
            buffer: buf,
            bind_group: bind_group,
        }
    }

    pub fn get_compute_pipeline_wrapper(&self) -> &ComputePipelineWrapper {
        &self.compute_pipeline_wrapper
    }

    // fn create_bingroups(&self, device: &wgpu::Device) {
    //     self.bind_groups = create_bind_groups(
    //             &device,
    //             &self.compute_object.bind_group_layout_entries,
    //             &self.compute_object.bind_group_layouts,
    //             &vec![
    //                 vec![
    //                 &self.noise_params.get_buffer().as_entire_binding(),
    //                 &self.buffer.as_entire_binding(),
    //                 ],
    //             ]
    //     );
    // }

    pub fn dispatch(&self, encoder: &mut wgpu::CommandEncoder) {

        let global_dimension = self.noise_params.noise_params.global_dim;
        let local_dimension  = self.noise_params.noise_params.local_dim;

        let total_grid_count =
                        global_dimension[0] *
                        global_dimension[1] *
                        global_dimension[2] *
                        local_dimension[0] *
                        local_dimension[1] *
                        local_dimension[2];

        self.compute_pipeline_wrapper.dispatch(
            &vec![(0, &self.bind_group)],
            encoder,
            total_grid_count / 1024, 1, 1, Some("noise dispatch")
        );
    }

    pub fn update_param_a(&mut self, queue: &wgpu::Queue, param_a: f32) {
        self.noise_params.noise_params.param_a = param_a;
        self.noise_params.update(queue);
    }

    pub fn update_param_b(&mut self, queue: &wgpu::Queue, param_b: f32) {
        self.noise_params.noise_params.param_b = param_b;
        self.noise_params.update(queue);
    }

    pub fn update_param_c(&mut self, queue: &wgpu::Queue, param_c: f32) {
        self.noise_params.noise_params.param_c = param_c;
        self.noise_params.update(queue);
    }

    pub fn update_position(&mut self, queue: &wgpu::Queue, pos: [f32; 3]) {
        self.noise_params.noise_params.position = pos;
        self.noise_params.update(queue);
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn get_position(&self) -> [f32; 3] {
        self.noise_params.noise_params.position
    }
}
