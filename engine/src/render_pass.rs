use wgpu::StoreOp;
use wgpu::Label;
use crate::texture::Texture as Tex;

/// Create a render pass object. Resolve target not used yet.
/// TODO: create a RenderPayload struct to pass in?
pub fn create_render_pass<'a>(encoder: &'a mut wgpu::CommandEncoder,
                          view: &'a wgpu::TextureView,
                          // depth_texture: &'a Option<Tex>,
                          depth_texture: Option<&'a Tex>,
                          clear: bool,
                          clear_color: &Option<wgpu::Color>,
                          label: &Option<Label>) -> impl wgpu::util::RenderEncoder<'a> {

    encoder.begin_render_pass(
        &wgpu::RenderPassDescriptor {
            label: label.unwrap_or_else(|| None),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear {
                            true => {
                                wgpu::LoadOp::Clear(clear_color.unwrap())
                            }
                            false => {
                                wgpu::LoadOp::Load
                            }
                        },
                        store: StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: if depth_texture.is_none() { None } else {
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_texture.as_ref().unwrap().get_view().as_ref().unwrap(),
                    depth_ops: Some(wgpu::Operations {
                        load: match clear { true => wgpu::LoadOp::Clear(1.0), false => wgpu::LoadOp::Load },
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                })
            },
            timestamp_writes: None,
            occlusion_query_set: None,
        })
}
