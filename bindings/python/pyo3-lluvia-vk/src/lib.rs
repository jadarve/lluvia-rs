use pyo3::prelude::*;

mod session;
use session::PySession;

mod buffer;
use buffer::PyBuffer;

mod program;
use program::PyProgram;

mod image;
use image::{
    PyChannelCount, PyChannelType, PyImage, PyImageAddressMode, PyImageDescriptor, PyImageFilterMode, PyImageView,
    PyImageViewDescriptor,
};

mod node;
use node::{PyComputeNode, PyComputeNodeDescriptor, PyPortDescriptor, PyPortDirection, PyPortType};

mod command_buffer;
use command_buffer::{PyCommandBuffer, PyCommandBufferBuilder};

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn lluvia_vk(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySession>()?;
    m.add_class::<PyBuffer>()?;
    m.add_class::<PyProgram>()?;
    m.add_class::<PyImage>()?;
    m.add_class::<PyImageDescriptor>()?;
    m.add_class::<PyImageView>()?;
    m.add_class::<PyImageViewDescriptor>()?;
    m.add_class::<PyChannelType>()?;
    m.add_class::<PyChannelCount>()?;
    m.add_class::<PyImageFilterMode>()?;
    m.add_class::<PyImageAddressMode>()?;
    m.add_class::<PyComputeNode>()?;
    m.add_class::<PyComputeNodeDescriptor>()?;
    m.add_class::<PyPortDescriptor>()?;
    m.add_class::<PyPortType>()?;
    m.add_class::<PyPortDirection>()?;
    m.add_class::<PyCommandBuffer>()?;
    m.add_class::<PyCommandBufferBuilder>()?;
    Ok(())
}
