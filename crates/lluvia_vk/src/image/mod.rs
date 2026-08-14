//! Image and ImageView abstractions for Vulkan image management.
//!
//! This module mirrors the C++ `ll::Image`, `ll::ImageDescriptor`,
//! `ll::ImageView`, and `ll::ImageViewDescriptor` classes.

use std::sync::Arc;

use vulkano::device::DeviceOwned;

use thiserror::Error;
use vulkano::format::Format;
use vulkano::image::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo};
use vulkano::image::view::{ImageView as VkImageView, ImageViewCreateInfo};
use vulkano::image::{Image as VkImage, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};

use strum::{Display, EnumString};

#[derive(Error, Debug)]
pub enum ImageError {
    #[error("Image creation failed: {0}")]
    CreationFailed(String),

    #[error("Image view creation failed: {0}")]
    ViewCreationFailed(String),

    #[error("Invalid image descriptor: {0}")]
    InvalidDescriptor(String),

    #[error("Invalid channel count: {0}")]
    InvalidChannelCount(u32),
}

// ---------------------------------------------------------------------------
// ChannelType / ChannelCount (mirrors C++ ll::ChannelType / ll::ChannelCount)
// ---------------------------------------------------------------------------

/// Supported image channel types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum ChannelType {
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

impl ChannelType {
    /// Size of one element of this channel type in bytes.
    pub fn size_bytes(self) -> u64 {
        match self {
            Self::Uint8 | Self::Int8 => 1,
            Self::Uint16 | Self::Int16 | Self::Float16 => 2,
            Self::Uint32 | Self::Int32 | Self::Float32 => 4,
            Self::Uint64 | Self::Int64 | Self::Float64 => 8,
        }
    }
}

/// Supported image channel counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ChannelCount {
    C1 = 1,
    C2 = 2,
    C3 = 3,
    C4 = 4,
}

impl TryFrom<u32> for ChannelCount {
    type Error = ImageError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::C1),
            2 => Ok(Self::C2),
            3 => Ok(Self::C3),
            4 => Ok(Self::C4),
            _ => Err(ImageError::InvalidChannelCount(value)),
        }
    }
}

/// Maps (ChannelCount, ChannelType) → Vulkan Format, matching the C++
/// `getVulkanImageFormat` helper.
pub fn vulkan_image_format(channels: ChannelCount, channel_type: ChannelType) -> Format {
    use ChannelCount::*;
    use ChannelType::*;
    match (channels, channel_type) {
        (C1, Uint8) => Format::R8_UINT,
        (C1, Int8) => Format::R8_SINT,
        (C1, Uint16) => Format::R16_UINT,
        (C1, Int16) => Format::R16_SINT,
        (C1, Float16) => Format::R16_SFLOAT,
        (C1, Uint32) => Format::R32_UINT,
        (C1, Int32) => Format::R32_SINT,
        (C1, Float32) => Format::R32_SFLOAT,
        (C1, Uint64) => Format::R64_UINT,
        (C1, Int64) => Format::R64_SINT,
        (C1, Float64) => Format::R64_SFLOAT,

        (C2, Uint8) => Format::R8G8_UINT,
        (C2, Int8) => Format::R8G8_SINT,
        (C2, Uint16) => Format::R16G16_UINT,
        (C2, Int16) => Format::R16G16_SINT,
        (C2, Float16) => Format::R16G16_SFLOAT,
        (C2, Uint32) => Format::R32G32_UINT,
        (C2, Int32) => Format::R32G32_SINT,
        (C2, Float32) => Format::R32G32_SFLOAT,
        (C2, Uint64) => Format::R64G64_UINT,
        (C2, Int64) => Format::R64G64_SINT,
        (C2, Float64) => Format::R64G64_SFLOAT,

        (C3, Uint8) => Format::R8G8B8_UINT,
        (C3, Int8) => Format::R8G8B8_SINT,
        (C3, Uint16) => Format::R16G16B16_UINT,
        (C3, Int16) => Format::R16G16B16_SINT,
        (C3, Float16) => Format::R16G16B16_SFLOAT,
        (C3, Uint32) => Format::R32G32B32_UINT,
        (C3, Int32) => Format::R32G32B32_SINT,
        (C3, Float32) => Format::R32G32B32_SFLOAT,
        (C3, Uint64) => Format::R64G64B64_UINT,
        (C3, Int64) => Format::R64G64B64_SINT,
        (C3, Float64) => Format::R64G64B64_SFLOAT,

        (C4, Uint8) => Format::R8G8B8A8_UINT,
        (C4, Int8) => Format::R8G8B8A8_SINT,
        (C4, Uint16) => Format::R16G16B16A16_UINT,
        (C4, Int16) => Format::R16G16B16A16_SINT,
        (C4, Float16) => Format::R16G16B16A16_SFLOAT,
        (C4, Uint32) => Format::R32G32B32A32_UINT,
        (C4, Int32) => Format::R32G32B32A32_SINT,
        (C4, Float32) => Format::R32G32B32A32_SFLOAT,
        (C4, Uint64) => Format::R64G64B64A64_UINT,
        (C4, Int64) => Format::R64G64B64A64_SINT,
        (C4, Float64) => Format::R64G64B64A64_SFLOAT,
    }
}

