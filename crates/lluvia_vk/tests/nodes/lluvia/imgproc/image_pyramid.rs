use anyhow::Result;
use lluvia_vk::{
    self as ll, math,
    node::{Argument, Node},
};
use std::collections::HashMap;

#[test]
fn test_image_pyramid() -> Result<()> {
    let session = ll::Session::new(ll::SessionDescriptor::default())?;

    let width = 16u32;
    let height = 16u32;
    let levels = 3;

    // Create input data
    let mut input_data = vec![0u8; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            input_data[(y * width + x) as usize] = (x * 3 + y) as u8;
        }
    }

    // Create staging buffer for input image
    let staging_in = session.create_buffer_host_visible((width * height) as u64)?;
    staging_in.write(&input_data);

    // Create GPU image for input
    let img_in_desc = ll::image::ImageDescriptor::builder()
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
    let img_in = session.create_image(img_in_desc)?;
    let view_desc = ll::image::ImageViewDescriptor::default();
    let view_in = img_in.create_image_view(&view_desc)?;

    // Output views for each level
    // Level 0: 16x16
    let img_out_0_desc = ll::image::ImageDescriptor::builder()
        .width(16)
        .height(16)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Uint8)
        .usage(ll_in_out_usage())
        .build();
    let img_out_0 = session.create_image(img_out_0_desc)?;
    let view_out_0 = img_out_0.create_image_view(&view_desc)?;

    // Level 1: 8x8
    let img_out_1_desc = ll::image::ImageDescriptor::builder()
        .width(8)
        .height(8)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Uint8)
        .usage(ll_in_out_usage())
        .build();
    let img_out_1 = session.create_image(img_out_1_desc)?;
    let view_out_1 = img_out_1.create_image_view(&view_desc)?;

    // Level 2: 4x4
    let img_out_2_desc = ll::image::ImageDescriptor::builder()
        .width(4)
        .height(4)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Uint8)
        .usage(ll_in_out_usage())
        .build();
    let img_out_2 = session.create_image(img_out_2_desc)?;
    let view_out_2 = img_out_2.create_image_view(&view_desc)?;

    // Top Level Alias (same size as level 2: 4x4)
    let img_out_desc = ll::image::ImageDescriptor::builder()
        .width(4)
        .height(4)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Uint8)
        .usage(ll_in_out_usage())
        .build();
    let img_out = session.create_image(img_out_desc)?;
    let view_out = img_out.create_image_view(&view_desc)?;

    // Arguments
    let args: HashMap<String, Argument> = HashMap::from([
        ("resolution".to_string(), math::UVec2::new(width, height).into()),
        ("levels".to_string(), Argument::I32(levels)),
    ]);

    // Load child compute node builders (required by container node)
    let _builder_down_x = session.load_compute_node_builder("lluvia/imgproc/ImageDownsampleX_r8ui")?;
    let _builder_down_y = session.load_compute_node_builder("lluvia/imgproc/ImageDownsampleY_r8ui")?;

    // Load container builder
    let builder_pyramid = session.load_container_node_builder("lluvia/imgproc/ImagePyramid_r8ui")?;

    let mut node_pyramid = builder_pyramid
        .build_descriptor(args)?
        .bind("in_gray", ll::node::NodePort::ImageView(view_in.clone()))?
        .bind("out_gray_0", ll::node::NodePort::ImageView(view_out_0.clone()))?
        .bind("out_gray_1", ll::node::NodePort::ImageView(view_out_1.clone()))?
        .bind("out_gray_2", ll::node::NodePort::ImageView(view_out_2.clone()))?
        .bind("out_gray", ll::node::NodePort::ImageView(view_out.clone()))?
        .build()?;

    // Create staging buffers for verification
    let staging_out_0 = session.create_buffer_host_visible((16 * 16) as u64)?;
    let staging_out_1 = session.create_buffer_host_visible((8 * 8) as u64)?;
    let staging_out_2 = session.create_buffer_host_visible((4 * 4) as u64)?;
    let staging_out = session.create_buffer_host_visible((4 * 4) as u64)?;

    // Record command buffer
    let mut builder_cb = session.create_command_buffer_builder()?;
    builder_cb.copy_buffer_to_image(staging_in, img_in)?;
    builder_cb.record_container_node(&mut node_pyramid)?;
    let actual_view_0 = match node_pyramid.port("out_gray_0").unwrap() {
        ll::node::NodePort::ImageView(v) => v.clone(),
        _ => panic!("Expected ImageView"),
    };
    let actual_view_1 = match node_pyramid.port("out_gray_1").unwrap() {
        ll::node::NodePort::ImageView(v) => v.clone(),
        _ => panic!("Expected ImageView"),
    };
    let actual_view_2 = match node_pyramid.port("out_gray_2").unwrap() {
        ll::node::NodePort::ImageView(v) => v.clone(),
        _ => panic!("Expected ImageView"),
    };
    let actual_view_out = match node_pyramid.port("out_gray").unwrap() {
        ll::node::NodePort::ImageView(v) => v.clone(),
        _ => panic!("Expected ImageView"),
    };

    let actual_img_out_0 = actual_view_0.image().clone();
    let actual_img_out_1 = actual_view_1.image().clone();
    let actual_img_out_2 = actual_view_2.image().clone();
    let actual_img_out = actual_view_out.image().clone();

    builder_cb.copy_image_to_buffer(actual_img_out_0, staging_out_0.clone())?;
    builder_cb.copy_image_to_buffer(actual_img_out_1, staging_out_1.clone())?;
    builder_cb.copy_image_to_buffer(actual_img_out_2, staging_out_2.clone())?;
    builder_cb.copy_image_to_buffer(actual_img_out, staging_out.clone())?;
    let command_buffer = builder_cb.build_command_buffer()?;
    session.run(command_buffer)?;

    // Verify Level 0: should be exact match
    let out_data_0 = staging_out_0.read();
    assert_eq!(&input_data[..], &out_data_0[..]);

    // Verify Level 1: downsampled from input_data (16x16 -> X -> 8x16 -> Y -> 8x8)
    let out_data_1 = staging_out_1.read();
    let expected_1 = downsample_reference(&input_data, 16, 16);
    assert_eq!(&expected_1[..], &out_data_1[..]);

    // Verify Level 2: downsampled from expected_1 (8x8 -> X -> 4x8 -> Y -> 4x4)
    let out_data_2 = staging_out_2.read();
    let expected_2 = downsample_reference(&expected_1, 8, 8);
    assert_eq!(&expected_2[..], &out_data_2[..]);

    // Verify Alias: should match Level 2
    let out_data = staging_out.read();
    assert_eq!(&expected_2[..], &out_data[..]);

    Ok(())
}

