use wgpu::BindGroupLayout;
use std::fmt::Debug;
use std::num::NonZeroU32;
use std::hash::Hash;
use std::string::String;
use std::collections::HashMap;
use wgpu::Label;
// use wgpu::BindGroupLayoutEntry;

pub type EntryLocation = (u32, u32);

/// A data structure that holds information about pipeline layout entries.
/// Type T is type of key which can be used to access a layout entry.
///
/// Example usage:
///
/// enum MyShaderLayout { 
///     CameraUniformLayout,
///     DirectionalLightUniform,
///     TerrainDiffuseTexture,
///     TerrainDiffuseSampler,
///     WaterDiffuseTexture,
///     WaterDiffuseSampler,
/// }
/// let entries = LayoutEntries<MyShaderLayout>::init();
///
/// entries.insert(EntryLocation(0,0),
///                create_uniform_bindgroup_layout(0, wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT),
///                MyShaderLayout::CameraUniformLayout);
///
///
///
///
///

pub struct BindGroupMapper {
    bind_group_layout_entries: Vec<Vec<Option<wgpu::BindGroupLayoutEntry>>>,
    bind_group_layouts: Vec<wgpu::BindGroupLayout>,
}

impl BindGroupMapper {

    pub fn init(device: &wgpu::Device) -> Self {
        let limits = device.limits();
        let bind_groups = limits.max_bind_groups; 
        let bindings = limits.max_bindings_per_bind_group; 

        log::info!("Initializing BindGroupMapper with dimensions [{:?}, {:?}]", bind_groups, bindings);

        Self {
            bind_group_layout_entries: vec![vec![None ; bindings.try_into().unwrap()] ; bind_groups.try_into().unwrap()],
            bind_group_layouts: Vec::with_capacity(40),
        }
    }

    pub fn insert(&mut self, device: &wgpu::Device, group_index: u32, bind_group_layout_entry: &wgpu::BindGroupLayoutEntry) {

        debug_assert!(group_index < device.limits().max_bind_groups); 
        debug_assert!(bind_group_layout_entry.binding < device.limits().max_bindings_per_bind_group); 
        debug_assert!(self.bind_group_layout_entries[group_index as usize][bind_group_layout_entry.binding as usize].is_none()); 

        self.bind_group_layout_entries[group_index as usize][bind_group_layout_entry.binding as usize] = Some(*bind_group_layout_entry);
    }

    pub fn build_bind_group_layouts(&mut self, device: &wgpu::Device) {

        // TODO: validation for mapping => layout

        self.bind_group_layouts.clear();

        for x in self.bind_group_layout_entries.iter() {
            let mut temp: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
            for y in x.iter() {
                if y.is_none() {
                    break;
                }
                else {
                    temp.push(y.unwrap());
                }
            }
            if temp.len() > 0 {
                self.bind_group_layouts.push(
                    device.create_bind_group_layout(
                        &wgpu::BindGroupLayoutDescriptor {
                            entries: &temp,
                            label: None,
                        }
                    )
                );
            }
        }
    }

    pub fn get_bind_group_layouts(&self) -> &Vec<wgpu::BindGroupLayout> {
        debug_assert!(self.bind_group_layouts.len() > 0);
        &self.bind_group_layouts
    }

    /// Create a bind group for group.
    pub fn create_bind_group(&self, device: &wgpu::Device, resources: &Vec<wgpu::BindingResource>, group_index: usize) -> wgpu::BindGroup {

        // Does the group exist?
        debug_assert!(self.bind_group_layouts.len() < group_index);
        debug_assert!(self.bind_group_layouts.len() == resources.len());

        // Create entries.
        let entries = resources.iter().enumerate().map(|(ind, res)| wgpu::BindGroupEntry { binding: ind as u32, resource: res.clone(), }).collect::<Vec<_>>(); 

        device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layouts[group_index],
                entries: &entries,
            }
        )
    }
}

/// A wrapper for render pipeline. Do we need a wrapper, or just a function? 
//pub struct RenderPipelineWrapper<T: std::cmp::Eq + Hash + Copy + Debug> {
pub struct RenderPipelineWrapper {
    pipeline: wgpu::RenderPipeline,
    layout_mapper: BindGroupMapper,
}