// ---------------------------------------------------------------------------
// ImageDescriptor
// ---------------------------------------------------------------------------

/// Descriptor for creating images, mirroring C++ `ll::ImageDescriptor`.
#[derive(Debug, Clone, bon::Builder)]
pub struct ImageDescriptor {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub channel_type: ChannelType,
    pub channel_count: ChannelCount,
    pub usage: ImageUsage,
}

impl Default for ImageDescriptor {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            depth: 1,
            channel_type: ChannelType::Uint8,
            channel_count: ChannelCount::C1,
            usage: ImageUsage::STORAGE | ImageUsage::SAMPLED | ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST,
        }
    }
}

impl ImageDescriptor {
    /// Vulkan format derived from channel count and type.
    pub fn format(&self) -> Format {
        vulkan_image_format(self.channel_count, self.channel_type)
    }

    /// Vulkan image type derived from dimensions.
    pub fn image_type(&self) -> ImageType {
        if self.height == 1 {
            ImageType::Dim1d
        } else if self.depth == 1 {
            ImageType::Dim2d
        } else {
            ImageType::Dim3d
        }
    }

    /// Size in bytes (without padding).
    pub fn size_bytes(&self) -> u64 {
        self.width as u64
            * self.height as u64
            * self.depth as u64
            * self.channel_type.size_bytes()
            * self.channel_count as u64
    }

