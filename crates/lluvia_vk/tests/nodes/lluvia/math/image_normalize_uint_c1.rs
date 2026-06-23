use anyhow::Result;
use lluvia_vk::{
    self as ll, math,
    node::{Argument, Constant},
};
use std::collections::HashMap;

#[test]
fn test_image_normalize_uint_c1_normalized() -> Result<()> {
    let session = ll::Session::new(ll::SessionDescriptor::default())?;

    // Load reference input image using image crate
    let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/mouse.jpg");
    let img = image::open(&input_path)?;
    let luma_img = img.to_luma8();
    let (width, height) = luma_img.dimensions();

    // Convert input to u32
    let original = luma_img.as_raw();
    let u32_raw: Vec<u32> = original.iter().map(|&x| x as u32).collect();

    // Create staging buffer for input image
    let img_in_size = (width * height * 4) as u64;
    let staging_in = session.create_buffer_host_visible(img_in_size)?;
    staging_in.write(bytemuck::cast_slice(&u32_raw));

    // Create GPU image for input image (Uint32, C1)
    let img_in_desc = ll::image::ImageDescriptor::builder()
        .width(width)
        .height(height)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Uint32)
        .usage(
            vulkano::image::ImageUsage::STORAGE
                | vulkano::image::ImageUsage::TRANSFER_DST
                | vulkano::image::ImageUsage::TRANSFER_SRC,
        )
        .build();
    let img_in = session.create_image(img_in_desc)?;
    let view_desc = ll::image::ImageViewDescriptor::default();
    let view_in = img_in.create_image_view(&view_desc)?;

    // Create GPU image for output image (Float32, C1)
    let img_out_desc = ll::image::ImageDescriptor::builder()
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
    let img_out = session.create_image(img_out_desc)?;
    let view_out = img_out.create_image_view(&view_desc)?;

    // Arguments for resolution
    let args: HashMap<String, Argument> =
        HashMap::from([("resolution".to_string(), math::UVec2::new(width, height).into())]);

    let builder = session.load_compute_node_builder("lluvia/math/normalize/ImageNormalize_uint_C1")?;
    let mut node = builder
        .build_descriptor(args)?
        .set_constant("max_value", Constant::Float(255.0))
        .bind("in_image_uint", ll::node::NodePort::ImageView(view_in))?
        .bind("out_image_float", ll::node::NodePort::ImageView(view_out))?
        .build()?;

    let staging_out = session.create_buffer_host_visible((width * height * 4) as u64)?;

    // Run ImageNormalize
    let mut builder_cb = session.create_command_buffer_builder()?;
    builder_cb.copy_buffer_to_image(staging_in, img_in)?;
    builder_cb.record_compute_node(&mut node)?;
    builder_cb.copy_image_to_buffer(img_out, staging_out.clone())?;
    let command_buffer = builder_cb.build_command_buffer()?;
    session.run(command_buffer)?;

    let out_data_bytes = staging_out.read();
    let out_data: &[f32] = bytemuck::cast_slice(&out_data_bytes);

    // Verify values: divided by 255.0
    for i in 0..(width * height) as usize {
        let expected = original[i] as f32 / 255.0;
        let diff = (out_data[i] - expected).abs();
        assert!(
            diff < 1e-5,
            "Normalize mismatch at pixel {}: got {}, expected {}, diff {}",
            i,
            out_data[i],
            expected,
            diff
        );
    }

    Ok(())
}

#[test]
fn test_image_normalize_uint_c1_passthrough() -> Result<()> {
    let session = ll::Session::new(ll::SessionDescriptor::default())?;

    // Load reference input image using image crate
    let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/mouse.jpg");
    let img = image::open(&input_path)?;
    let luma_img = img.to_luma8();
    let (width, height) = luma_img.dimensions();

    // Convert input to u32
    let original = luma_img.as_raw();
    let u32_raw: Vec<u32> = original.iter().map(|&x| x as u32).collect();

    // Create staging buffer for input image
    let img_in_size = (width * height * 4) as u64;
    let staging_in = session.create_buffer_host_visible(img_in_size)?;
    staging_in.write(bytemuck::cast_slice(&u32_raw));

    // Create GPU image for input image (Uint32, C1)
    let img_in_desc = ll::image::ImageDescriptor::builder()
        .width(width)
        .height(height)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Uint32)
        .usage(
            vulkano::image::ImageUsage::STORAGE
                | vulkano::image::ImageUsage::TRANSFER_DST
                | vulkano::image::ImageUsage::TRANSFER_SRC,
        )
        .build();
    let img_in = session.create_image(img_in_desc)?;
    let view_desc = ll::image::ImageViewDescriptor::default();
    let view_in = img_in.create_image_view(&view_desc)?;

    // Create GPU image for output image (Float32, C1)
    let img_out_desc = ll::image::ImageDescriptor::builder()
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
    let img_out = session.create_image(img_out_desc)?;
    let view_out = img_out.create_image_view(&view_desc)?;

    // Arguments for resolution
    let args: HashMap<String, Argument> =
        HashMap::from([("resolution".to_string(), math::UVec2::new(width, height).into())]);

    let builder = session.load_compute_node_builder("lluvia/math/normalize/ImageNormalize_uint_C1")?;
    let mut node = builder
        .build_descriptor(args)?
        .set_constant("max_value", Constant::Float(0.0))
        .bind("in_image_uint", ll::node::NodePort::ImageView(view_in))?
        .bind("out_image_float", ll::node::NodePort::ImageView(view_out))?
        .build()?;

    let staging_out = session.create_buffer_host_visible((width * height * 4) as u64)?;

    // Run ImageNormalize
    let mut builder_cb = session.create_command_buffer_builder()?;
    builder_cb.copy_buffer_to_image(staging_in, img_in)?;
    builder_cb.record_compute_node(&mut node)?;
    builder_cb.copy_image_to_buffer(img_out, staging_out.clone())?;
    let command_buffer = builder_cb.build_command_buffer()?;
    session.run(command_buffer)?;

    let out_data_bytes = staging_out.read();
    let out_data: &[f32] = bytemuck::cast_slice(&out_data_bytes);

    // Verify values: should be equal to the input casted to float directly (max_value <= 0)
    for i in 0..(width * height) as usize {
        let expected = original[i] as f32;
        let diff = (out_data[i] - expected).abs();
        assert!(
            diff < 1e-5,
            "Normalize mismatch at pixel {}: got {}, expected {}, diff {}",
            i,
            out_data[i],
            expected,
            diff
        );
    }

    Ok(())
}
