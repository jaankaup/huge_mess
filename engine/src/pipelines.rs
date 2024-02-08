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

/// A struct that hold information about bind group layout entry and its location.
#[derive(Copy, Clone, Debug)]
struct LayoutData {
    bind_group_layout_entry: wgpu::BindGroupLayoutEntry, 
    entry_location: EntryLocation,
}

/// TODO: documentation.
pub struct LayoutMapper<T: std::cmp::Eq + Hash + Copy + Debug > {
    layout_data: Vec<LayoutData>,
    mapping: HashMap<T, u32>, 
}
 
impl<T: std::cmp::Eq + Hash + Copy + Debug> LayoutMapper<T> { 
    /// Initialize LayoutMapper object.
    pub fn init() -> Self {
        Self {
           layout_data: Vec::with_capacity(15),
           mapping: HashMap::<T, u32>::with_capacity(15),
        }
    }
    /// Add entry location, bind group layout entry and key (tag).
    pub fn add(&mut self, entry_location: &EntryLocation, bind_group_layout_entry: &wgpu::BindGroupLayoutEntry, tag: &T) -> Result<(), String> {
        if self.mapping.contains_key(tag) {
            Err("Key already exists.".to_string()) 
        }
        // bind_group_entry Copy here?
        else {
            self.layout_data.push(LayoutData {
                bind_group_layout_entry: *bind_group_layout_entry,
                entry_location: *entry_location }
            );
            let index = self.layout_data.len() - 1; 
            println!("{:?} :: {:?} :: {:?}", entry_location, index, tag); 
            self.mapping.insert(*tag, index as u32);  
            Ok(())
        }
    }

    pub fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        self.layout_data.clone().into_iter().map(|x| x.bind_group_layout_entry).collect()
    }

    pub fn create_bind_group_layouts(&self, device: &wgpu::Device) -> Vec<wgpu::BindGroupLayout> {

        let mut bind_group_layouts: Vec<wgpu::BindGroupLayout> = Vec::with_capacity(self.layout_data.len());

        // Add to groups.
        let mut temp_map = HashMap::<u32, Vec<wgpu::BindGroupLayoutEntry>>::new();

        // Insert bindgroup layout entries to their corresponding groups.
        for e in self.layout_data.iter() {
            (*temp_map.entry(e.entry_location.1).or_insert(Vec::new())).push(e.bind_group_layout_entry);
            println!("{:?}", e);
        }

        // Sort  
        let mut temp_map_vec = temp_map.iter().collect::<Vec<_>>();
        temp_map_vec.sort_by_key(|e| e.0);





        println!("***************************************************");
        println!("{:?}", temp_map);
        println!("***************************************************");
        println!("{:?}", temp_map.len());

        for e in temp_map_vec { //into_values().collect::<Vec<_>>() { 
            bind_group_layouts.push(device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        entries: &e.1,
                        label: None,
                        // label: Some(&format!("entry {:?}", e.entry_location)[..]),
                    }
                    ));
        }

        // for e in self.layout_data.iter() {
        //     println!("{:?}", e);
        //     bind_group_layouts.push(device.create_bind_group_layout(
        //             &wgpu::BindGroupLayoutDescriptor {
        //                 entries: &[e.bind_group_layout_entry],
        //                 label: Some(&format!("entry {:?}", e.entry_location)[..]),
        //             }
        //             ));
        // }
        bind_group_layouts
    }
}


/// A wrapper for render pipeline. Do we need a wrapper, or just a function? 
pub struct RenderPipelineWrapper<T: std::cmp::Eq + Hash + Copy + Debug> {
    pipeline: wgpu::RenderPipeline,
    layout_mapper: LayoutMapper<T>,
}

impl<T: std::cmp::Eq + Hash + Copy + Debug> RenderPipelineWrapper<T> {
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
            layout_mapper: LayoutMapper<T>
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
         layout_mapper: layout_mapper,
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
