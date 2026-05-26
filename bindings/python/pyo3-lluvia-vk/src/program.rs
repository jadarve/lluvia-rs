use lluvia_vk::program::Program;
use pyo3::prelude::*;

#[pyclass(name = "Program")]
#[derive(Clone)]
pub struct PyProgram {
    pub(crate) inner: Program,
}

#[pymethods]
impl PyProgram {
    // Methods if any
}
