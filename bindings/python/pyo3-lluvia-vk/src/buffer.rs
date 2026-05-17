use lluvia_vk::buffer::Buffer;
use pyo3::prelude::*;
use std::sync::Arc;

#[pyclass(name = "Buffer")]
#[derive(Clone)]
pub struct PyBuffer {
    pub(crate) inner: Arc<Buffer>,
}

#[pymethods]
impl PyBuffer {
    #[getter]
    pub fn size(&self) -> u64 {
        self.inner.size()
    }

    pub fn write(&self, data: &[u8]) {
        self.inner.write(data)
    }

    pub fn read(&self) -> Vec<u8> {
        self.inner.read()
    }
}
