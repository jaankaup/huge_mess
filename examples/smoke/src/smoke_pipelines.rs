use std::borrow::Cow;
use engine::default_things::VertexStateWrapper;
use engine::pipelines::{
    LayoutMapper,
    RenderPipelineWrapper,
    EntryLocation,
};
use engine::vertex::create_vertex_attributes;
use engine::bindgroups::{
    create_uniform_bindgroup_layout,
    create_texture,
    create_texture_sampler,
};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum DefaultBindGroups {
    CameraUniform,
    LightUniform,
    Texture1,
    Texture1Sampler,
    Texture2,
    Texture2Sampler,
}

pub fn create_default_render_pipeline(device: &wgpu::Device, sc_desc: &wgpu::SurfaceConfiguration) -> RenderPipelineWrapper {

    // Create layout
    let mut layout = LayoutMapper::init();

    let _ = layout.add(&(0,0),
               &create_uniform_bindgroup_layout(0, wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT),
               &DefaultBindGroups::CameraUniform);

    let _ = layout.add(&(1,0),
               &create_uniform_bindgroup_layout(1, wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT),
               &DefaultBindGroups::LightUniform);

    let _ = layout.add(&(0,1),
               &create_texture(0, wgpu::ShaderStages::FRAGMENT),
               &DefaultBindGroups::Texture1);

    let _ = layout.add(&(1,1),
               &create_texture_sampler(1, wgpu::ShaderStages::FRAGMENT),
               &DefaultBindGroups::Texture1Sampler);

    let _ = layout.add(&(2,1),
               &create_texture(2, wgpu::ShaderStages::FRAGMENT),
               &DefaultBindGroups::Texture2);

    let _ = layout.add(&(3,1),
               &create_texture_sampler(3, wgpu::ShaderStages::FRAGMENT),
               &DefaultBindGroups::Texture2Sampler);

    // Create wgsl module.
    let wgsl_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor { // Create shaders before this.
        label: Some("Default render shader"),
        source: wgpu::ShaderSource::Wgsl(
            Cow::Borrowed(include_str!("assets/wgsl/v4n4_camera_light_tex2.wgsl"))),

    });

    // Create pipeline layout
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Default pipeline layout"),
        bind_group_layouts: &layout.create_bind_group_layouts(device).iter().collect::<Vec<_>>(),
        push_constant_ranges: &[],
    });

    // Calculate stride and create vertex attributes.
    let (stride, attributes) = create_vertex_attributes(&vec![wgpu::VertexFormat::Float32x4, wgpu::VertexFormat::Float32x4]);


    // Create primitive state
    let primitive_state = wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw, //  } else { wgpu::FrontFace::Cw },
        cull_mode: None, //Some(wgpu::Face::Back),
                         // cull_mode: Some(wgpu::Face::Front),
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false,
    };

    let depth_stencil = Some(wgpu::DepthStencilState {
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

    let multisample = wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
    };

    let binding = [Some(wgpu::ColorTargetState {
               format: sc_desc.format,
               blend: None, //Some(wgpu::BlendState {
                      //     color: wgpu::BlendComponent {
                      //          src_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                      //          dst_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                      //          operation: wgpu::BlendOperation::Max,
                      //     },
                      //     alpha: wgpu::BlendComponent {
                      //          src_factor: wgpu::BlendFactor::SrcAlpha,
                      //          dst_factor: wgpu::BlendFactor::One,
                      //          operation: wgpu::BlendOperation::Add,
                      //     },
                      // }),
               // alpha_blend: wgpu::BlendState::REPLACE,
               // color_blend: wgpu::BlendState::REPLACE,
               write_mask: wgpu::ColorWrites::COLOR,
           })];


    let fragment_state = Some(wgpu::FragmentState {
           module: wgsl_module,
           entry_point: "fs_main",
           targets: &binding,
       });

    let multiview = None;

    // Create vertex state for pipeline.
    let mut vertex_state = VertexStateWrapper::init();     
    vertex_state.create_vertex_wrapper(
        stride,
        wgpu::VertexStepMode::Vertex,
        &attributes,
        &wgsl_module,
        "vs_main");

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
        &depth_stencil,
        multisample,
        &fragment_state,
        &multiview,
        Some("Jeejee")
    )
}
