use crate::buffer::PyBuffer;
use crate::node::PyComputeNode;
use lluvia_vk::command_buffer::{CommandBuffer, CommandBufferBuilder};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

#[pyclass(unsendable, name = "CommandBufferBuilder")]
pub struct PyCommandBufferBuilder {
    pub(crate) inner: Option<CommandBufferBuilder>,
}

#[pymethods]
impl PyCommandBufferBuilder {
    pub fn copy_buffer(&mut self, src: &PyBuffer, dst: &PyBuffer) -> PyResult<()> {
        if let Some(builder) = &mut self.inner {
            builder
                .copy_buffer(src.inner.clone(), dst.inner.clone())
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            Ok(())
        } else {
            Err(PyRuntimeError::new_err("CommandBufferBuilder already built"))
        }
    }

    pub fn record_compute_node(&mut self, node: &mut PyComputeNode) -> PyResult<()> {
        if let Some(builder) = &mut self.inner {
            builder
                .record_compute_node(&mut node.inner)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            Ok(())
        } else {
            Err(PyRuntimeError::new_err("CommandBufferBuilder already built"))
        }
    }

    pub fn build(&mut self) -> PyResult<PyCommandBuffer> {
        if let Some(builder) = self.inner.take() {
            let inner = builder
                .build_command_buffer()
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            Ok(PyCommandBuffer { inner })
        } else {
            Err(PyRuntimeError::new_err("CommandBufferBuilder already built"))
        }
    }
}

#[pyclass(unsendable, name = "CommandBuffer")]
pub struct PyCommandBuffer {
    pub(crate) inner: CommandBuffer,
}