// impl<T: std::cmp::Eq + Hash + Copy + Debug> RenderPipelineWrapper<T> {
impl RenderPipelineWrapper {
    pub fn init(
            device: &wgpu::Device,
            layout: &wgpu::PipelineLayout,
            vertex_state: &wgpu::VertexState,
            primitive_state: &wgpu::PrimitiveState,
            depth_stencil_state: &Option<wgpu::DepthStencilState>,
            multisample_state: wgpu::MultisampleState,
            fragment_state: &Option<wgpu::FragmentState>,
            multiview: &Option<NonZeroU32>,
            label: Label,
            bind_group_mapper: BindGroupMapper
            ) -> Self {

     // Create the render pipeline.
     let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
         label: label,
         layout: Some(layout),
         vertex: vertex_state.to_owned(),
         primitive: *primitive_state,
         depth_stencil: if depth_stencil_state.is_none() { None } else { depth_stencil_state.to_owned() },
         multisample: multisample_state,
         fragment: if fragment_state.is_none() { None } else { fragment_state.to_owned() },
         multiview: *multiview,
     });

     Self {
         pipeline: pipeline,
         layout_mapper: bind_group_mapper,
     }


        // Create the render pipeline.
     //   let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
     //       label: None,
     //       layout: Some(&pipeline_layout),
     //       vertex: wgpu::VertexState {
     //           module: wgsl_module,
     //           entry_point: "vs_main",
     //           buffers: &[
     //               wgpu::VertexBufferLayout {
     //                   array_stride: stride,
     //                   step_mode: wgpu::VertexStepMode::Vertex,
     //                   attributes: &attributes,
     //               }],
     //       },
     //       primitive: wgpu::PrimitiveState {
     //           //topology: wgpu::PrimitiveTopology::TriangleList,
     //           topology,
     //           strip_index_format: None,
     //           front_face: if ccw { wgpu::FrontFace::Ccw } else { wgpu::FrontFace::Cw },
     //           cull_mode: None, //Some(wgpu::Face::Back),
     //           // cull_mode: Some(wgpu::Face::Front),
     //           unclipped_depth: false, // ???
     //           polygon_mode: wgpu::PolygonMode::Fill,
     //           conservative: false,
     //       },
     //       depth_stencil: Some(wgpu::DepthStencilState {
     //           format: wgpu::TextureFormat::Depth32Float,
     //           depth_write_enabled: true,
     //           depth_compare: wgpu::CompareFunction::Less,
     //           stencil: wgpu::StencilState {
     //               front: wgpu::StencilFaceState::IGNORE,
     //               back: wgpu::StencilFaceState::IGNORE,
     //               read_mask: 0,
     //               write_mask: 0,
     //           },
     //           bias: wgpu::DepthBiasState {
     //               constant: 0,
     //               slope_scale: 0.0,
     //               clamp: 0.0,
     //           },
     //       }),
     //       multisample: wgpu::MultisampleState {
     //           count: 1,
     //           mask: !0,
     //           alpha_to_coverage_enabled: false,
     //       },
     //       fragment: Some(wgpu::FragmentState {
     //           module: wgsl_module,
     //           entry_point: "fs_main",
     //           targets: &[Some(wgpu::ColorTargetState {
     //               format: sc_desc.format,
     //               blend: None, //Some(wgpu::BlendState {
     //                      //     color: wgpu::BlendComponent {
     //                      //          src_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
     //                      //          dst_factor: wgpu::BlendFactor::OneMinusDstAlpha,
     //                      //          operation: wgpu::BlendOperation::Max,
     //                      //     },
     //                      //     alpha: wgpu::BlendComponent {
     //                      //          src_factor: wgpu::BlendFactor::SrcAlpha,
     //                      //          dst_factor: wgpu::BlendFactor::One,
     //                      //          operation: wgpu::BlendOperation::Add,
     //                      //     },
     //                      // }),
     //               // alpha_blend: wgpu::BlendState::REPLACE,
     //               // color_blend: wgpu::BlendState::REPLACE,
     //               write_mask: wgpu::ColorWrites::COLOR,
     //           })],
     //       }),
     //       multiview: None,
     //   });

     //   Self {
     //       pipeline: pipeline,
     //   }
     }
}
