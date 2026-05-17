use crate::buffer::PyBuffer;
use crate::image::PyImageView;
use crate::program::PyProgram;
use lluvia_vk::node::{ComputeNode, ComputeNodeDescriptor, NodePort, PortDescriptor, PortDirection, PortType};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

#[pyclass(name = "PortDirection")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyPortDirection {
    In,
    Out,
}

impl From<PyPortDirection> for PortDirection {
    fn from(dir: PyPortDirection) -> Self {
        match dir {
            PyPortDirection::In => PortDirection::In,
            PyPortDirection::Out => PortDirection::Out,
        }
    }
}

#[pyclass(name = "PortType")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyPortType {
    Buffer,
    ImageView,
}

impl From<PyPortType> for PortType {
    fn from(pt: PyPortType) -> Self {
        match pt {
            PyPortType::Buffer => PortType::Buffer,
            PyPortType::ImageView => PortType::ImageView,
        }
    }
}

#[pyclass(name = "PortDescriptor")]
#[derive(Clone)]
pub struct PyPortDescriptor {
    pub(crate) inner: PortDescriptor,
}

#[pymethods]
impl PyPortDescriptor {
    #[new]
    pub fn new(binding: u32, name: String, direction: PyPortDirection, port_type: PyPortType) -> Self {
        Self {
            inner: PortDescriptor {
                binding,
                name,
                direction: direction.into(),
                port_type: port_type.into(),
            },
        }
    }
}

#[pyclass(name = "ComputeNodeDescriptor")]
#[derive(Clone)]
pub struct PyComputeNodeDescriptor {
    pub(crate) inner: ComputeNodeDescriptor,
}

#[pymethods]
impl PyComputeNodeDescriptor {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: ComputeNodeDescriptor::default(),
        }
    }

    pub fn program(&mut self, program: &PyProgram) {
        // Need to clone inner since builder takes self, then replace
        let inner = self.inner.clone().program(program.inner.clone());
        self.inner = inner;
    }

    pub fn function_name(&mut self, name: &str) {
        let inner = self.inner.clone().function_name(name);
        self.inner = inner;
    }

    pub fn local_shape(&mut self, shape: [u32; 3]) {
        let inner = self.inner.clone().local_shape(shape);
        self.inner = inner;
    }

    pub fn grid_shape(&mut self, shape: [u32; 3]) {
        let inner = self.inner.clone().grid_shape(shape);
        self.inner = inner;
    }

    pub fn configure_grid_shape(&mut self, global_shape: [u32; 3]) {
        let inner = self.inner.clone().configure_grid_shape(global_shape);
        self.inner = inner;
    }

    pub fn add_port(&mut self, port: &PyPortDescriptor) {
        let inner = self.inner.clone().add_port(port.inner.clone());
        self.inner = inner;
    }
}

#[pyclass(name = "ComputeNode")]
pub struct PyComputeNode {
    pub(crate) inner: ComputeNode,
}

#[pymethods]
impl PyComputeNode {
    pub fn bind_buffer(&mut self, name: &str, buffer: &PyBuffer) -> PyResult<()> {
        use lluvia_vk::node::Node;
        self.inner
            .bind(name, NodePort::Buffer(buffer.inner.clone()))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    pub fn bind_image_view(&mut self, name: &str, view: &PyImageView) -> PyResult<()> {
        use lluvia_vk::node::Node;
        self.inner
            .bind(name, NodePort::ImageView(view.inner.clone()))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    pub fn grid_shape(&self) -> [u32; 3] {
        self.inner.grid_shape()
    }

    pub fn set_grid_shape(&mut self, shape: [u32; 3]) {
        self.inner.set_grid_shape(shape)
    }

    pub fn local_shape(&self) -> [u32; 3] {
        self.inner.local_shape()
    }
}
