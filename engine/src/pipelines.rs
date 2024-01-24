use std::string::String;
use std::collections::HashMap;
// use wgpu::BindGroupLayoutEntry;

type EntryLocation = (i32, i32);

/// A data structure that holds information about pipeline layout entries.
pub struct LayoutEntries<T> {
    layouts: HashMap<EntryLocation, wgpu::BindGroupLayoutEntry>,
    layout_names: Option<HashMap<T, EntryLocation>>,
}

impl<T> LayoutEntries<T> {
    
    /// Initializer.
    pub fn init() -> Self {
        Self {
            layouts: HashMap::new(),
            layout_names: None,
        }
    }

    /// Insert a layout entry at some location. It's now allowed to store entry layout a location
    /// that already exists.
    pub fn insert(&mut self, location: EntryLocation, name: &Option<T>) -> Result<(), String> {
        unimplemented!("Not implemented yet.")
    }

    /// Validate layout entries.
    /// Checks that all layout entries are in sequences (0,0), (0,1), (0,3) results an error
    /// because (0,2) entry layout is missing.
    pub fn validate(&self) -> Result<(), String> {
        unimplemented!("Not implemented yet.")
    }
}

// pub fn create_render_pipeline(device: &wgpu::Device,
