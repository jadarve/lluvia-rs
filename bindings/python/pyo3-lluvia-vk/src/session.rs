use crate::buffer::PyBuffer;
use crate::command_buffer::{PyCommandBuffer, PyCommandBufferBuilder};
use crate::image::{PyImage, PyImageDescriptor};
use crate::node::{PyComputeNode, PyComputeNodeDescriptor};
use crate::program::PyProgram;
use lluvia_vk::node::ComputeNodeDescriptor;
use lluvia_vk::{Session, SessionDescriptor};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::sync::Arc;

#[pyclass(name = "Session")]
pub struct PySession {
    pub(crate) inner: Arc<Session>,
}

#[pymethods]
impl PySession {
    #[new]
    #[pyo3(signature = (enable_debug=false))]
    pub fn new(enable_debug: bool) -> PyResult<Self> {
        let descriptor = SessionDescriptor::builder().enable_debug(enable_debug).build();

        let inner = Session::new(descriptor).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(Self { inner })
    }

    pub fn create_buffer_device_local(&self, size: u64) -> PyResult<PyBuffer> {
        let inner = self
            .inner
            .create_buffer_device_local(size)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyBuffer { inner })
    }

    pub fn create_buffer_host_visible(&self, size: u64) -> PyResult<PyBuffer> {
        let inner = self
            .inner
            .create_buffer_host_visible(size)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyBuffer { inner })
    }

    pub fn create_program(&self, spirv: Vec<u8>) -> PyResult<PyProgram> {
        let inner = self
            .inner
            .create_program(spirv)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyProgram { inner })
    }

    pub fn create_compute_node(&self, descriptor: &PyComputeNodeDescriptor) -> PyResult<PyComputeNode> {
        let program = descriptor
            .program
            .clone()
            .ok_or_else(|| PyRuntimeError::new_err("program is required"))?;

        let mut builder = ComputeNodeDescriptor::builder()
            .global_shape(descriptor.global_shape.unwrap_or(lluvia_vk::math::UVec3::ONE))
            .program(program)
            .function_name(descriptor.function_name.clone().unwrap_or_else(|| "main".to_string()));

        for port in descriptor.ports.iter() {
            builder = builder.add_port(port.clone());
        }

        let inner = self
            .inner
            .create_compute_node(builder.build())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyComputeNode { inner })
    }

    pub fn create_command_buffer_builder(&self) -> PyResult<PyCommandBufferBuilder> {
        let inner = self
            .inner
            .create_command_buffer_builder()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyCommandBufferBuilder { inner: Some(inner) })
    }

    pub fn run(&self, command_buffer: &PyCommandBuffer) -> PyResult<()> {
        let cb = command_buffer.inner.clone();
        self.inner.run(cb).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }

    pub fn create_image(&self, descriptor: &PyImageDescriptor) -> PyResult<PyImage> {
        let inner = self
            .inner
            .create_image(descriptor.inner.clone())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyImage { inner })
    }
}
