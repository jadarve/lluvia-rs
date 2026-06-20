use anyhow::Result;
use lluvia_vk::{self as ll, math, node::Argument};
use std::collections::HashMap;

#[test]
fn test_bgra2gray() -> Result<()> {
    let session = ll::Session::new(ll::SessionDescriptor::default())?;

    // Load reference input image using image crate
    let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/koala.jpg");
    let img = image::open(&input_path)?;
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();

    let mut bgra_raw = rgba_img.clone().into_raw();
    for chunk in bgra_raw.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }
    let img_in_size = (width * height * 4) as u64;
    let staging_bgra_in = session.create_buffer_host_visible(img_in_size)?;
    staging_bgra_in.write(&bgra_raw);

    // Create GPU image for input BGRA
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
    let img_bgra_in = session.create_image(img_in_desc)?;
    let view_desc = ll::image::ImageViewDescriptor::default();
    let view_bgra_in = img_bgra_in.create_image_view(&view_desc)?;

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
    let img_gray_bgra = session.create_image(img_gray_desc)?;
    let view_gray_bgra = img_gray_bgra.create_image_view(&view_desc)?;

    // Arguments for resolution
    let args: HashMap<String, Argument> =
        HashMap::from([("resolution".to_string(), math::UVec2::new(width, height).into())]);

    let builder_bgra2gray = session.load_compute_node_builder("lluvia/color/BGRA2Gray")?;
    let mut node_bgra2gray = builder_bgra2gray
        .build_descriptor(args)?
        .bind("in_bgra", ll::node::NodePort::ImageView(view_bgra_in))?
        .bind("out_gray", ll::node::NodePort::ImageView(view_gray_bgra))?
        .build()?;

    let staging_gray_bgra = session.create_buffer_host_visible((width * height) as u64)?;

    // Run BGRA2Gray
    let mut builder_cb = session.create_command_buffer_builder()?;
    builder_cb.copy_buffer_to_image(staging_bgra_in, img_bgra_in)?;
    builder_cb.record_compute_node(&mut node_bgra2gray)?;
    builder_cb.copy_image_to_buffer(img_gray_bgra, staging_gray_bgra.clone())?;
    let command_buffer = builder_cb.build_command_buffer()?;
    session.run(command_buffer)?;

    let gray_data_bgra2gray = staging_gray_bgra.read();

    // Verify it yields expected values
    let original_rgba = rgba_img.as_raw();
    for i in 0..(width * height) as usize {
        let r = original_rgba[4 * i] as f32;
        let g = original_rgba[4 * i + 1] as f32;
        let b = original_rgba[4 * i + 2] as f32;
        let expected_gray = (0.299_f32 * r + 0.587_f32 * g + 0.114_f32 * b) as u8;
        let diff = (gray_data_bgra2gray[i] as i32 - expected_gray as i32).abs();
        assert!(
            diff <= 1,
            "Grayscale mismatch at pixel {}: got {}, expected {}, diff {}",
            i,
            gray_data_bgra2gray[i],
            expected_gray,
            diff
        );
    }

    Ok(())
}
