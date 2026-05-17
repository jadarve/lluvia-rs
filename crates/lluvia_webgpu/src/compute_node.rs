use std::collections::HashMap;

use crate::{Buffer, ShaderModule};

#[derive(Clone, Debug, PartialEq)]
pub enum PortDirection {
    Input,
    Output,
    InputOutput,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PortType {
    Buffer,
}

#[derive(bon::Builder, Clone, Debug)]
pub struct PortDescriptor {
    pub(crate) name: String,
    pub(crate) direction: PortDirection,
    pub(crate) port_type: PortType,
    pub(crate) binding: u32,
    // TODO: may need to support group binding
}

impl From<&PortDescriptor> for wgpu::BindGroupLayoutEntry {
    fn from(port: &PortDescriptor) -> Self {
        let read_only = match port.direction {
            PortDirection::Input => true,
            PortDirection::Output => false,
            PortDirection::InputOutput => false,
        };

        // TODO: all shader stages?
        let visibility = wgpu::ShaderStages::COMPUTE;

        let ty = match port.port_type {
            PortType::Buffer => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        };

        wgpu::BindGroupLayoutEntry {
            binding: port.binding,
            visibility,
            ty,
            count: None,
        }
    }
}

#[derive(bon::Builder, Debug, Clone)]
pub struct ComputeNodeDescriptor {
    pub(crate) label: Option<String>,
    pub(crate) shader_module: ShaderModule,
    pub(crate) entry_point: String,
    pub(crate) ports: Vec<PortDescriptor>,
}

pub struct ComputeNode {
    pub(crate) desc: ComputeNodeDescriptor,
    pub(crate) device: wgpu::Device,
    pub(crate) handle: wgpu::ComputePipeline,

    bindings: tokio::sync::Mutex<HashMap<String, Buffer>>,
}

impl ComputeNode {
    pub(crate) fn new(desc: &ComputeNodeDescriptor, device: wgpu::Device, handle: wgpu::ComputePipeline) -> Self {
        Self {
            desc: desc.clone(),
            device,
            handle,
            bindings: tokio::sync::Mutex::new(HashMap::new()),
        }
    }

    pub async fn bind(&self, name: &str, buffer: &Buffer) {
        // check if there is a port with the same name
        if !self.desc.ports.iter().any(|port| port.name == name) {
            // TODO: return an error
            panic!("Port with name {} not found", name);
        }

        if let Some(port_desc) = self.desc.ports.iter().find(|port| port.name == name) {
            // check if the buffer is compatible with the port
            if port_desc.port_type != PortType::Buffer {
                // TODO: return an error
                panic!("Port with name {} is not a buffer", name);
            }

            self.bindings.lock().await.insert(name.to_string(), buffer.to_owned());
        }
        // check if the buffer is compatible with the port
    }

    pub(crate) async fn get_bind_group(&self) -> wgpu::BindGroup {
        // TODO: I need a device to create all this...

        let bind_group_layout = self.handle.get_bind_group_layout(0);

        let mut bind_group_entries: Vec<wgpu::BindGroupEntry> = Vec::new();

        let bindings = self.bindings.lock().await;
        for (name, buffer) in bindings.iter() {
            let port_desc = self.desc.ports.iter().find(|port| port.name == *name).unwrap(); // FIXME: return error

            let binding = port_desc.binding;
            let resource = buffer.handle.as_entire_binding();

            bind_group_entries.push(wgpu::BindGroupEntry { binding, resource });
        }

        

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        })
    }
}
