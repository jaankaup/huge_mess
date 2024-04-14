use crate::buffer::{buffer_from_data, to_vec};

/// Histogram struct for GPU purposes. 
pub struct Histogram {
    histogram: wgpu::Buffer,
    data: Vec<u32>,
}

impl Histogram {

    /// Create histogram with initial values.
    pub fn init(device: &wgpu::Device, initial_values: &Vec<u32>) -> Self {

        assert!(initial_values.len() > 0, "{}", format!("{} > 0", initial_values.len()));

        let histogram = buffer_from_data::<u32>(
            &device,
            &initial_values,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            None);

        Self {
            histogram: histogram,
            data: initial_values.to_owned(),
        }
    }

    /// TODO: implement wasm version! 
    pub fn get_values(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<u32> {

        // log::info!("get_values: BEGIN");
        let result = to_vec::<u32>(&device,
                                   &queue,
                                   &self.histogram,
                                   0 as wgpu::BufferAddress,
                                   (std::mem::size_of::<u32>() * self.data.len()) as wgpu::BufferAddress
        );
        // log::info!("get_values: {:?}", result);
        // log::info!("get_values: DONE");
        // TODO: validation?
        result.unwrap()
    }
    
    pub fn get_histogram_buffer(&self) -> &wgpu::Buffer {
        &self.histogram
    }

    pub fn set_values_cpu_version(&self, device: &wgpu::Device, queue: &wgpu::Queue, value: &Vec<u32>)
    {
        // Make sure the updated values are the same size as old values.
        assert!(value.len() == self.data.len(), "{}", format!("{} > {}", self.data.len(), value.len()));

        // log::info!("write_value: BEGIN HISTOGRAM ENCODER");
        // Necessery?
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Histogram set values encoder") });
        // log::info!("VVAAAALLLUUUUUUEEEEEEEEEE {:?}", value);
        queue.write_buffer(
            &self.histogram,
            0,
            bytemuck::cast_slice(&value)
        );
        queue.submit(Some(encoder.finish()));
        // log::info!("write_value: END HISTOGRAM ENCODER");
    }

    pub fn reset_all_cpu_version(&self, device: &wgpu::Device, queue: &wgpu::Queue, value: u32) {
        // Necessery?
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Histogram reset all values encoder") });
        queue.write_buffer(
            &self.histogram,
            0,
            bytemuck::cast_slice(&vec![value ; self.data.len() as usize])
        );
        queue.submit(Some(encoder.finish()));
    }
}