#[test]
fn test_image_pyramid_koala() -> Result<()> {
    let session = ll::Session::new(ll::SessionDescriptor::default())?;

    // Load reference input image using image crate
    let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/koala_gray.jpg");
    let img = image::open(&input_path)?;
    let gray_img = img.to_luma8();
    let (width, height) = gray_img.dimensions();
    let levels = 3;

    // Create staging buffer for input image
    let staging_in = session.create_buffer_host_visible((width * height) as u64)?;
    staging_in.write(gray_img.as_raw());

    // Create GPU image for input
    let img_in_desc = ll::image::ImageDescriptor::builder()
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
    let img_in = session.create_image(img_in_desc)?;
    let view_desc = ll::image::ImageViewDescriptor::default();
    let view_in = img_in.create_image_view(&view_desc)?;

    // Level 0: width x height
    let img_out_0_desc = ll::image::ImageDescriptor::builder()
        .width(width)
        .height(height)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Uint8)
        .usage(ll_in_out_usage())
        .build();
    let img_out_0 = session.create_image(img_out_0_desc)?;
    let view_out_0 = img_out_0.create_image_view(&view_desc)?;

    // Level 1: width/2 x height/2
    let img_out_1_desc = ll::image::ImageDescriptor::builder()
        .width(width / 2)
        .height(height / 2)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Uint8)
        .usage(ll_in_out_usage())
        .build();
    let img_out_1 = session.create_image(img_out_1_desc)?;
    let view_out_1 = img_out_1.create_image_view(&view_desc)?;

    // Level 2: width/4 x height/4
    let img_out_2_desc = ll::image::ImageDescriptor::builder()
        .width(width / 4)
        .height(height / 4)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Uint8)
        .usage(ll_in_out_usage())
        .build();
    let img_out_2 = session.create_image(img_out_2_desc)?;
    let view_out_2 = img_out_2.create_image_view(&view_desc)?;

    // Top Level Alias (width/4 x height/4)
    let img_out_desc = ll::image::ImageDescriptor::builder()
        .width(width / 4)
        .height(height / 4)
        .depth(1)
        .channel_count(ll::image::ChannelCount::C1)
        .channel_type(ll::image::ChannelType::Uint8)
        .usage(ll_in_out_usage())
        .build();
    let img_out = session.create_image(img_out_desc)?;
    let view_out = img_out.create_image_view(&view_desc)?;

    // Arguments
    let args: HashMap<String, Argument> = HashMap::from([
        ("resolution".to_string(), math::UVec2::new(width, height).into()),
        ("levels".to_string(), Argument::I32(levels)),
    ]);

    // Load compute node builders
    let _builder_down_x = session.load_compute_node_builder("lluvia/imgproc/ImageDownsampleX_r8ui")?;
    let _builder_down_y = session.load_compute_node_builder("lluvia/imgproc/ImageDownsampleY_r8ui")?;

    // Load container builder
    let builder_pyramid = session.load_container_node_builder("lluvia/imgproc/ImagePyramid_r8ui")?;

    let mut node_pyramid = builder_pyramid
        .build_descriptor(args)?
        .bind("in_gray", ll::node::NodePort::ImageView(view_in.clone()))?
        .bind("out_gray_0", ll::node::NodePort::ImageView(view_out_0.clone()))?
        .bind("out_gray_1", ll::node::NodePort::ImageView(view_out_1.clone()))?
        .bind("out_gray_2", ll::node::NodePort::ImageView(view_out_2.clone()))?
        .bind("out_gray", ll::node::NodePort::ImageView(view_out.clone()))?
        .build()?;

    // Create staging buffers for verification
    let staging_out_0 = session.create_buffer_host_visible((width * height) as u64)?;
    let staging_out_1 = session.create_buffer_host_visible(((width / 2) * (height / 2)) as u64)?;
    let staging_out_2 = session.create_buffer_host_visible(((width / 4) * (height / 4)) as u64)?;
    let staging_out = session.create_buffer_host_visible(((width / 4) * (height / 4)) as u64)?;

    // Record command buffer
    let mut builder_cb = session.create_command_buffer_builder()?;
    builder_cb.copy_buffer_to_image(staging_in, img_in)?;
    builder_cb.record_container_node(&mut node_pyramid)?;

    let actual_view_0 = match node_pyramid.port("out_gray_0").unwrap() {
        ll::node::NodePort::ImageView(v) => v.clone(),
        _ => panic!("Expected ImageView"),
    };
    let actual_view_1 = match node_pyramid.port("out_gray_1").unwrap() {
        ll::node::NodePort::ImageView(v) => v.clone(),
        _ => panic!("Expected ImageView"),
    };
    let actual_view_2 = match node_pyramid.port("out_gray_2").unwrap() {
        ll::node::NodePort::ImageView(v) => v.clone(),
        _ => panic!("Expected ImageView"),
    };
    let actual_view_out = match node_pyramid.port("out_gray").unwrap() {
        ll::node::NodePort::ImageView(v) => v.clone(),
        _ => panic!("Expected ImageView"),
    };

    let actual_img_out_0 = actual_view_0.image().clone();
    let actual_img_out_1 = actual_view_1.image().clone();
    let actual_img_out_2 = actual_view_2.image().clone();
    let actual_img_out = actual_view_out.image().clone();

    builder_cb.copy_image_to_buffer(actual_img_out_0, staging_out_0.clone())?;
    builder_cb.copy_image_to_buffer(actual_img_out_1, staging_out_1.clone())?;
    builder_cb.copy_image_to_buffer(actual_img_out_2, staging_out_2.clone())?;
    builder_cb.copy_image_to_buffer(actual_img_out, staging_out.clone())?;
    let command_buffer = builder_cb.build_command_buffer()?;
    session.run(command_buffer)?;

    // Write all intermediate images as output
    let out_data_0 = staging_out_0.read();
    let out_img_0 = image::GrayImage::from_raw(width, height, out_data_0.to_vec())
        .ok_or_else(|| anyhow::anyhow!("Failed to construct Level 0 output GrayImage"))?;
    let output_path_0 =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/output_koala_pyramid_0.jpg");
    out_img_0.save(&output_path_0)?;

    let out_data_1 = staging_out_1.read();
    let out_img_1 = image::GrayImage::from_raw(width / 2, height / 2, out_data_1.to_vec())
        .ok_or_else(|| anyhow::anyhow!("Failed to construct Level 1 output GrayImage"))?;
    let output_path_1 =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/output_koala_pyramid_1.jpg");
    out_img_1.save(&output_path_1)?;

    let out_data_2 = staging_out_2.read();
    let out_img_2 = image::GrayImage::from_raw(width / 4, height / 4, out_data_2.to_vec())
        .ok_or_else(|| anyhow::anyhow!("Failed to construct Level 2 output GrayImage"))?;
    let output_path_2 =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/output_koala_pyramid_2.jpg");
    out_img_2.save(&output_path_2)?;

    let out_data = staging_out.read();
    let out_img_alias = image::GrayImage::from_raw(width / 4, height / 4, out_data.to_vec())
        .ok_or_else(|| anyhow::anyhow!("Failed to construct Alias output GrayImage"))?;
    let output_path_alias =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/output_koala_pyramid_alias.jpg");
    out_img_alias.save(&output_path_alias)?;

    // Basic assertions that outputs are correct downsampled versions of previous ones
    let expected_1 = downsample_reference(gray_img.as_raw(), width, height);
    assert_eq!(&expected_1[..], &out_data_1[..]);

    let expected_2 = downsample_reference(&expected_1, width / 2, height / 2);
    assert_eq!(&expected_2[..], &out_data_2[..]);
    assert_eq!(&expected_2[..], &out_data[..]);

    Ok(())
}

