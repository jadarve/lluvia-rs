use lluvia_vk::program::Program;
use pyo3::prelude::*;

#[pyclass(from_py_object, name = "Program")]
#[derive(Clone)]
pub struct PyProgram {
    pub(crate) inner: Program,
}

#[pymethods]
impl PyProgram {
    // Methods if any
}
