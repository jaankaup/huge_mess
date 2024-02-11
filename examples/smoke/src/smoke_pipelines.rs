use core::fmt::Debug;
use std::hash::Hash;
use std::num::NonZeroU32;
use wgpu::ShaderModule;
use std::borrow::Cow;
use engine::default_things::VertexStateWrapper;
use engine::pipelines::{
    RenderPipelineWrapper,
    BindGroupMapper
};
use engine::vertex::create_vertex_attributes;
use engine::bindgroups::{
    create_uniform_bindgroup_layout,
    create_texture,
    create_texture_sampler,
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum DefaultBindGroups {
    CameraUniform,
    LightUniform,
    Texture1,
    Texture1Sampler,
    Texture2,
    Texture2Sampler,
}

/// Define basic render pipeline. TODO: Refactor pipeline creation.
pub fn default_render_shader_v4n4_camera_light_tex2(device: &wgpu::Device, sc_desc: &wgpu::SurfaceConfiguration) -> RenderPipelineWrapper {

    let vertex_attributes = vec![wgpu::VertexFormat::Float32x4, wgpu::VertexFormat::Float32x4];
      
    let mut bind_group_mapper = BindGroupMapper::init(device);
    bind_group_mapper.insert(device, 0, &create_uniform_bindgroup_layout(0, wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT));
    bind_group_mapper.insert(device, 0, &create_uniform_bindgroup_layout(1, wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT));
    bind_group_mapper.insert(device, 1, &create_texture(0, wgpu::ShaderStages::FRAGMENT));
    bind_group_mapper.insert(device, 1, &create_texture_sampler(1, wgpu::ShaderStages::FRAGMENT));
    bind_group_mapper.insert(device, 1, &create_texture(2, wgpu::ShaderStages::FRAGMENT));
    bind_group_mapper.insert(device, 1, &create_texture_sampler(3, wgpu::ShaderStages::FRAGMENT));
    bind_group_mapper.build_bind_group_layouts(device);

    // Create wgsl module.
    let wgsl_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Default render shader"),
        source: wgpu::ShaderSource::Wgsl(
            Cow::Borrowed(include_str!("assets/wgsl/v4n4_camera_light_tex2.wgsl"))),

    });

    // Define primitive state
    let primitive_state = wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw, //  wgpu::FrontFace::Cw
        cull_mode: None, // Some(wgpu::Face::Back),
                         // Some(wgpu::Face::Front),
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false,
    };

    let depth_state = Some(wgpu::DepthStencilState {
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
    });

    // Fragment state
    let binding = [Some(wgpu::ColorTargetState {
               format: sc_desc.format,
               blend: None,
               write_mask: wgpu::ColorWrites::COLOR,
           })];

    let fragment_state = Some(wgpu::FragmentState {
           module: wgsl_module,
           entry_point: "fs_main",
           targets: &binding,
       });

    create_render_pipeline_wrapper(
        device,
        sc_desc,
        bind_group_mapper,
        &wgsl_module,
        &vertex_attributes,
        wgpu::VertexStepMode::Vertex,
        &"vs_main",
        &primitive_state,
        &depth_state,
        &fragment_state, 
        None)
}

pub fn create_render_pipeline_wrapper(
     device: &wgpu::Device,
     sc_desc: &wgpu::SurfaceConfiguration,
     bind_group_mapper: BindGroupMapper,
     wgsl_module: &ShaderModule,
     vertex_attributes: &Vec<wgpu::VertexFormat>,
     vertex_step_mode: wgpu::VertexStepMode,
     vertex_entry: &str,
     primitive_state: &wgpu::PrimitiveState,
     depth_state: &Option<wgpu::DepthStencilState>,
     fragment_state: &Option<wgpu::FragmentState>,
     multiview: Option<NonZeroU32>) -> RenderPipelineWrapper { 

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
        Some("Jeejee"),
        bind_group_mapper,
    )
}