use anyhow::Result;
use lluvia_vk::{self as ll, math, node::Argument};
use std::collections::HashMap;

#[test]
fn test_hsva2rgba() -> Result<()> {
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
    let img_in = session.create_image(img_in_desc.clone())?;
    let view_desc = ll::image::ImageViewDescriptor::default();
    let view_in = img_in.create_image_view(&view_desc)?;

    // Arguments for resolution
    let args: HashMap<String, Argument> =
        HashMap::from([("resolution".to_string(), math::UVec2::new(width, height).into())]);

    // Create intermediate HSVA image
    let img_hsva_desc = ll::image::ImageDescriptor::builder()
        .width(width)
        .height(height)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C4)
        .channel_type(ll::image::ChannelType::Float32)
        .usage(
            vulkano::image::ImageUsage::STORAGE
                | vulkano::image::ImageUsage::TRANSFER_DST
                | vulkano::image::ImageUsage::TRANSFER_SRC,
        )
        .build();
    let img_hsva = session.create_image(img_hsva_desc)?;
    let view_hsva = img_hsva.create_image_view(&view_desc)?;

    let builder_rgba2hsva = session.load_compute_node_builder("lluvia/color/RGBA2HSVA")?;
    let mut node_rgba2hsva = builder_rgba2hsva
        .build_descriptor(args.clone())?
        .bind("in_rgba", ll::node::NodePort::ImageView(view_in.clone()))?
        .bind("out_hsva", ll::node::NodePort::ImageView(view_hsva.clone()))?
        .set_constant("min_chroma", ll::node::Constant::Float(0.0))
        .build()?;

    // Create target output reconstructed image
    let img_rgba_reconstructed = session.create_image(img_in_desc)?;
    let view_rgba_reconstructed = img_rgba_reconstructed.create_image_view(&view_desc)?;

    let builder_hsva2rgba = session.load_compute_node_builder("lluvia/color/HSVA2RGBA")?;
    let mut node_hsva2rgba = builder_hsva2rgba
        .build_descriptor(args)?
        .bind("in_hsva", ll::node::NodePort::ImageView(view_hsva))?
        .bind("out_rgba", ll::node::NodePort::ImageView(view_rgba_reconstructed))?
        .build()?;

    let staging_rgba_reconstructed = session.create_buffer_host_visible(img_in_size)?;

    // Run pipeline
    let mut builder_cb = session.create_command_buffer_builder()?;
    builder_cb.copy_buffer_to_image(staging_in, img_in)?;
    builder_cb.record_compute_node(&mut node_rgba2hsva)?;
    builder_cb.record_compute_node(&mut node_hsva2rgba)?;
    builder_cb.copy_image_to_buffer(img_rgba_reconstructed, staging_rgba_reconstructed.clone())?;
    let command_buffer = builder_cb.build_command_buffer()?;
    session.run(command_buffer)?;

    let reconstructed_rgba = staging_rgba_reconstructed.read();
    let original_rgba = rgba_img.as_raw();

    // Assert that round-trip outputs are extremely close to the original inputs
    // HSV/RGB conversions can suffer from minor rounding errors, so allow a tolerance of 1
    for i in 0..(width * height * 4) as usize {
        let diff = (original_rgba[i] as i32 - reconstructed_rgba[i] as i32).abs();
        assert!(
            diff <= 1,
            "Pixel channel mismatch at index {}: original {}, reconstructed {}, diff {}",
            i,
            original_rgba[i],
            reconstructed_rgba[i],
            diff
        );
    }

    Ok(())
}
