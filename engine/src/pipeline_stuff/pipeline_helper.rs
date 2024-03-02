use std::num::NonZeroU32;
use wgpu::ShaderModule;
use std::borrow::Cow;

use crate::pipelines::{
    RenderPipelineWrapper,
    BindGroupMapper
};
use crate::vertex::create_vertex_attributes;
// use engine::bindgroups::{
//     create_uniform_bindgroup_layout,
//     create_texture,
//     create_texture_sampler,
// };

/// Creates a default depth stencil state.
pub fn create_default_depth_stencil_state() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState {
            front: wgpu::StencilFaceState::IGNORE,
            back: wgpu::StencilFaceState::IGNORE,
            read_mask: 0,
            write_mask: 0,
        },
        bias: wgpu::DepthBiasState {
            constant: 0,
            slope_scale: 0.0,
            clamp: 0.0,
        },
    }
}

pub fn create_render_pipeline_wrapper(
     device: &wgpu::Device,
     _sc_desc: &wgpu::SurfaceConfiguration, // NOT USED. Should we remove this?
     bind_group_mapper: BindGroupMapper,
     wgsl_module: &ShaderModule,
     _vertex_attributes: &Vec<wgpu::VertexFormat>,
     _vertex_step_mode: wgpu::VertexStepMode,
     _vertex_entry: &str,
     primitive_state: &wgpu::PrimitiveState,
     depth_state: &Option<wgpu::DepthStencilState>,
     fragment_state: &Option<wgpu::FragmentState>,
     multiview: Option<NonZeroU32>,
     label: Option<&str>) -> RenderPipelineWrapper { 

    // Create pipeline layout
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Default pipeline layout"),
        bind_group_layouts: &bind_group_mapper.get_bind_group_layouts().iter().collect::<Vec<_>>(),
        push_constant_ranges: &[],
    });

    // Calculate stride and create vertex attributes.
    let (stride, attributes) = create_vertex_attributes(&vec![wgpu::VertexFormat::Float32x4, wgpu::VertexFormat::Float32x4]);

    // As function parameter?
    let multisample = wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
    };

    RenderPipelineWrapper::init(
        device,
        &pipeline_layout,
        &wgpu::VertexState {
            module: &wgsl_module,
            entry_point: "vs_main",
            buffers: &[
                wgpu::VertexBufferLayout {
                    array_stride: stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attributes,
                }],
        },
        &primitive_state,
        &depth_state,
        multisample,
        &fragment_state,
        &multiview,
        label,
        bind_group_mapper,
    )
}
