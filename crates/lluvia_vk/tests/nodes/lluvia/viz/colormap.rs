use anyhow::Result;
use lluvia_vk::{self as ll, math, node::Argument};
use rstest::rstest;
use std::collections::HashMap;

#[test]
fn test_colormap_float() -> Result<()> {
    let session = ll::Session::new(ll::SessionDescriptor::default())?;

    let width = 64;
    let height = 64;

    // Create synthetic scalar float data (width * height * 4 bytes)
    let in_size = (width * height * 4) as u64;
    let staging_in = session.create_buffer_host_visible(in_size)?;
    {
        let mut data = vec![0.0f32; (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                data[idx] = x as f32 / width as f32;
            }
        }
        staging_in.write(bytemuck::cast_slice(&data));
    }

    let img_in_desc = ll::image::ImageDescriptor::builder()
        .width(width)
        .height(height)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Float32)
        .usage(
            vulkano::image::ImageUsage::STORAGE
                | vulkano::image::ImageUsage::TRANSFER_DST
                | vulkano::image::ImageUsage::TRANSFER_SRC,
        )
        .build();
    let img_in = session.create_image(img_in_desc)?;
    let view_desc = ll::image::ImageViewDescriptor::default();
    let view_in = img_in.create_image_view(&view_desc)?;

    let args: HashMap<String, Argument> = HashMap::from([
        ("colormap".to_string(), Argument::String("viridis".to_string())),
        ("min_value".to_string(), 0.0f32.into()),
        ("max_value".to_string(), 1.0f32.into()),
    ]);

    session.load_compute_node_builder("lluvia/viz/colormap/ColorMap_float")?;
    let builder_colormap = session.load_container_node_builder("lluvia/viz/colormap/ColorMap")?;

    let img_rgba_desc = ll::image::ImageDescriptor::builder()
        .width(width)
        .height(height)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C4)
        .channel_type(ll::image::ChannelType::Uint8)
        .usage(
            vulkano::image::ImageUsage::STORAGE
                | vulkano::image::ImageUsage::TRANSFER_DST
                | vulkano::image::ImageUsage::TRANSFER_SRC,
        )
        .build();
    let img_rgba = session.create_image(img_rgba_desc)?;
    let view_rgba = img_rgba.create_image_view(&view_desc)?;

    let mut node_colormap = builder_colormap
        .build_descriptor(args)?
        .bind("in_image", ll::node::NodePort::ImageView(view_in))?
        .bind("out_rgba", ll::node::NodePort::ImageView(view_rgba))?
        .build()?;

    // Since ColorMap is a container node, it executes internally.
    let staging_rgba = session.create_buffer_host_visible((width * height * 4) as u64)?;

    let mut builder_cb = session.create_command_buffer_builder()?;
    builder_cb.copy_buffer_to_image(staging_in, img_in)?;
    builder_cb.record_container_node(&mut node_colormap)?;
    builder_cb.copy_image_to_buffer(img_rgba, staging_rgba.clone())?;
    let command_buffer = builder_cb.build_command_buffer()?;
    session.run(command_buffer)?;

    let rgba_data: Vec<u8> = staging_rgba.read();
    assert_eq!(rgba_data.len(), (width * height * 4) as usize);

    Ok(())
}

