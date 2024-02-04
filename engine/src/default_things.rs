use wgpu::VertexAttribute;
use wgpu::BufferAddress;

/// A wrapper struct for wgpu::VertexState. TODO: remove. This doesn't work now.
pub struct VertexStateWrapper<'a> {
    vertex_state: Option<wgpu::VertexState<'a>>,
    buffers: Option<Vec<wgpu::VertexBufferLayout<'a>>>,
}

impl<'a> VertexStateWrapper<'a> {

    pub fn init() -> Self {
        Self {
            vertex_state: None,
            buffers: None,
        }
    }

    fn add_buffer(&mut self, stride: BufferAddress,
                      step_mode: wgpu::VertexStepMode,
                      attributes: &'a [VertexAttribute]) {

        self.buffers = Some(Vec::<wgpu::VertexBufferLayout<'a>>::new());
        self.buffers.as_mut().unwrap().push(wgpu::VertexBufferLayout {
            array_stride: stride,
            step_mode: step_mode,
            attributes: attributes,
        });
    }

    /// Initializes VertexStateWrapper.
    pub fn create_vertex_wrapper(&'a mut self,
                                 stride: BufferAddress,
                                 step_mode: wgpu::VertexStepMode,
                                 attributes: &'a [VertexAttribute],
                                 wgsl_module: &'a wgpu::ShaderModule,
                                 entry_point: &'a str) {

        self.add_buffer(stride, step_mode, attributes);
        self.vertex_state = Some(wgpu::VertexState {
            module: &wgsl_module,
            entry_point: entry_point,
            buffers: &self.buffers.as_ref().unwrap(),
        }); 
    }

    /// Get the VertexState. Panics if create_texter_wrapper is not called before this function.
    pub fn get_vertex_state(&'a self) -> &wgpu::VertexState {

        assert!(self.vertex_state.is_none());

        self.vertex_state.as_ref().unwrap()
    }
}

// pub fn create_primitive_state(topology: wgpu::PrimitiveTopology, ) -> wgpu::PrimitiveState {
//      wgpu::PrimitiveState {
//          topology: topology,
//          strip_index_format: None,
//          front_face: if ccw { wgpu::FrontFace::Ccw } else { wgpu::FrontFace::Cw },
//          cull_mode: None, //Some(wgpu::Face::Back),
//          // cull_mode: Some(wgpu::Face::Front),
//          unclipped_depth: false, // ???
//          polygon_mode: wgpu::PolygonMode::Fill,
//          conservative: false,
//      },
// }
