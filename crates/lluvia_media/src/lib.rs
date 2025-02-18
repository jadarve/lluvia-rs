pub mod common;
pub mod containers;

use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
///
/// # Arguments
///
/// * `a` - The first number.
/// * `b` - The second number.
///
/// # Returns
///
/// The sum of `a` and `b` as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn lluvia_media(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}