#[rstest]
fn test_colormap_from_image(
    #[values(
        "viridis", "plasma", "inferno", "magma", "cividis", "gray", "purples", "blues", "greens", "oranges", "reds",
        "spectral", "coolwarm", "bwr", "seismic", "twilight", "hsv"
    )]
    colormap_name: &str,
    #[values(false, true)] reverse: bool,
) -> Result<()> {
    let session = ll::Session::new(ll::SessionDescriptor::default())?;

    // Load reference input image using image crate
    let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/mouse.jpg");
    let img = image::open(&input_path)?;
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();

    // Create staging buffer for input RGBA image
    let img_in_size = (width * height * 4) as u64;
    let staging_in = session.create_buffer_host_visible(img_in_size)?;
    staging_in.write(rgba_img.as_raw());

    // Create GPU image for input RGBA
    let img_in_desc = ll::image::ImageDescriptor::builder()
        .width(width)
        .height(height)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C4)
        .channel_type(ll::image::ChannelType::Uint8)
        .usage(
            vulkano::image::ImageUsage::STORAGE
                | vulkano::image::ImageUsage::TRANSFER_DST
                | vulkano::image::ImageUsage::TRANSFER_SRC,
        )
        .build();
    let img_in = session.create_image(img_in_desc)?;
    let view_desc = ll::image::ImageViewDescriptor::default();
    let view_in = img_in.create_image_view(&view_desc)?;

    // Create GPU image for Uint8 gray
    let img_gray_u8_desc = ll::image::ImageDescriptor::builder()
        .width(width)
        .height(height)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Uint8)
        .usage(
            vulkano::image::ImageUsage::STORAGE
                | vulkano::image::ImageUsage::TRANSFER_DST
                | vulkano::image::ImageUsage::TRANSFER_SRC,
        )
        .build();
    let img_gray_u8 = session.create_image(img_gray_u8_desc)?;
    let view_gray_u8 = img_gray_u8.create_image_view(&view_desc)?;

    // Create GPU image for Float32 gray
    let img_gray_f32_desc = ll::image::ImageDescriptor::builder()
        .width(width)
        .height(height)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Float32)
        .usage(
            vulkano::image::ImageUsage::STORAGE
                | vulkano::image::ImageUsage::TRANSFER_DST
                | vulkano::image::ImageUsage::TRANSFER_SRC,
        )
        .build();
    let img_gray_f32 = session.create_image(img_gray_f32_desc)?;
    let view_gray_f32 = img_gray_f32.create_image_view(&view_desc)?;

    // 1. RGBA2Gray node
    let args_resolution: HashMap<String, Argument> =
        HashMap::from([("resolution".to_string(), math::UVec2::new(width, height).into())]);

    let builder_rgba2gray = session.load_compute_node_builder("lluvia/color/RGBA2Gray")?;

    let mut node_rgba2gray = builder_rgba2gray
        .build_descriptor(args_resolution.clone())?
        .bind("in_rgba", ll::node::NodePort::ImageView(view_in))?
        .bind("out_gray", ll::node::NodePort::ImageView(view_gray_u8.clone()))?
        .build()?;

    // 2. ImageNormalizer node (convert u8 gray [0, 255] to float32 gray [0, 255])
    let builder_normalizer = session.load_compute_node_builder("lluvia/math/normalize/ImageNormalizer_r8ui_r32f")?;

    let mut node_normalizer = builder_normalizer
        .build_descriptor(args_resolution)?
        .set_constant("max_value", ll::node::Constant::Float(255.0))
        .bind("in_gray", ll::node::NodePort::ImageView(view_gray_u8))?
        .bind("out_gray", ll::node::NodePort::ImageView(view_gray_f32.clone()))?
        .build()?;

    // 3. Colormap node
    session.load_compute_node_builder("lluvia/viz/colormap/ColorMap_float")?;
    let builder_colormap = session.load_container_node_builder("lluvia/viz/colormap/ColorMap")?;

    let img_rgba_out_desc = ll::image::ImageDescriptor::builder()
        .width(width)
        .height(height)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C4)
        .channel_type(ll::image::ChannelType::Uint8)
        .usage(
            vulkano::image::ImageUsage::STORAGE
                | vulkano::image::ImageUsage::TRANSFER_DST
                | vulkano::image::ImageUsage::TRANSFER_SRC,
        )
        .build();
    let img_rgba_out = session.create_image(img_rgba_out_desc)?;
    let view_rgba_out = img_rgba_out.create_image_view(&view_desc)?;

    let reverse_val = if reverse { 1.0f32 } else { 0.0f32 };
    let args_colormap: HashMap<String, Argument> = HashMap::from([
        ("colormap".to_string(), Argument::String(colormap_name.to_string())),
        ("min_value".to_string(), 0.0f32.into()),
        ("max_value".to_string(), 1.0f32.into()),
        ("reverse".to_string(), reverse_val.into()),
    ]);

    let mut node_colormap = builder_colormap
        .build_descriptor(args_colormap)?
        .bind("in_image", ll::node::NodePort::ImageView(view_gray_f32))?
        .bind("out_rgba", ll::node::NodePort::ImageView(view_rgba_out))?
        .build()?;

    let staging_out = session.create_buffer_host_visible((width * height * 4) as u64)?;

    let mut builder_cb = session.create_command_buffer_builder()?;
    builder_cb.copy_buffer_to_image(staging_in, img_in)?;
    builder_cb.record_compute_node(&mut node_rgba2gray)?;
    builder_cb.record_compute_node(&mut node_normalizer)?;
    builder_cb.record_container_node(&mut node_colormap)?;
    builder_cb.copy_image_to_buffer(img_rgba_out, staging_out.clone())?;
    let command_buffer = builder_cb.build_command_buffer()?;
    session.run(command_buffer)?;

    let rgba_data: Vec<u8> = staging_out.read();
    assert_eq!(rgba_data.len(), (width * height * 4) as usize);

    let out_image = image::RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| anyhow::anyhow!("Failed to construct output RgbaImage"))?;
    let rgb_image = image::DynamicImage::ImageRgba8(out_image).into_rgb8();

    let output_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/results/colormap");
    std::fs::create_dir_all(&output_dir)?;
    let file_suffix = if reverse { "_r" } else { "" };
    let output_path = output_dir.join(format!("mouse_{}{}.jpg", colormap_name, file_suffix));
    rgb_image.save(&output_path)?;
    assert!(output_path.exists());

    Ok(())
}
