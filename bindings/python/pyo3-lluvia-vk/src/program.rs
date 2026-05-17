use lluvia_vk::program::Program;
use pyo3::prelude::*;
use std::sync::Arc;

#[pyclass(name = "Program")]
#[derive(Clone)]
pub struct PyProgram {
    pub(crate) inner: Arc<Program>,
}

#[pymethods]
impl PyProgram {
    // Methods if any
}
