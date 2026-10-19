use anyhow::Result;
use lluvia_vk::{self as ll, math, node::Argument};
use std::collections::HashMap;

#[test]
fn test_rgba2bgra() -> Result<()> {
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

    let img_bgra_out = session.create_image(img_in_desc)?;
    let view_bgra_out = img_bgra_out.create_image_view(&view_desc)?;

    let builder_rgba2bgra = session.load_compute_node_builder("lluvia/color/RGBA2BGRA")?;
    let mut node_rgba2bgra = builder_rgba2bgra
        .build_descriptor(args)?
        .bind("in_rgba", ll::node::NodePort::ImageView(view_in))?
        .bind("out_bgra", ll::node::NodePort::ImageView(view_bgra_out))?
        .build()?;

    let staging_bgra_out = session.create_buffer_host_visible(img_in_size)?;

    // Run RGBA2BGRA
    let mut builder_cb = session.create_command_buffer_builder()?;
    builder_cb.copy_buffer_to_image(staging_in, img_in)?;
    builder_cb.record_compute_node(&mut node_rgba2bgra)?;
    builder_cb.copy_image_to_buffer(img_bgra_out, staging_bgra_out.clone())?;
    let command_buffer = builder_cb.build_command_buffer()?;
    session.run(command_buffer)?;

    let bgra_out_data = staging_bgra_out.read();
    // Verify swapping red and blue channels
    let original_rgba = rgba_img.as_raw();
    for i in 0..(width * height) as usize {
        assert_eq!(bgra_out_data[4 * i], original_rgba[4 * i + 2]); // B = R
        assert_eq!(bgra_out_data[4 * i + 1], original_rgba[4 * i + 1]); // G = G
        assert_eq!(bgra_out_data[4 * i + 2], original_rgba[4 * i]); // R = B
        assert_eq!(bgra_out_data[4 * i + 3], original_rgba[4 * i + 3]); // A = A
    }

    Ok(())
}
