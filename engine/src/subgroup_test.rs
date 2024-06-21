use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use crate::impl_convert;
use crate::pipelines::{BindGroupMapper, ComputePipelineWrapper};
use crate::bindgroups::{
    create_uniform_bindgroup_layout,
    create_buffer_bindgroup_layout,
};
use crate::buffer::buffer_from_data;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
struct ScanParams {
    data_start_index: u32,
    data_end_index: u32,
    data_size: u32,
    padding: u32,
}

pub struct MultiLevelScan {
    pipeline_wrapper: ComputePipelineWrapper,
    scan_params: ScanParams,
    scan_param_buffer: wgpu::Buffer,
    //bindgroups: wgpu::BindGroup,
}

impl MultiLevelScan {

    /// Initialize MultiLevelScan. 
    pub fn init(device: &wgpu::Device) -> Self {

        let module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shaders/warp_test.wgsl"),
            source: wgpu::ShaderSource::Wgsl(
                Cow::Borrowed(include_str!("shaders/multi_level_scan.wgsl"))),
        });

        let mut bind_group_mapper = BindGroupMapper::init(device);
        bind_group_mapper.insert(device, 0, &create_uniform_bindgroup_layout(0, wgpu::ShaderStages::COMPUTE));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(1, wgpu::ShaderStages::COMPUTE, true));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(2, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(3, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.build_bind_group_layouts(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Multi-level layout"),
            bind_group_layouts: &bind_group_mapper.get_bind_group_layouts().iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        let pipeline_wrapper = ComputePipelineWrapper::init(
            device,
            &pipeline_layout,
            &module,
            "main",
            bind_group_mapper,
            Some("Multi-level pipeline")
            );

        let scan_params: ScanParams = Default::default();

        let scan_param_buffer = 
            buffer_from_data::<ScanParams>(
            &device,
            &[scan_params],
            wgpu::BufferUsages::COPY_SRC |
            wgpu::BufferUsages::COPY_DST |
            wgpu::BufferUsages::STORAGE,
            None
        );


        Self {
            pipeline_wrapper: pipeline_wrapper,
            scan_params: scan_params,
            scan_param_buffer: scan_param_buffer, 
        }
    }

    pub fn update_params(&mut self, queue: &wgpu::Queue, scan_params: ScanParams) {

        self.scan_params = scan_params;

        queue.write_buffer(
            &self.scan_param_buffer,
            0,
            bytemuck::cast_slice(&[self.scan_params ])
        );
    }

    pub fn get_mc_params(&self) -> &ScanParams {
        &self.scan_params
    }

}