    fn validate(&self) -> Result<(), ImageError> {
        if self.width == 0 || self.height == 0 || self.depth == 0 {
            return Err(ImageError::InvalidDescriptor(
                "width, height, and depth must be > 0".into(),
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Image
// ---------------------------------------------------------------------------

/// A GPU image backed by `vulkano::image::Image`.
pub struct Image {
    inner: Arc<VkImage>,
    descriptor: ImageDescriptor,
}

impl Image {
    /// Creates a new device-local image from the given descriptor.
    pub fn new(allocator: Arc<StandardMemoryAllocator>, descriptor: ImageDescriptor) -> Result<Self, ImageError> {
        descriptor.validate()?;

        let image_type = descriptor.image_type();
        let extent = match image_type {
            ImageType::Dim1d => [descriptor.width, 1, 1],
            ImageType::Dim2d => [descriptor.width, descriptor.height, 1],
            ImageType::Dim3d => [descriptor.width, descriptor.height, descriptor.depth],
            _ => [descriptor.width, descriptor.height, descriptor.depth],
        };

        let create_info = ImageCreateInfo {
            image_type,
            format: descriptor.format(),
            extent,
            usage: descriptor.usage,
            ..Default::default()
        };

        let alloc_info = AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        };

        let inner =
            VkImage::new(allocator, create_info, alloc_info).map_err(|e| ImageError::CreationFailed(e.to_string()))?;

        Ok(Self { inner, descriptor })
    }

    /// Returns a reference to the underlying `vulkano::image::Image`.
    pub fn inner(&self) -> &Arc<VkImage> {
        &self.inner
    }

    /// Returns the image descriptor.
    pub fn descriptor(&self) -> &ImageDescriptor {
        &self.descriptor
    }

    /// Creates an [`ImageView`] from this image using the given view descriptor.
    pub fn create_image_view(
        self: &Arc<Self>,
        view_descriptor: &ImageViewDescriptor,
    ) -> Result<Arc<ImageView>, ImageError> {
        ImageView::new(self.clone(), view_descriptor).map(Arc::new)
    }
}

// ---------------------------------------------------------------------------
// ImageViewDescriptor
// ---------------------------------------------------------------------------

/// Address mode when sampling outside image boundaries.
/// Mirrors C++ `ll::ImageAddressMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageAddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
    MirrorClampToEdge,
}

impl TryFrom<u32> for ImageAddressMode {
    type Error = ImageError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Repeat),
            1 => Ok(Self::MirroredRepeat),
            2 => Ok(Self::ClampToEdge),
            3 => Ok(Self::ClampToBorder),
            4 => Ok(Self::MirrorClampToEdge),
            _ => Err(ImageError::InvalidDescriptor(format!(
                "Invalid ImageAddressMode value: {value}"
            ))),
        }
    }
}

impl From<ImageAddressMode> for SamplerAddressMode {
    fn from(m: ImageAddressMode) -> Self {
        match m {
            ImageAddressMode::Repeat => SamplerAddressMode::Repeat,
            ImageAddressMode::MirroredRepeat => SamplerAddressMode::MirroredRepeat,
            ImageAddressMode::ClampToEdge => SamplerAddressMode::ClampToEdge,
            ImageAddressMode::ClampToBorder => SamplerAddressMode::ClampToBorder,
            ImageAddressMode::MirrorClampToEdge => SamplerAddressMode::MirrorClampToEdge,
        }
    }
}

/// Filter mode for sampling.  Mirrors C++ `ll::ImageFilterMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFilterMode {
    Nearest,
    Linear,
}

impl TryFrom<u32> for ImageFilterMode {
    type Error = ImageError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Nearest),
            1 => Ok(Self::Linear),
            _ => Err(ImageError::InvalidDescriptor(format!(
                "Invalid ImageFilterMode value: {value}"
            ))),
        }
    }
}

impl From<ImageFilterMode> for Filter {
    fn from(f: ImageFilterMode) -> Self {
        match f {
            ImageFilterMode::Nearest => Filter::Nearest,
            ImageFilterMode::Linear => Filter::Linear,
        }
    }
}

/// Descriptor for `ImageView` creation, mirroring C++ `ll::ImageViewDescriptor`.
#[derive(Debug, Clone, bon::Builder)]
pub struct ImageViewDescriptor {
    #[builder(field = ImageAddressMode::Repeat)]
    pub address_mode_u: ImageAddressMode,

    #[builder(field = ImageAddressMode::Repeat)]
    pub address_mode_v: ImageAddressMode,

    #[builder(field = ImageAddressMode::Repeat)]
    pub address_mode_w: ImageAddressMode,

    #[builder(default = ImageFilterMode::Nearest)]
    pub filter_mode: ImageFilterMode,

    #[builder(default)]
    pub normalized_coordinates: bool,

    #[builder(default)]
    pub is_sampled: bool,
}

impl Default for ImageViewDescriptor {
    fn default() -> Self {
        Self {
            filter_mode: ImageFilterMode::Nearest,
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            address_mode_w: ImageAddressMode::Repeat,
            normalized_coordinates: false,
            is_sampled: false,
        }
    }
}

impl<State: image_view_descriptor_builder::State> ImageViewDescriptorBuilder<State> {
    pub fn address_mode_u(mut self, mode: ImageAddressMode) -> Self {
        self.address_mode_u = mode;
        self
    }

