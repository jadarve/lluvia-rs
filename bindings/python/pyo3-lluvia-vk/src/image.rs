use lluvia_vk::image::{
    ChannelCount, ChannelType, Image, ImageAddressMode, ImageDescriptor, ImageFilterMode, ImageView,
    ImageViewDescriptor,
};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::sync::Arc;

#[pyclass(from_py_object, name = "ChannelType")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyChannelType {
    Uint8,
    Int8,
    Uint16,
    Int16,
    Float16,
    Uint32,
    Int32,
    Float32,
    Uint64,
    Int64,
    Float64,
}

impl From<PyChannelType> for ChannelType {
    fn from(ct: PyChannelType) -> Self {
        match ct {
            PyChannelType::Uint8 => ChannelType::Uint8,
            PyChannelType::Int8 => ChannelType::Int8,
            PyChannelType::Uint16 => ChannelType::Uint16,
            PyChannelType::Int16 => ChannelType::Int16,
            PyChannelType::Float16 => ChannelType::Float16,
            PyChannelType::Uint32 => ChannelType::Uint32,
            PyChannelType::Int32 => ChannelType::Int32,
            PyChannelType::Float32 => ChannelType::Float32,
            PyChannelType::Uint64 => ChannelType::Uint64,
            PyChannelType::Int64 => ChannelType::Int64,
            PyChannelType::Float64 => ChannelType::Float64,
        }
    }
}

#[pyclass(from_py_object, name = "ChannelCount")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyChannelCount {
    C1 = 1,
    C2 = 2,
    C3 = 3,
    C4 = 4,
}

impl From<PyChannelCount> for ChannelCount {
    fn from(cc: PyChannelCount) -> Self {
        match cc {
            PyChannelCount::C1 => ChannelCount::C1,
            PyChannelCount::C2 => ChannelCount::C2,
            PyChannelCount::C3 => ChannelCount::C3,
            PyChannelCount::C4 => ChannelCount::C4,
        }
    }
}

#[pyclass(from_py_object, name = "ImageDescriptor")]
#[derive(Clone)]
pub struct PyImageDescriptor {
    pub(crate) inner: ImageDescriptor,
}

#[pymethods]
impl PyImageDescriptor {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: ImageDescriptor::default(),
        }
    }

    pub fn width(&mut self, w: u32) {
        self.inner.width = w;
    }

    pub fn height(&mut self, h: u32) {
        self.inner.height = h;
    }

    pub fn depth(&mut self, d: u32) {
        self.inner.depth = d;
    }

    pub fn channel_type(&mut self, ct: PyChannelType) {
        self.inner.channel_type = ct.into();
    }

    pub fn channel_count(&mut self, cc: PyChannelCount) {
        self.inner.channel_count = cc.into();
    }
}

#[pyclass(from_py_object, name = "Image")]
#[derive(Clone)]
pub struct PyImage {
    pub(crate) inner: Arc<Image>,
}

#[pymethods]
impl PyImage {
    pub fn create_image_view(&self, view_descriptor: &PyImageViewDescriptor) -> PyResult<PyImageView> {
        let inner = self
            .inner
            .create_image_view(&view_descriptor.inner)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyImageView { inner })
    }
}

#[pyclass(from_py_object, name = "ImageFilterMode")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyImageFilterMode {
    Nearest,
    Linear,
}

impl From<PyImageFilterMode> for ImageFilterMode {
    fn from(m: PyImageFilterMode) -> Self {
        match m {
            PyImageFilterMode::Nearest => ImageFilterMode::Nearest,
            PyImageFilterMode::Linear => ImageFilterMode::Linear,
        }
    }
}

#[pyclass(from_py_object, name = "ImageAddressMode")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyImageAddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
    MirrorClampToEdge,
}

impl From<PyImageAddressMode> for ImageAddressMode {
    fn from(m: PyImageAddressMode) -> Self {
        match m {
            PyImageAddressMode::Repeat => ImageAddressMode::Repeat,
            PyImageAddressMode::MirroredRepeat => ImageAddressMode::MirroredRepeat,
            PyImageAddressMode::ClampToEdge => ImageAddressMode::ClampToEdge,
            PyImageAddressMode::ClampToBorder => ImageAddressMode::ClampToBorder,
            PyImageAddressMode::MirrorClampToEdge => ImageAddressMode::MirrorClampToEdge,
        }
    }
}

#[pyclass(from_py_object, name = "ImageViewDescriptor")]
#[derive(Clone)]
pub struct PyImageViewDescriptor {
    pub(crate) inner: ImageViewDescriptor,
}

#[pymethods]
impl PyImageViewDescriptor {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: ImageViewDescriptor::default(),
        }
    }

    pub fn filter_mode(&mut self, mode: PyImageFilterMode) {
        self.inner.filter_mode = mode.into();
    }

    pub fn address_mode(&mut self, mode: PyImageAddressMode) {
        let m = mode.into();
        self.inner.address_mode_u = m;
        self.inner.address_mode_v = m;
        self.inner.address_mode_w = m;
    }

    pub fn normalized_coordinates(&mut self, enabled: bool) {
        self.inner.normalized_coordinates = enabled;
    }

    pub fn is_sampled(&mut self, sampled: bool) {
        self.inner.is_sampled = sampled;
    }
}

#[pyclass(from_py_object, name = "ImageView")]
#[derive(Clone)]
pub struct PyImageView {
    pub(crate) inner: Arc<ImageView>,
}
