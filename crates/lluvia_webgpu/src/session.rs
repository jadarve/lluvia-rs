use std::panic::RefUnwindSafe;

use thiserror::Error;

use crate::{
    Buffer, BufferDescriptor, BufferUsages, ComputeNode, ComputeNodeDescriptor, LluviaGpuError,
    ShaderCode, ShaderModule, ShaderModuleDescriptor,
};

#[derive(Error, Debug)]
pub enum SessionError {
    /// Failed to create a session
    #[error("Failed to create a session: {0}")]
    CreateSession(String),

    /// Failed to create a device
    #[error("Failed to create a device: {0}")]
    CreateDevice(String),
}

pub struct Session {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

// FIXME: Needed to catch panics from wgpu calls
impl RefUnwindSafe for Session {}

impl Session {
    pub async fn new() -> Result<Self, SessionError> {
        // create a wgpu instance
        let instance = wgpu::Instance::default();
        println!("Instance: {:?}", instance);

        // get a wgpu adapter
        let options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        };

        let adapter =
            instance
                .request_adapter(&options)
                .await
                .ok_or(SessionError::CreateSession(
                    "No GPU adapter available".to_string(),
                ))?;

        println!("Adapter: {:?}", adapter.get_info());

        // request a wgpu device
        let desc = wgpu::DeviceDescriptor {
            label: Some("lluvia_device"),
            ..Default::default()
        };
        let (device, queue) = adapter
            .request_device(&desc, None)
            .await
            .map_err(|e| SessionError::CreateDevice(format!("Unable to create device: {e}")))?;

        Ok(Self { device, queue })
    }

    ///////////////////////////////////////////////////////////////////////////
    // Command encoder
    pub fn create_command_encoder(&self) -> crate::CommandEncoder {
        let handle = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command_encoder"),
            });

        crate::CommandEncoder::new(handle)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Work submission

    pub fn run_command_buffer(&self, command_buffer: &crate::CommandBuffer) {
        let submission_index = self.queue.submit(Some(command_buffer.handle.clone()));

        self.device
            .poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
    }

    ///////////////////////////////////////////////////////////////////////////
    // Buffer creation

    pub fn create_buffer(&self, size: usize, usage: BufferUsages, label: Option<String>) -> Buffer {
        let handle = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: label.as_deref(),
            size: size as u64,
            usage: usage.into(),
            mapped_at_creation: false,
        });

        Buffer { handle }
    }

    pub async fn create_buffer_from_descriptor(
        &self,
        desc: &BufferDescriptor,
    ) -> Result<Buffer, LluviaGpuError> {
        self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let handle = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: desc.label.as_deref(),
            size: desc.size as u64,
            usage: desc.usage.into(),
            mapped_at_creation: false,
        });

        if let Some(err) = self.device.pop_error_scope().await {
            return Err(LluviaGpuError::BufferCreationError(err.to_string()));
        }

        Ok(crate::Buffer { handle })
    }

    pub async fn buffer_map_read(&self, buffer: &Buffer) -> Result<Vec<u8>, LluviaGpuError> {
        ///////////////////////////////////////////////////////
        // read output buffer
        let buffer_slice = buffer.handle.slice(..);

        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());

        self.device.poll(wgpu::Maintain::wait()).panic_on_timeout();

        if let Ok(Ok(())) = receiver.recv_async().await {
            let data = {
                let data = buffer_slice.get_mapped_range();
                data.to_vec()
            };

            buffer.handle.unmap();
            Ok(data)
        } else {
            Err(LluviaGpuError::BufferMapError("map read error".to_string()))
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Shader module

    pub fn create_shader_module(&self, desc: &ShaderModuleDescriptor) -> ShaderModule {
        let wgpu_desc = wgpu::ShaderModuleDescriptor {
            label: desc.label.as_deref(),
            source: match &desc.code {
                ShaderCode::Wgsl(code) => wgpu::ShaderSource::Wgsl(code.into()),
            },
        };

        let wgpu_handle = self.device.create_shader_module(wgpu_desc);

        ShaderModule {
            handle: wgpu_handle,
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Compute Node

    pub fn create_compute_node(&self, desc: &ComputeNodeDescriptor) -> ComputeNode {
        let binding_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: desc.label.as_deref(),
                    entries: &desc
                        .ports
                        .iter()
                        .map(|port| port.into())
                        .collect::<Vec<wgpu::BindGroupLayoutEntry>>(),
                });

        // layout
        let layout_desc = wgpu::PipelineLayoutDescriptor {
            label: Some("compute_pipeline_layout"),
            bind_group_layouts: &[&binding_group_layout],
            push_constant_ranges: &[],
        };

        let pipeline_layout = self.device.create_pipeline_layout(&layout_desc);

        let compute_pipeline =
            self.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("compute_pipeline"),
                    layout: Some(&pipeline_layout),
                    module: &desc.shader_module.handle,
                    entry_point: Some(desc.entry_point.as_str()),
                    compilation_options: Default::default(),
                    cache: None,
                });

        ComputeNode::new(desc, self.device.clone(), compute_pipeline)

        // later for binding buffers
        // let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("bind_group"),
        //     layout: &bind_group_layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: output_buffer.as_entire_binding(),
        //     }],
        // });

        // todo!()
    }
}
