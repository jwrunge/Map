//! Dynamic uniform buffer system for efficient batch rendering
//!
//! This module provides multiple uniform buffer slots to avoid overwriting
//! uniform data when rendering multiple objects in a single render pass.

use wgpu::util::DeviceExt;
use std::mem;

/// Maximum number of objects that can be rendered in a single batch
const MAX_OBJECTS_PER_BATCH: usize = 64;

/// Dynamic uniform buffer manager
pub struct DynamicUniformBuffer {
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_groups: Vec<wgpu::BindGroup>,
    uniform_size: u64,
    current_slot: usize,
}

impl DynamicUniformBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        // Calculate aligned uniform size (must be aligned to 256 bytes for uniform buffers)
        let matrix_size = mem::size_of::<[[f32; 4]; 4]>() as u64;
        let uniform_alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let uniform_size = ((matrix_size + uniform_alignment - 1) / uniform_alignment) * uniform_alignment;

        // Create large buffer that can hold multiple uniforms
        let buffer_size = uniform_size * MAX_OBJECTS_PER_BATCH as u64;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dynamic Uniform Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Dynamic Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true, // Key: dynamic offset support
                    min_binding_size: Some(std::num::NonZeroU64::new(matrix_size).unwrap()),
                },
                count: None,
            }],
        });

        // Create bind groups for each slot
        let mut bind_groups = Vec::new();
        for i in 0..MAX_OBJECTS_PER_BATCH {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Dynamic Uniform Bind Group {}", i)),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0, // Will use dynamic offset instead
                        size: Some(std::num::NonZeroU64::new(matrix_size).unwrap()),
                    }),
                }],
            });
            bind_groups.push(bind_group);
        }

        Self {
            buffer,
            bind_group_layout,
            bind_groups,
            uniform_size,
            current_slot: 0,
        }
    }

    /// Reset for new frame
    pub fn reset_frame(&mut self) {
        self.current_slot = 0;
    }

    /// Upload matrix data to next available slot and return bind group + offset
    pub fn upload_matrix(
        &mut self,
        queue: &wgpu::Queue,
        matrix: &glam::Mat4,
    ) -> Option<(&wgpu::BindGroup, u32)> {
        if self.current_slot >= MAX_OBJECTS_PER_BATCH {
            log::warn!("Dynamic uniform buffer full! Cannot render more than {} objects per batch", MAX_OBJECTS_PER_BATCH);
            return None;
        }

        let offset = self.current_slot as u64 * self.uniform_size;
        
        // Upload matrix data to the correct slot
        queue.write_buffer(
            &self.buffer,
            offset,
            bytemuck::cast_slice(matrix.as_ref()),
        );

        let bind_group = &self.bind_groups[0]; // Use first bind group with dynamic offset
        let dynamic_offset = offset as u32;
        
        self.current_slot += 1;
        Some((bind_group, dynamic_offset))
    }

    /// Get the bind group layout for pipeline creation
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Upload multiple matrices in a batch operation
    pub fn upload_matrices(&mut self, queue: &wgpu::Queue, matrices: &[glam::Mat4]) -> Vec<(&wgpu::BindGroup, u32)> {
        let mut result = Vec::new();
        
        for matrix in matrices.iter() {
            if self.current_slot >= MAX_OBJECTS_PER_BATCH {
                log::warn!("Dynamic uniform buffer full! Cannot render more than {} objects per batch", MAX_OBJECTS_PER_BATCH);
                break;
            }

            let offset = self.current_slot as u64 * self.uniform_size;
            
            // Upload matrix data to the correct slot
            queue.write_buffer(
                &self.buffer,
                offset,
                bytemuck::cast_slice(matrix.as_ref()),
            );

            let bind_group = &self.bind_groups[0]; // Use first bind group with dynamic offset
            let dynamic_offset = offset as u32;
            
            result.push((bind_group, dynamic_offset));
            self.current_slot += 1;
        }
        
        result
    }
}
