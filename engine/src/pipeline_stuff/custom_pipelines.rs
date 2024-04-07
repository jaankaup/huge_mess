use bytemuck::{
    Pod,
    Zeroable,
};
use crate::buffer::buffer_from_data;
use crate::pipeline_stuff::pipeline_helper::create_render_pipeline_wrapper;
use crate::pipelines::RenderPipelineWrapper;
use crate::pipelines::BindGroupMapper;
use crate::bindgroups::{
    create_uniform_bindgroup_layout,
    create_texture,
    create_texture_sampler,
};
use std::borrow::Cow;
use crate::pipeline_stuff::pipeline_helper::create_default_depth_stencil_state;

/// Define a basic vvvvnnnn + camera + light + 2 textures render pipeline. TODO: Refactor pipeline creation.
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
        label: Some("default_render_shader_v4n4_camera_light_tex2"),
        source: wgpu::ShaderSource::Wgsl(
            Cow::Borrowed(include_str!("../../../assets/wgsl/v4n4_camera_light_tex2.wgsl"))),

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
    let depth_state = Some(create_default_depth_stencil_state());

    // Fragment state
    let binding = [Some(wgpu::ColorTargetState::from(sc_desc.format))];
    let constants = Default::default();

    let fragment_state = Some(wgpu::FragmentState {
           module: wgsl_module,
           entry_point: "fs_main",
           constants: &constants,
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
        None,
        Some("default_render_shader_v4n4_camera_light_tex2"))
}

/// Define a basic vvvc + camera render pipeline.
pub fn default_render_shader_v3c1(device: &wgpu::Device, sc_desc: &wgpu::SurfaceConfiguration) -> RenderPipelineWrapper {

    let vertex_attributes = vec![wgpu::VertexFormat::Float32x3, wgpu::VertexFormat::Uint32];
      
    let mut bind_group_mapper = BindGroupMapper::init(device);
    bind_group_mapper.insert(device, 0, &create_uniform_bindgroup_layout(0, wgpu::ShaderStages::VERTEX));
    bind_group_mapper.build_bind_group_layouts(device);

    // Create wgsl module.
    let wgsl_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("renderer_v3c1.wgsl"),
        source: wgpu::ShaderSource::Wgsl(
            Cow::Borrowed(include_str!("../../../assets/wgsl/renderer_v3c1.wgsl"))),

    });

    // Define primitive state
    let primitive_state = wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::PointList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw, //  wgpu::FrontFace::Cw
        cull_mode: None, // Some(wgpu::Face::Back),
                         // Some(wgpu::Face::Front),
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false,
    };
    let depth_state = Some(create_default_depth_stencil_state());

    // Fragment state
    let binding = [Some(wgpu::ColorTargetState::from(sc_desc.format))];
    let constants = Default::default();

    let fragment_state = Some(wgpu::FragmentState {
           module: wgsl_module,
           entry_point: "fs_main",
           constants: &constants,
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
        None,
        Some("renderer_v3c1.wgsl"))
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct RenderParams {
    scale_factor: f32,
}

pub struct RenderParamBuffer {
    _params: RenderParams,
    buffer: wgpu::Buffer,
}

impl RenderParamBuffer {
    pub fn create(device: &wgpu::Device, scale_factor: f32 ) -> Self {

        let params = RenderParams { scale_factor: scale_factor, };

        let buf = buffer_from_data::<RenderParams>(
                  &device,
                  &vec![params],
                  wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
                  Some("render params buffer.")
        );

        Self {
            _params: params,
            buffer: buf,
        }
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

/// Define a basic vvvvnnnn + camera + light + other_params
pub fn render_v4n4_camera_light_other_params(device: &wgpu::Device, sc_desc: &wgpu::SurfaceConfiguration) -> RenderPipelineWrapper {

    let vertex_attributes = vec![wgpu::VertexFormat::Float32x4, wgpu::VertexFormat::Float32x4];
      
    let mut bind_group_mapper = BindGroupMapper::init(device);
    bind_group_mapper.insert(device, 0, &create_uniform_bindgroup_layout(0, wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT));
    bind_group_mapper.insert(device, 0, &create_uniform_bindgroup_layout(1, wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT));
    bind_group_mapper.insert(device, 0, &create_uniform_bindgroup_layout(2, wgpu::ShaderStages::VERTEX));
    bind_group_mapper.build_bind_group_layouts(device);

    // Create wgsl module.
    let wgsl_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("renderer_v4n4_debug_visualizator.wgsl"),
        source: wgpu::ShaderSource::Wgsl(
            Cow::Borrowed(include_str!("../../../assets/wgsl/renderer_v4n4_debug_visualizator.wgsl"))),

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
    let depth_state = Some(create_default_depth_stencil_state());

    // Fragment state
    let binding = [Some(wgpu::ColorTargetState::from(sc_desc.format))];
    let constants = Default::default();
    //let binding = &mut Default::default();

    let fragment_state = Some(wgpu::FragmentState {
           module: wgsl_module,
           entry_point: "fs_main",
           constants: &constants,
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
        None,
        Some("renderer_v4n4_debug_visualizator.wgsl"))
}
