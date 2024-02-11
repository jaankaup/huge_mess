use std::num::NonZeroU32;
use crate::core::WGPUContext;

/// Texture.
#[allow(dead_code)]
pub struct Texture {
    pub texture: Option<wgpu::Texture>,
    pub view: Option<wgpu::TextureView>,
    pub sampler: Option<wgpu::Sampler>,
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

    /// Creates a texture from a sequency of bytes (expects bytes to be in png format 'rgb'). Alpha value is set to 255.
    /// Returns a rgba texture.
    /// TODO: give alpha value as function parameter.
    /// TODO: check if aplha value already exists.
    /// TODO: allow a texture to been created from non png data.
    /// TODO: sample_count is not used. Do we need it?
    pub fn create_from_bytes(queue: &wgpu::Queue, device: &wgpu::Device, sc_desc: &wgpu::SurfaceConfiguration, sample_count : u32, bytes: &[u8], label: Option<&str>) -> Self {

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None, // Some(wgpu::CompareFunction::Always),
            ..Default::default()
        });

        let png = std::io::Cursor::new(bytes);
        let decoder = png::Decoder::new(png);
        //let (info, mut reader) = decoder.read_info().expect("Can't read info!");
        let mut reader = decoder.read_info().expect("Can't read info!");
        let info = reader.info();
        let width = info.width;
        let height = info.height;
        let bits_per_pixel = info.color_type.samples() as u32;

        if !(bits_per_pixel == 3 || bits_per_pixel == 4) {
            panic!("Bits per pixel must be 3 or 4. Bits per pixel == {}", bits_per_pixel);
        }

        let mut buffer: Vec<u8> = vec![0; (info.width * bits_per_pixel * info.height) as usize ];
        reader.next_frame(&mut buffer).unwrap();

        // TODO: check the size of the image.
        let mut temp: Vec<u8> = Vec::new();

        // The png has only rgb components. Add the alpha component to each texel.
        if bits_per_pixel == 3 {
            for i in 0..buffer.len()/3 {
                let offset = i*3;
                let red: u8 = buffer[offset];
                let green: u8 = buffer[offset+1];
                let blue: u8 = buffer[offset+2];
                temp.push(blue); // blue
                temp.push(green); // green
                temp.push(red); // red
                temp.push(255); // alpha
            }
        }

        let texture_extent = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: label,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: sc_desc.format, // wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            match bits_per_pixel {
                3 => &temp,
                4 => &buffer,
                _ => panic!("Bits size of {} is not supported", bits_per_pixel),
            },
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(width * 4).unwrap().into()), // now only 4 bits per pixel is supported,
                rows_per_image: Some(NonZeroU32::new(height).unwrap().into()),
            },
            texture_extent,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(sc_desc.format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1), // std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: Some(1), // std::num::NonZeroU32::new(1),
        });

        let width = texture_extent.width;
        let height = texture_extent.height;
        let depth = texture_extent.depth_or_array_layers;

        Self {

            texture: Some(texture),
            view: Some(view),
            sampler: Some(sampler),
            // width,
            // height,
            // depth,
        }
    }
}
