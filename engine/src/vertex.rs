/// Takes wgpu::VertexFormats as input and returns (stride, Vec<wgpu::VertexAttribute>)
pub fn create_vertex_attributes(formats: &Vec<wgpu::VertexFormat>) -> (u64, Vec<wgpu::VertexAttribute>) { 

    let mut attribute_descriptors: Vec<wgpu::VertexAttribute> = Vec::new();
    let mut stride: u64 = 0;
    for (i, format) in formats.iter().enumerate() {
        let size = match format {
                wgpu::VertexFormat::Uint8x2 => 2 * std::mem::size_of::<u8>() as u64, 
                wgpu::VertexFormat::Uint8x4 => 4 * std::mem::size_of::<u8>() as u64,
                wgpu::VertexFormat::Sint8x2 => 2 * std::mem::size_of::<i8>() as u64,
                wgpu::VertexFormat::Sint8x4 => 4 * std::mem::size_of::<i8>() as u64,
                wgpu::VertexFormat::Unorm8x2 => 2 * std::mem::size_of::<u8>() as u64,
                wgpu::VertexFormat::Unorm8x4 => 4 * std::mem::size_of::<u8>() as u64,
                wgpu::VertexFormat::Snorm8x2 => 2 * std::mem::size_of::<u8>() as u64,
                wgpu::VertexFormat::Snorm8x4 => 4 * std::mem::size_of::<u8>() as u64,
                wgpu::VertexFormat::Uint16x2 => 2 * std::mem::size_of::<u16>() as u64,
                wgpu::VertexFormat::Uint16x4 => 4 * std::mem::size_of::<u16>() as u64,
                wgpu::VertexFormat::Sint16x2 => 2 * std::mem::size_of::<i16>() as u64,
                wgpu::VertexFormat::Sint16x4 => 4 * std::mem::size_of::<i16>() as u64,
                wgpu::VertexFormat::Unorm16x2 => 2 * std::mem::size_of::<u16>() as u64,
                wgpu::VertexFormat::Unorm16x4 => 4 * std::mem::size_of::<u16>() as u64,
                wgpu::VertexFormat::Snorm16x2 => 2 * std::mem::size_of::<i16>() as u64,
                wgpu::VertexFormat::Snorm16x4 => 4 * std::mem::size_of::<i16>() as u64,
                wgpu::VertexFormat::Float16x2 => unimplemented!(),
                wgpu::VertexFormat::Float16x4 => unimplemented!(),
                wgpu::VertexFormat::Float32 => std::mem::size_of::<f32>() as u64,
                wgpu::VertexFormat::Float32x2 => 2 * std::mem::size_of::<f32>() as u64,
                wgpu::VertexFormat::Float32x3 => 3 * std::mem::size_of::<f32>() as u64,
                wgpu::VertexFormat::Float32x4 => 4 * std::mem::size_of::<f32>() as u64,
                wgpu::VertexFormat::Uint32 => std::mem::size_of::<u32>() as u64,
                wgpu::VertexFormat::Uint32x2 => 2 * std::mem::size_of::<u32>() as u64,
                wgpu::VertexFormat::Uint32x3 => 3 * std::mem::size_of::<u32>() as u64,
                wgpu::VertexFormat::Uint32x4 => 4 * std::mem::size_of::<u32>() as u64,
                wgpu::VertexFormat::Sint32 => std::mem::size_of::<i32>() as u64,
                wgpu::VertexFormat::Sint32x2 => 2 * std::mem::size_of::<i32>() as u64,
                wgpu::VertexFormat::Sint32x3 => 3 * std::mem::size_of::<i32>() as u64,
                wgpu::VertexFormat::Sint32x4 => 4 * std::mem::size_of::<i32>() as u64,
                wgpu::VertexFormat::Float64
                | wgpu::VertexFormat::Float64x2
                | wgpu::VertexFormat::Float64x3
                | wgpu::VertexFormat::Float64x4
                => panic!("VERTEX_ATTRIBUTE_64BIT must be enabled to use Double formats")
        };
        attribute_descriptors.push(
            wgpu::VertexAttribute {
                format: *format,
                offset: stride,
                shader_location: i as u32, 
            }
        );
        stride += size;
    }

    (stride, attribute_descriptors)
}

// pub fn create_vertex_buffer_layout<'a>(step_mode: &'a wgpu::VertexStepMode, formats: &'a Vec<wgpu::VertexFormat>) -> wgpu::VertexBufferLayout<'a> {
// // pub fn create_vertex_buffer_layout(step_mode: &wgpu::VertexStepMode, formats: &Vec<wgpu::VertexFormat>) -> wgpu::VertexBufferLayout {
// 
//     let (stride, vas) = create_vertex_attributes(formats);
//     wgpu::VertexBufferLayout {
//         array_stride: stride,
//         step_mode: *step_mode,
//         attributes: &vas,
//     }
// }
//     wgpu::VertexState {
//         module: &wgsl_module,
//         entry_point: entry_point,
//         buffers: &[
//             wgpu::VertexBufferLayout {
//                 array_stride: stride,
//                 step_mode: step_mode,
//                 attributes: attributes,
//             }],
//     }
// }