fn ll_in_out_usage() -> vulkano::image::ImageUsage {
    vulkano::image::ImageUsage::STORAGE
        | vulkano::image::ImageUsage::TRANSFER_DST
        | vulkano::image::ImageUsage::TRANSFER_SRC
}

// Emulates X + Y downsampling logic
fn downsample_reference(input: &[u8], w: u32, h: u32) -> Vec<u8> {
    let out_w = w / 2;
    let mut x_down = vec![0u8; (out_w * h) as usize];
    // Downsample X
    for y in 0..h {
        for x in 0..out_w {
            let img_0 = input[(y * w + 2 * x) as usize] as u32;
            let img_m = if x == 0 {
                img_0
            } else {
                input[(y * w + 2 * x - 1) as usize] as u32
            };
            let img_p = if x == out_w - 1 {
                img_0
            } else {
                input[(y * w + 2 * x + 1) as usize] as u32
            };
            x_down[(y * out_w + x) as usize] = ((img_0 >> 1) + ((img_m + img_p) >> 2)) as u8;
        }
    }

    // Downsample Y
    let out_h = h / 2;
    let mut xy_down = vec![0u8; (out_w * out_h) as usize];
    for y in 0..out_h {
        for x in 0..out_w {
            let img_0 = x_down[(2 * y * out_w + x) as usize] as u32;
            let img_m = if y == 0 {
                img_0
            } else {
                x_down[((2 * y - 1) * out_w + x) as usize] as u32
            };
            let img_p = if y == out_h - 1 {
                img_0
            } else {
                x_down[((2 * y + 1) * out_w + x) as usize] as u32
            };
            xy_down[(y * out_w + x) as usize] = ((img_0 >> 1) + ((img_m + img_p) >> 2)) as u8;
        }
    }

    xy_down
}