    pub fn address_mode_v(mut self, mode: ImageAddressMode) -> Self {
        self.address_mode_v = mode;
        self
    }

    pub fn address_mode_w(mut self, mode: ImageAddressMode) -> Self {
        self.address_mode_w = mode;
        self
    }

    pub fn address_mode(mut self, mode: ImageAddressMode) -> Self {
        self.address_mode_u = mode;
        self.address_mode_v = mode;
        self.address_mode_w = mode;
        self
    }
}

// ---------------------------------------------------------------------------
// ImageView
// ---------------------------------------------------------------------------

/// A view into an [`Image`], optionally with an associated sampler.
/// Mirrors C++ `ll::ImageView`.
pub struct ImageView {
    view: Arc<VkImageView>,
    sampler: Option<Arc<Sampler>>,
    image: Arc<Image>,
    descriptor: ImageViewDescriptor,
}

impl ImageView {
    /// Creates a new `ImageView` from an [`Image`] and descriptor.
    pub fn new(image: Arc<Image>, view_descriptor: &ImageViewDescriptor) -> Result<Self, ImageError> {
        let view_info = ImageViewCreateInfo::from_image(image.inner());

        let view = VkImageView::new(image.inner().clone(), view_info)
            .map_err(|e| ImageError::ViewCreationFailed(e.to_string()))?;

        let sampler = if view_descriptor.is_sampled {
            let device = image.inner().device().clone();
            let sampler_info = SamplerCreateInfo {
                mag_filter: view_descriptor.filter_mode.into(),
                min_filter: view_descriptor.filter_mode.into(),
                address_mode: [
                    view_descriptor.address_mode_u.into(),
                    view_descriptor.address_mode_v.into(),
                    view_descriptor.address_mode_w.into(),
                ],
                unnormalized_coordinates: !view_descriptor.normalized_coordinates,
                ..Default::default()
            };
            Some(Sampler::new(device, sampler_info).map_err(|e| ImageError::ViewCreationFailed(e.to_string()))?)
        } else {
            None
        };

        Ok(Self {
            view,
            sampler,
            image,
            descriptor: view_descriptor.clone(),
        })
    }

    /// Returns the underlying `vulkano::image::view::ImageView`.
    pub fn view(&self) -> &Arc<VkImageView> {
        &self.view
    }

    /// Returns the sampler (if this is a sampled image view).
    pub fn sampler(&self) -> Option<&Arc<Sampler>> {
        self.sampler.as_ref()
    }

    /// Returns the parent image.
    pub fn image(&self) -> &Arc<Image> {
        &self.image
    }

    /// Returns the image view descriptor.
    pub fn descriptor(&self) -> &ImageViewDescriptor {
        &self.descriptor
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::session::{Session, SessionDescriptor};
    use anyhow::Result;

    #[test]
    fn test_image_creation() -> Result<()> {
        let session = Session::new(SessionDescriptor::builder().build())?;
        let desc = ImageDescriptor::builder()
            .width(64)
            .height(64)
            .depth(1)
            .channel_type(ChannelType::Float32)
            .channel_count(ChannelCount::C4)
            .usage(ImageUsage::STORAGE | ImageUsage::SAMPLED | ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST)
            .build();

        let image = session.create_image(desc)?;
        assert_eq!(image.descriptor().width, 64);
        assert_eq!(image.descriptor().height, 64);
        Ok(())
    }

    #[test]
    fn test_image_view_creation() -> Result<()> {
        let session = Session::new(SessionDescriptor::builder().build())?;
        let desc = ImageDescriptor::builder()
            .width(32)
            .height(32)
            .depth(1)
            .channel_type(ChannelType::Uint8)
            .channel_count(ChannelCount::C4)
            .usage(ImageUsage::STORAGE | ImageUsage::SAMPLED | ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST)
            .build();

        let image = session.create_image(desc)?;
        let view_desc = ImageViewDescriptor::default();
        let _view = image.create_image_view(&view_desc)?;
        Ok(())
    }
}
