use std::hash::Hash;
use std::string::String;
use std::collections::HashMap;
// use wgpu::BindGroupLayoutEntry;

type EntryLocation = (i32, i32);

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



// LayoutMapper::add(EntryLocation(0,1), 
//                   Option<T>,
//               create_uniform_bindgroup_layout(0, wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT),
//               MyShaderLayout::CameraUniformLayout)
//     );

struct LayoutData {
    bind_group_layout_entry: wgpu::BindGroupLayoutEntry, 
    entry_location: EntryLocation,
}

pub struct LayoutMapper<T: std::cmp::Eq + Hash + Copy > {
    layout_data: Vec<LayoutData>,
    mapping: HashMap<T, u32>, 
}
 
impl<T: std::cmp::Eq + Hash + Copy> LayoutMapper<T> { 
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
            self.mapping.insert(*tag, index as u32);  
            Ok(())
        }
    }
}

// impl<T> LayoutEntries<T> {
//     
//     /// Initializer.
//     pub fn init() -> Self {
//         Self {
//             layouts: HashMap::with_capacity(15),
//             layout_names_to_location: None,
//             layout_names_to_bind_group_index: None,
//         }
//     }
// 
//     /// Insert a layout entry at some location. It's now allowed to store entry layout a location
//     /// that already exists.
//     pub fn insert(&mut self, location: EntryLocation, entry: wgpu::BindGroupLayoutEntry, tag: &Option<T>) -> Result<(), String> {
//         if self.layouts.contains_key(&location) {
//             Err(format!("LayoutEntries already contains entry {:?}. A location can only be inserted once.", location))
//         }
//         else {
//             self.layouts.insert(location, entry);
//             Ok(())
//         }
//     }
// 
//     /// Validate layout entries.
//     /// Checks that all layout entries are in sequences (0,0), (0,1), (0,3) results an error
//     /// because (0,2) entry layout is missing.
//     pub fn validate(&self) -> Result<(), String> {
//         unimplemented!("Not implemented yet.")
//     }
// 
//     /// Create bind group layouts from entries.
//     pub fn create_bind_group_layouts(&self) -> Vec<wgpu::BindGroupLayout> {
// 
//         // let mut keys = self.layouts.keys().collect::<Vec<_>>();
//         // keys.sort_unstable();
// 
//         // let mut result;
//         // for k in keys.iter() {
//         //     result.push_back(self.layouts.get().as_ref().to_owned());
//         // }
//         // self.layouts.into_iter().
//         unimplemented!("Not implemented yet.")
//     }
// }

// pub fn create_render_pipeline(device: &wgpu::Device,
