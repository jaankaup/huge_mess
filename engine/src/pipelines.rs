use wgpu::BindGroupLayout;
use std::fmt::Debug;
use std::num::NonZeroU32;
use std::hash::Hash;
use std::string::String;
use std::collections::HashMap;
use wgpu::Label;

/// Struct for bindgroup entries and layouts.
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

    /// Add bind group entry to a group location.
    pub fn insert(&mut self, device: &wgpu::Device, group_index: u32, bind_group_layout_entry: &wgpu::BindGroupLayoutEntry) {

        debug_assert!(group_index < device.limits().max_bind_groups); 
        debug_assert!(bind_group_layout_entry.binding < device.limits().max_bindings_per_bind_group); 
        debug_assert!(self.bind_group_layout_entries[group_index as usize][bind_group_layout_entry.binding as usize].is_none()); 

        self.bind_group_layout_entries[group_index as usize][bind_group_layout_entry.binding as usize] = Some(*bind_group_layout_entry);
    }

    /// When all bind group entries has been added, this function can be used to create bind group
    /// layouts.
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

    /// Create a bind group for group. The layouts must be created before calling this function.
    pub fn create_bind_group(&self, device: &wgpu::Device, resources: &Vec<wgpu::BindingResource>, group_index: usize) -> wgpu::BindGroup {

        log::info!("Creating entries.");
        debug_assert!(group_index < self.bind_group_layouts.len());

        // Create entries.
        let entries = resources.iter().enumerate().map(|(ind, res)| wgpu::BindGroupEntry { binding: ind as u32, resource: res.clone(), }).collect::<Vec<_>>(); 

        log::info!("Creating bind group.");
        device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layouts[group_index],
                entries: &entries,
            }
        )
    }
}

/// A wrapper for render pipeline.
/// Stores the pipeline. The wrapper can be used to create bind groups for this pipeline.
pub struct RenderPipelineWrapper {
    pipeline: wgpu::RenderPipeline,
    layout_mapper: BindGroupMapper,
}

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
    }

    pub fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn create_bind_group(&self, device: &wgpu::Device, resources: &Vec<wgpu::BindingResource>, group_index: usize) -> wgpu::BindGroup {

        self.layout_mapper.create_bind_group(device, resources, group_index)
    }
}

/// A wrapper for render pipeline.
/// Stores the pipeline. The wrapper can be used to create bind groups for this pipeline.
pub struct ComputePipelineWrapper {

}

impl ComputePipelineWrapper {

}
