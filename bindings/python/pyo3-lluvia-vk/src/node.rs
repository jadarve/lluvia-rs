use crate::buffer::PyBuffer;
use crate::image::PyImageView;
use crate::program::PyProgram;
use lluvia_vk::node::{ComputeNode, ComputeNodeDescriptor, NodePort, PortDescriptor, PortDirection, PortType};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

#[pyclass(from_py_object, name = "PortDirection")]
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

#[pyclass(from_py_object, name = "PortType")]
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

#[pyclass(from_py_object, name = "PortDescriptor")]
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

#[pyclass(from_py_object, name = "ComputeNodeDescriptor")]
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
        self.inner.program = Some(program.inner.clone());
    }

    pub fn function_name(&mut self, name: &str) {
        self.inner.function_name = name.to_string();
    }

    pub fn local_shape(&mut self, shape: [u32; 3]) {
        self.inner.local_shape = lluvia_vk::math::UVec3::new(shape[0], shape[1], shape[2]);
    }

    pub fn grid_shape(&mut self, shape: [u32; 3]) {
        self.inner.grid_shape = lluvia_vk::math::UVec3::new(shape[0], shape[1], shape[2]);
    }

    pub fn configure_grid_shape(&mut self, global_shape: [u32; 3]) {
        let global = lluvia_vk::math::UVec3::new(global_shape[0], global_shape[1], global_shape[2]);
        self.inner.grid_shape.inner.x = global.inner.x.div_ceil(self.inner.local_shape.inner.x);
        self.inner.grid_shape.inner.y = global.inner.y.div_ceil(self.inner.local_shape.inner.y);
        self.inner.grid_shape.inner.z = global.inner.z.div_ceil(self.inner.local_shape.inner.z);
    }

    pub fn add_port(&mut self, port: &PyPortDescriptor) {
        self.inner.ports.push(port.inner.clone());
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
        self.inner.grid_shape().inner.to_array()
    }

    pub fn set_grid_shape(&mut self, shape: [u32; 3]) {
        self.inner
            .set_grid_shape(&lluvia_vk::math::UVec3::new(shape[0], shape[1], shape[2]))
    }

    pub fn local_shape(&self) -> [u32; 3] {
        self.inner.local_shape().inner.to_array()
    }
}
