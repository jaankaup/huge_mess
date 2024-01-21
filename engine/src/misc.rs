// use bytemuck::{Pod, Zeroable};

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
