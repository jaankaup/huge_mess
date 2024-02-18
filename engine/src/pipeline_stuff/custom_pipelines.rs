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
        label: Some("Default render shader"),
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
