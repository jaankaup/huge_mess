use std::ops::Range;
use crate::texture::Texture;
use crate::render_pass::create_render_pass;
use wgpu::util::RenderEncoder;

/// A basic draw command.
pub fn draw(encoder: &mut wgpu::CommandEncoder,
            view: &wgpu::TextureView,
            depth_texture: Option<&Texture>,
            bind_groups: &Vec<&wgpu::BindGroup>,
            pipeline: &wgpu::RenderPipeline,
            draw_buffer: &wgpu::Buffer,
            range: Range<u32>,
            clear_color: &Option<wgpu::Color>,
            clear: bool) {

    let mut render_pass = create_render_pass(
                          encoder,
                          view,
                          depth_texture,
                          clear,
                          clear_color,
                          &None
    );
    
    render_pass.set_pipeline(&pipeline);

    // Set bind groups.
    for (e, bgs) in bind_groups.iter().enumerate() {
        render_pass.set_bind_group(e as u32, bgs, &[]);
    }
    
    // Set vertex buffer.
    render_pass.set_vertex_buffer(
        0,
        draw_buffer.slice(..)
    );
    
    render_pass.draw(range, 0..1);
}

/// A basic draw command for indirect buffer.
pub fn draw_indirect(
            encoder: &mut wgpu::CommandEncoder,
            view: &wgpu::TextureView,
            depth_texture: Option<&Texture>,
            bind_groups: &Vec<&wgpu::BindGroup>,
            pipeline: &wgpu::RenderPipeline,
            draw_buffer: &wgpu::Buffer,
            indirect_buffer: &wgpu::Buffer,
            offset: wgpu::BufferAddress,
            clear_color: &Option<wgpu::Color>,
            clear: bool) {

    let mut render_pass = create_render_pass(
                          encoder,
                          view,
                          depth_texture,
                          clear,
                          clear_color,
                          &None
    );

    render_pass.set_pipeline(&pipeline);

    // Set bind groups.
    for (e, bgs) in bind_groups.iter().enumerate() {
        render_pass.set_bind_group(e as u32, &bgs, &[]);
    }

    // Set vertex buffer.
    render_pass.set_vertex_buffer(
        0,
        draw_buffer.slice(..)
    );

    render_pass.draw_indirect(indirect_buffer, offset);
}
