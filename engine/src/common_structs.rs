use crate::impl_convert;
use crate::misc::Convert2Vec;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct DrawIndirect {
    pub vertex_count: u32, // The number of vertices to draw.
    pub instance_count: u32, // The number of instances to draw.
    pub base_vertex: u32, // The Index of the first vertex to draw.
    pub base_instance: u32, // The instance ID of the first instance to draw.
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct DispatchIndirect {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl_convert!{DrawIndirect}
impl_convert!{DispatchIndirect}
