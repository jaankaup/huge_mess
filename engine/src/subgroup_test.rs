use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use crate::impl_convert;
use crate::pipelines::{BindGroupMapper, ComputePipelineWrapper};
use crate::bindgroups::{
    create_uniform_bindgroup_layout,
    create_buffer_bindgroup_layout,
};
use crate::buffer::buffer_from_data;

pub struct WarpTest {
    pipeline_wrapper: ComputePipelineWrapper,
    //bindgroups: wgpu::BindGroup,
}

impl WarpTest {

    pub fn init(device: &wgpu::Device) -> Self {

        let module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shaders/warp_test.wgsl"),
            source: wgpu::ShaderSource::Wgsl(
                Cow::Borrowed(include_str!("shaders/warp_test.wgsl"))),
        });

        let mut bind_group_mapper = BindGroupMapper::init(device);
        bind_group_mapper.insert(device, 0, &create_uniform_bindgroup_layout(0, wgpu::ShaderStages::COMPUTE));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(1, wgpu::ShaderStages::COMPUTE, false));
        // bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(2, wgpu::ShaderStages::COMPUTE, false));
        // bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(3, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.build_bind_group_layouts(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Prefix sum layout"),
            bind_group_layouts: &bind_group_mapper.get_bind_group_layouts().iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        let pipeline_wrapper = ComputePipelineWrapper::init(
            device,
            &pipeline_layout,
            &module,
            "main",
            bind_group_mapper,
            Some("Prefix sum pipeline")
            );

        Self {
            pipeline_wrapper: pipeline_wrapper,
        }
    }

}
