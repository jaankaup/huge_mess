use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use crate::impl_convert;
use crate::pipelines::{BindGroupMapper, ComputePipelineWrapper};
use crate::bindgroups::{
    create_uniform_bindgroup_layout,
    create_buffer_bindgroup_layout,
};
use crate::buffer::buffer_from_data;

// let fmm_prefix_params = buffer_from_data::<FmmPrefixParams>(
//     &device,
//     &vec![FmmPrefixParams {
//         data_start_index: 0,
//         data_end_index: (number_of_fmm_cells - 1) as u32,
//         exclusive_parts_start_index: number_of_fmm_cells as u32,
//         exclusive_parts_end_index: number_of_fmm_cells as u32 + 2048,
//     }],
//     wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//     None
//     );

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct FmmPrefixParams {
    /// The start index of prefix sum
    pub data_start_index: u32,
    /// The end index of prefix sum
    pub data_end_index: u32,
    /// The start index of exclusive prefix sum (the sum of prefix sums)
    pub exclusive_parts_start_index: u32,
    /// The end index of exclusive prefix sum (the sum of prefix sums)
    pub exclusive_parts_end_index: u32,
}

// Plan:
// Prerequisities:
//     temp:array: size (as big (u32) as the input data length) + length of auxiliar sum.
//

// Prefix sum params.
//++ let fmm_prefix_params = buffer_from_data::<FmmPrefixParams>(
//++     &device,
//++     &vec![FmmPrefixParams {
//++         data_start_index: 0,
//++         data_end_index: (number_of_fmm_cells - 1) as u32,
//++             exclusive_parts_start_index: number_of_fmm_cells as u32, exclusive_parts_end_index: number_of_fmm_cells as u32 + 2048,
//++     }],
//++     wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//++     None
//++                                                                                                                                                                                               );
pub fn get_scan_block_size(thread_count: u32) -> u32 {
    thread_count 

}
/// A struct for prefix sum operations and resources.
/// 
/// Usage: 
/// 
/// 
pub struct PrefixSum {
    /// The pipeline wrapper for prefix sum.
    prefix_sum_wrapper: ComputePipelineWrapper,
    /// The bindgroups that prefix sum is using.
    prefix_sum_bindgroup: wgpu::BindGroup,
    /// A buffer for prefix sum params.
    prefix_param_buffer: wgpu::Buffer,
    /// The number of objects after prefix_sum operation
    number_of_filtered_objects: u32,
}

impl PrefixSum {

    /// prefix_sum buffer is a temporary buffer for intermediate results of prefix_sum.
    /// input_data is the data that should be processed.
    /// output_data is filtered data.
    pub fn init(device: &wgpu::Device,
                prefix_sum_buffer: &wgpu::Buffer,
                input_data: &wgpu::Buffer,
                output_data: &wgpu::Buffer,
                scan_block_size: u32) -> Self {

        // TODO: even bigger scan_block_sizes
        assert!(scan_block_size == 64 || 
                scan_block_size == 128 || 
                scan_block_size == 192 || 
                scan_block_size == 256 || 
                scan_block_size == 320 || 
                scan_block_size == 384 || 
                scan_block_size == 448 || 
                scan_block_size == 512 || 
                scan_block_size == 576 || 
                scan_block_size == 640 || 
                scan_block_size == 704 || 
                scan_block_size == 768 || 
                scan_block_size == 768 || 
                scan_block_size == 832 || 
                scan_block_size == 896 || 
                scan_block_size == 960 || 
                scan_block_size == 1024); 

// 64 :: 136
// 128 :: 272
// 192 :: 408
// 256 :: 544
// 320 :: 680
// 384 :: 816
// 448 :: 952
// 512 :: 1088
// 576 :: 1224
// 640 :: 1360
// 704 :: 1496
// 768 :: 1632
// 832 :: 1768
// 896 :: 1904
// 960 :: 2040
// 1024 :: 2176


        // Create prefix sum module.
        // In the future, create multiple versions for prefix_sum (thread_count, dispatch count), 
        // or create ability to create a creatain module using a specific parameters for
        // thread_count ...). Also, try to create some sort of typeparameter 
        let module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shaders/prefix_sum.wgsl"),
            source: wgpu::ShaderSource::Wgsl(
                Cow::Borrowed(include_str!("shaders/prefix_sum.wgsl"))),
        });

        // @group(0)
        // @binding(0)
        // var<uniform> fmm_prefix_params: PrefixParams;
        // 
        // @group(0)
        // @binding(1)
        // var<storage, read_write> fmm_blocks: array<FmmBlock>;
        // 
        // @group(0)
        // @binding(2)
        // var<storage, read_write> temp_prefix_sum: array<u32>;
        // 
        // @group(0)
        // @binding(3)
        // var<storage,read_write> filtered_blocks: array<FmmBlock>;

        // Create the bindgroup mapper for prefix_sum.
        let mut bind_group_mapper = BindGroupMapper::init(device);
        bind_group_mapper.insert(device, 0, &create_uniform_bindgroup_layout(0, wgpu::ShaderStages::COMPUTE));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(1, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(2, wgpu::ShaderStages::COMPUTE, false));
        bind_group_mapper.insert(device, 0, &create_buffer_bindgroup_layout(3, wgpu::ShaderStages::COMPUTE, false));
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

        let params = FmmPrefixParams {
            data_start_index: 0,
            data_end_index: 0,
            exclusive_parts_start_index: 0,
            exclusive_parts_end_index: 0,
        };

        let param_buffer = buffer_from_data::<FmmPrefixParams>(
            &device,
            &[params],
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            None
        );


        // Create bindgroups.
        let prefix_group = pipeline_wrapper.create_bind_group(
            device,
            &vec![
                &param_buffer.as_entire_binding(),
                &input_data.as_entire_binding(),
                &prefix_sum_buffer.as_entire_binding(),
                &output_data.as_entire_binding(),
            ],
            0);

        Self {
            prefix_sum_wrapper: pipeline_wrapper,
            prefix_sum_bindgroup: prefix_group,
            prefix_param_buffer: param_buffer,
            number_of_filtered_objects: 0,
        }
    }

    pub fn filter(device: &wgpu::Device) {
        
    }
}
