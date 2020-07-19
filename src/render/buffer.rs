use bytemuck::{cast_slice, Pod};
use std::{marker::PhantomData, mem::size_of};
use wgpu::{
    Buffer, BufferAddress, BufferAsyncErr, BufferDescriptor, BufferUsage, CommandBuffer,
    CommandEncoderDescriptor, Device, Maintain,
};

/// Statically-sized wrapper around a GPU buffer.
pub struct BufferWrapper<D: Pod + Sized> {
    buffer: Buffer,
    staging_buffer: Option<Buffer>,
    capacity: BufferAddress,
    staging_capacity: BufferAddress,
    size: BufferAddress,

    _marker: PhantomData<D>,
}

impl<D: Pod + Sized> BufferWrapper<D> {
    /// The size in bytes of this buffer's data type.
    pub const DATA_SIZE: BufferAddress = size_of::<D>() as BufferAddress;

    /// Creates a new buffer wrapper with the given data and usage.
    pub fn from_data(device: &Device, data: &[D], usage: BufferUsage) -> BufferWrapper<D> {
        let buffer =
            device.create_buffer_with_data(cast_slice(data), usage | BufferUsage::COPY_DST);
        let size = data.len() as BufferAddress;

        BufferWrapper {
            buffer,
            staging_buffer: None,
            capacity: size,
            staging_capacity: 0,
            size,
            _marker: PhantomData,
        }
    }

    /// Creates a new buffer wrapper with the given capacity.
    pub fn new(device: &Device, capacity: BufferAddress, usage: BufferUsage) -> BufferWrapper<D> {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("wrapped_buffer"),
            size: capacity * BufferWrapper::<D>::DATA_SIZE,
            usage: usage | BufferUsage::COPY_DST,
        });

        BufferWrapper {
            buffer,
            staging_buffer: None,
            capacity,
            staging_capacity: 0,
            size: 0,
            _marker: PhantomData,
        }
    }

    /// Gets this buffer's size.
    pub fn size(&self) -> BufferAddress {
        self.size
    }

    /// Gets this BufferWrapper's wrapped buffer.
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Effectively clears the data from this buffer.
    pub fn clear(&mut self) {
        self.size = 0;
    }

    /// Sets the contents of this buffer.
    pub async fn replace_all(
        &mut self,
        device: &Device,
        data: &[D],
    ) -> Result<CommandBuffer, BufferWriteError> {
        let data_len = data.len() as BufferAddress;

        if data_len > self.capacity {
            return Err(BufferWriteError::InsufficientCapacity);
        }

        self.ensure_staging_capacity(device, data_len);

        let staging_buffer = self.staging_buffer.as_ref().unwrap();

        let mapping_fut = staging_buffer.map_write(0, data_len * BufferWrapper::<D>::DATA_SIZE);

        // poll this future to make sure it actually runs
        // Ideally, this would happen in a loop designed for this.
        device.poll(Maintain::Wait);

        {
            let mut mapping = mapping_fut.await?;
            mapping.as_slice().copy_from_slice(cast_slice(data));
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("buffer_staging_encoder"),
        });

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.buffer,
            0,
            data_len * BufferWrapper::<D>::DATA_SIZE,
        );

        // TODO: should we update size here?
        // Even though the command to copy the data to the actual buffer has not been
        // submitted yet?
        self.size = data_len;

        Ok(encoder.finish())
    }

    /// Append data to the end of this buffer if there is space remaining.
    pub async fn append(
        &mut self,
        device: &Device,
        data: &[D],
    ) -> Result<CommandBuffer, BufferWriteError> {
        let data_len = data.len() as BufferAddress;

        if self.size + data_len > self.capacity {
            return Err(BufferWriteError::InsufficientCapacity);
        }

        self.ensure_staging_capacity(device, data_len);

        let staging_buffer = self.staging_buffer.as_ref().unwrap();

        let mapping_fut = staging_buffer.map_write(0, data_len * BufferWrapper::<D>::DATA_SIZE);

        device.poll(Maintain::Wait);

        {
            let mut mapping = mapping_fut.await?;
            mapping.as_slice().copy_from_slice(cast_slice(data));
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("buffer_staging_encoder"),
        });

        encoder.copy_buffer_to_buffer(
            staging_buffer,
            0,
            &self.buffer,
            self.size * BufferWrapper::<D>::DATA_SIZE,
            data_len * BufferWrapper::<D>::DATA_SIZE,
        );

        // TODO: address potential issues with setting size here.
        self.size += data_len;

        Ok(encoder.finish())
    }

    /// Makes sure there is enough space in the staging buffer to handle
    /// whatever needs the staging buffer.
    fn ensure_staging_capacity(&mut self, device: &Device, size: BufferAddress) {
        if self.staging_buffer.is_none() || self.staging_capacity < size {
            self.staging_buffer = Some(device.create_buffer(&BufferDescriptor {
                label: Some("wrapped_staging_buffer"),
                size: size * BufferWrapper::<D>::DATA_SIZE,
                usage: BufferUsage::MAP_WRITE | BufferUsage::COPY_SRC,
            }));
            self.staging_capacity = size;
        }
    }
}

/// Error potentially returned from write operations.
#[derive(Debug, Copy, Clone)]
pub enum BufferWriteError {
    InsufficientCapacity,
    BufferAsyncError,
}

impl From<BufferAsyncErr> for BufferWriteError {
    fn from(_: BufferAsyncErr) -> Self {
        BufferWriteError::BufferAsyncError
    }
}
