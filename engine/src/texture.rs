use crate::core::WGPUContext;

/// Texture.
#[allow(dead_code)]
pub struct Texture {
    texture: Option<wgpu::Texture>,
    view: Option<wgpu::TextureView>,
    sampler: Option<wgpu::Sampler>,
    // DO we need these? TODO: remove if now used.
    // width: u32,
    // height: u32,
    // depth: u32,
}

    /// TODO: how to determine address_mode_x: a struct?
    /// TODO: how to determine filters: a struct?
    /// TODO: how to compare function: parameter/struct?
    /// TODO: multisampling: parameter/struct?
    /// TODO: load png files parallel/simd?
    /// TODO: error handling and validation?

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Create a depth texture. Should we have more parameters for depth texture?
    pub fn create_depth_texture(context: &WGPUContext, sc_desc: &wgpu::SurfaceConfiguration, label: Option<&str>) -> Self {

        log::debug!("Creating depth texture");

        // TODO: refactor if we do not need these.
        let width = sc_desc.width;
        let height = sc_desc.height;
        let depth = 1;

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: depth,
        };
        let desc = wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let texture = context.device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::Less),
            ..Default::default()
        });

        Self { texture: Some(texture), view: Some(view), sampler: Some(sampler), } // width, height, depth }
        // Self { texture: Some(texture), view: Some(view), sampler: Some(sampler), width, height, depth }
    }

    pub fn get_view(&self) -> &Option<wgpu::TextureView>  {
        &self.view
    }
}
