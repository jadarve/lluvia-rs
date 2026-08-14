use anyhow::Result;
use lluvia_vk::{self as ll, math, node::Argument};
use std::collections::HashMap;

#[test]
fn test_rgba2gray() -> Result<()> {
    let session = ll::Session::new(ll::SessionDescriptor::default())?;

    // Load reference input image using image crate
    let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/koala.jpg");
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

    // Arguments for resolution
    let args: HashMap<String, Argument> =
        HashMap::from([("resolution".to_string(), math::UVec2::new(width, height).into())]);

    let builder_rgba2gray = session.load_compute_node_builder("lluvia/color/RGBA2Gray")?;

    let img_gray_desc = ll::image::ImageDescriptor::builder()
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
    let img_gray = session.create_image(img_gray_desc)?;
    let view_gray = img_gray.create_image_view(&view_desc)?;

    let mut node_rgba2gray = builder_rgba2gray
        .build_descriptor(args)?
        .bind("in_rgba", ll::node::NodePort::ImageView(view_in))?
        .bind("out_gray", ll::node::NodePort::ImageView(view_gray))?
        .build()?;

    let staging_gray = session.create_buffer_host_visible((width * height) as u64)?;

    // Run RGBA2Gray
    let mut builder_cb = session.create_command_buffer_builder()?;
    builder_cb.copy_buffer_to_image(staging_in, img_in)?;
    builder_cb.record_compute_node(&mut node_rgba2gray)?;
    builder_cb.copy_image_to_buffer(img_gray, staging_gray.clone())?;
    let command_buffer = builder_cb.build_command_buffer()?;
    session.run(command_buffer)?;

    let gray_data = staging_gray.read();

    // Save output gray image
    let gray_image = image::GrayImage::from_raw(width, height, gray_data)
        .ok_or_else(|| anyhow::anyhow!("Failed to construct output GrayImage"))?;
    let output_path_gray =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/output_koala_gray.jpg");
    gray_image.save(&output_path_gray)?;
    assert!(output_path_gray.exists());

    Ok(())
}
