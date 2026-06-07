use crate::buffer::PyBuffer;
use crate::image::PyImageView;
use crate::program::PyProgram;
use lluvia_vk::node::{ComputeNode, NodePort, PortDescriptor, PortDirection, PortType};
use lluvia_vk::program::Program;
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
    pub(crate) ports: Vec<PortDescriptor>,
    pub(crate) global_shape: Option<lluvia_vk::math::UVec3>,
    pub(crate) program: Option<Program>,
    pub(crate) function_name: Option<String>,
}

// FIXME: review why I need to set all attributes
#[pymethods]
impl PyComputeNodeDescriptor {
    #[new]
    pub fn new() -> Self {
        Self {
            ports: Vec::new(),
            global_shape: Some(lluvia_vk::math::UVec3::ONE),
            program: None,
            function_name: Some("main".to_string()),
        }
    }

    pub fn program(&mut self, program: &PyProgram) {
        self.program = Some(program.inner.clone());
    }

    pub fn function_name(&mut self, name: &str) {
        self.function_name = Some(name.to_string());
    }

    pub fn global_shape(&mut self, shape: [u32; 3]) {
        self.global_shape = Some(lluvia_vk::math::UVec3::new(shape[0], shape[1], shape[2]));
    }

    pub fn add_port(&mut self, port: &PyPortDescriptor) {
        self.ports.push(port.inner.clone());
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
}
