/// A trait for types that can be copied from wgpu::buffer buffer to
/// a std::Vec. // TODO: check if there is already an implementation for this.
pub trait Convert2Vec where Self: std::marker::Sized {
    fn convert(data: &[u8]) -> Vec<Self>;  
}

#[macro_export]
macro_rules! impl_convert {
  ($to_type:ty) => {
    impl Convert2Vec for $to_type {
      fn convert(data: &[u8]) -> Vec<Self> {
            let result = data
                .chunks_exact(std::mem::size_of::<Self>())
                .map(|b| *bytemuck::try_from_bytes::<Self>(b).unwrap())
                .collect();
            result
      }
    }
  }
}

impl_convert!{u32}
impl_convert!{f32}

pub fn udiv_up_32(x: u32, y: u32) -> u32 {
  (x + y - 1) / y
}

pub fn udiv_up_safe32(x: u32, y: u32) -> u32 {
  if y == 0 { 0 } else { (x + y - 1) / y }
}


// Map index to 3d coordinate (hexahedron). The x and y dimensions are chosen. The curve goes from
// left to right, row by row.
// The z direction is "unlimited". TODO: add check for z-dimension.
pub fn index_to_uvec3(index: u32, dim_x: u32, dim_y: u32) -> [u32 ; 3] {
    let mut x = index;
    let wh    = dim_x * dim_y;
    let z     = x / wh;
    x         = x - z * wh;
    let y     = x / dim_x;
    x         = x - y * dim_x;
    [x, y, z]
}

#[inline]
pub fn uvec3_to_index(x: u32, y: u32, z: u32, dimX: u32, dimY: u32) -> u32 {
    x + y * dimX + z * dimX * dimY 
}
