use crate::misc::Convert2Vec;
use bytemuck::Pod;
use wgpu::util::DeviceExt;
use wgpu::BufferAddress;

/// Add data to buffer. TODO: validate that there is enougt space in buffer.
pub fn add_data<T: Pod>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    t: &[T],
    buffer: &wgpu::Buffer,
    offset: BufferAddress) {

    log::info!("Writing data: offset {:?}", offset);
    queue.write_buffer(buffer, offset, bytemuck::cast_slice(t));
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Add Data") });
    queue.submit(Some(encoder.finish()));
}

/// Create wgpu::buffer from data.
pub fn buffer_from_data<T: Pod>(
    device: &wgpu::Device,
    t: &[T],
    usage: wgpu::BufferUsages,
    label: wgpu::Label)
    -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(t),
                usage,
            }
        )
}

/// Copy the content of the buffer into a vector.
pub fn to_vec<T: Convert2Vec + std::clone::Clone + bytemuck::Pod + std::marker::Send>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    src_offset: wgpu::BufferAddress,
    copy_size: wgpu::BufferAddress,
    ) -> Option<Vec<T>> {

    // TODO: Recycle staging buffers.
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: copy_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    // TODO: validate src_offset and copy_size!!!
    encoder.copy_buffer_to_buffer(buffer, src_offset, &staging_buffer, 0, copy_size);
    queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);

    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    device.poll(wgpu::Maintain::Wait).panic_on_timeout();

    // if let Ok(Ok(())) = receiver.recv_async().await {
    if let Ok(Ok(())) = receiver.recv() {

        // Gets contents of buffer
        //let data = buffer_slice.get_mapped_range();
        let data = buffer_slice.get_mapped_range().to_vec();
        // Since contents are got in bytes, this converts these bytes back to u32
        //let result = bytemuck::cast_slice(&data).to_vec();
        let res: Vec<T>;
        res = Convert2Vec::convert(&data);

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        staging_buffer.unmap(); // Unmaps buffer from memory
                                // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                //   delete myPointer;
                                //   myPointer = NULL;
                                // It effectively frees the memory

        // Returns data from buffer
        Some(res)
    }
    else {
        None
    }

    // Wasm version crashes: DOMException.getMappedRange: Buffer not mapped.
    // let data = buffer_slice.get_mapped_range().to_vec();
    // res = Convert2Vec::convert(&data);
    // drop(data);
    // staging_buffer.unmap();

    // res
}
