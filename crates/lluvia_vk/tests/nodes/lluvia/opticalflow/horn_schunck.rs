use anyhow::Result;
use lluvia_vk::{
    self as ll, math,
    node::{Argument, Node},
};
use std::collections::HashMap;

#[test]
fn test_horn_schunck_zeros() -> Result<()> {
    let session = ll::Session::new(ll::SessionDescriptor::default())?;

    let width = 64u32;
    let height = 48u32;

    // Create input gray image (zeros)
    let in_gray_desc = ll::image::ImageDescriptor::builder()
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
    let img_in = session.create_image(in_gray_desc)?;
    let view_desc = ll::image::ImageViewDescriptor::default();
    let view_in = img_in.create_image_view(&view_desc)?;

    // Populate input staging buffer with zeros
    let img_in_size = (width * height) as u64;
    let staging_in = session.create_buffer_host_visible(img_in_size)?;
    staging_in.write(&vec![0u8; img_in_size as usize]);

    // Arguments for the container node builder
    let args: HashMap<String, Argument> = HashMap::from([
        ("resolution".to_string(), math::UVec2::new(width, height).into()),
        ("iterations".to_string(), Argument::I32(5)),
        ("alpha".to_string(), Argument::F32(0.05)),
        ("float_precision".to_string(), Argument::I32(0)), // FP32
        ("clear_history".to_string(), Argument::I32(1)),
    ]);

    // Load dependencies
    session.load_compute_node_builder("lluvia/math/normalize/ImageNormalizer_r8ui_r32f")?;
    session.load_compute_node_builder("lluvia/opticalflow/HornSchunck/HornSchunck/ImageProcessor")?;
    session.load_compute_node_builder("lluvia/opticalflow/HornSchunck/HornSchunck/NumericIteration")?;

    // Load the container node builder
    let builder = session.load_container_node_builder("lluvia/opticalflow/HornSchunck/HornSchunck")?;

    let mut container_node = builder
        .build_descriptor(args)?
        .bind("in_gray", ll::node::NodePort::ImageView(view_in))?
        .build()?;

    container_node.init()?;

    // Verify output port characteristics
    let out_flow = container_node.port("out_flow");
    assert!(out_flow.is_some());
    if let Some(ll::node::NodePort::ImageView(view_flow)) = out_flow {
        assert_eq!(view_flow.image().descriptor().width, width);
        assert_eq!(view_flow.image().descriptor().height, height);
        assert_eq!(
            view_flow.image().descriptor().channel_count,
            ll::image::ChannelCount::C2
        );
        assert_eq!(
            view_flow.image().descriptor().channel_type,
            ll::image::ChannelType::Float32
        );
    } else {
        panic!("out_flow is not an ImageView!");
    }

    let out_gray = container_node.port("out_gray");
    assert!(out_gray.is_some());
    if let Some(ll::node::NodePort::ImageView(view_gray)) = out_gray {
        assert_eq!(view_gray.image().descriptor().width, width);
        assert_eq!(view_gray.image().descriptor().height, height);
        assert_eq!(
            view_gray.image().descriptor().channel_count,
            ll::image::ChannelCount::C1
        );
        assert_eq!(
            view_gray.image().descriptor().channel_type,
            ll::image::ChannelType::Float32
        );
    } else {
        panic!("out_gray is not an ImageView!");
    }

    let staging_flow = session.create_buffer_host_visible((width * height * 2 * 4) as u64)?;

    // Run HornSchunck
    let mut builder_cb = session.create_command_buffer_builder()?;
    builder_cb.copy_buffer_to_image(staging_in, img_in)?;
    builder_cb.record_container_node(&mut container_node)?;
    if let Some(ll::node::NodePort::ImageView(view_flow)) = container_node.port("out_flow") {
        builder_cb.copy_image_to_buffer(view_flow.image().clone(), staging_flow.clone())?;
    }
    let command_buffer = builder_cb.build_command_buffer()?;
    session.run(command_buffer)?;

    // Verify that the flow is exactly zero
    let flow_bytes = staging_flow.read();
    let flow_data: &[f32] = bytemuck::cast_slice(&flow_bytes);
    for &val in flow_data {
        assert_eq!(val, 0.0);
    }

    Ok(())
}

#[test]
fn test_horn_schunck_translation() -> Result<()> {
    let session = ll::Session::new(ll::SessionDescriptor::default())?;

    let width = 32u32;
    let height = 32u32;

    // Create frame 0: box at (10..20, 10..20)
    let mut frame0 = vec![0u8; (width * height) as usize];
    for y in 10..20 {
        for x in 10..20 {
            frame0[(y * width + x) as usize] = 255;
        }
    }

    // Create frame 1: box shifted 1 pixel to the right -> at (10..20, 11..21)
    let mut frame1 = vec![0u8; (width * height) as usize];
    for y in 10..20 {
        for x in 11..21 {
            frame1[(y * width + x) as usize] = 255;
        }
    }

    let in_gray_desc = ll::image::ImageDescriptor::builder()
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
    let img_in = session.create_image(in_gray_desc)?;
    let view_desc = ll::image::ImageViewDescriptor::default();
    let view_in = img_in.create_image_view(&view_desc)?;

    // Create staging buffers
    let staging_in = session.create_buffer_host_visible((width * height) as u64)?;

    // Load dependencies
    session.load_compute_node_builder("lluvia/math/normalize/ImageNormalizer_r8ui_r32f")?;
    session.load_compute_node_builder("lluvia/opticalflow/HornSchunck/HornSchunck/ImageProcessor")?;
    session.load_compute_node_builder("lluvia/opticalflow/HornSchunck/HornSchunck/NumericIteration")?;

    let builder = session.load_container_node_builder("lluvia/opticalflow/HornSchunck/HornSchunck")?;

    // Build the container node. During init, CopyInGrayToInGrayOld is loaded.
    let args: HashMap<String, Argument> = HashMap::from([
        ("resolution".to_string(), math::UVec2::new(width, height).into()),
        ("iterations".to_string(), Argument::I32(50)), // run multiple iterations to propagate flow
        ("alpha".to_string(), Argument::F32(0.05)),
        ("float_precision".to_string(), Argument::I32(0)),
        ("clear_history".to_string(), Argument::I32(1)),
    ]);

    let mut container_node = builder
        .build_descriptor(args)?
        .bind("in_gray", ll::node::NodePort::ImageView(view_in.clone()))?
        .build()?;

    // Run container node init (this allocates out_gray/in_gray_old)
    container_node.init()?;

    let view_gray = match container_node.port("out_gray") {
        Some(ll::node::NodePort::ImageView(v)) => v.clone(),
        _ => panic!("out_gray not found!"),
    };

    // Normalize and copy frame0 directly to the allocated out_gray image view
    let frame0_float: Vec<f32> = frame0.iter().map(|&x| x as f32 / 255.0).collect();
    let staging_gray_init = session.create_buffer_host_visible((width * height * 4) as u64)?;
    staging_gray_init.write(bytemuck::cast_slice(&frame0_float));

    let mut builder_gray_init = session.create_command_buffer_builder()?;
    builder_gray_init.copy_buffer_to_image(staging_gray_init, view_gray.image().clone())?;
    let cb_gray_init = builder_gray_init.build_command_buffer()?;
    session.run(cb_gray_init)?;

    // Write frame1 to staging buffer and copy to img_in (which is bound as "in_gray")
    staging_in.write(&frame1);
    let mut builder_run = session.create_command_buffer_builder()?;
    builder_run.copy_buffer_to_image(staging_in, img_in.clone())?;
    builder_run.record_container_node(&mut container_node)?;

    let staging_flow = session.create_buffer_host_visible((width * height * 2 * 4) as u64)?;
    if let Some(ll::node::NodePort::ImageView(view_flow)) = container_node.port("out_flow") {
        builder_run.copy_image_to_buffer(view_flow.image().clone(), staging_flow.clone())?;
    }

    let cb_run = builder_run.build_command_buffer()?;
    session.run(cb_run)?;

    let flow_bytes = staging_flow.read();
    let flow_data: &[f32] = bytemuck::cast_slice(&flow_bytes);

    // Verify the translation direction
    // In our coordinate system, the shift is +1 along the X axis.
    // So sum_dx should be positive (movement to the right), and much larger than sum_dy.
    let mut sum_dx = 0.0f32;
    let mut sum_dy = 0.0f32;
    let mut count = 0;

    for y in 10..20 {
        for x in 9..22 {
            let idx = (y * width + x) as usize;
            sum_dx += flow_data[2 * idx];
            sum_dy += flow_data[2 * idx + 1].abs();
            count += 1;
        }
    }

    let avg_dx = sum_dx / count as f32;
    let avg_dy = sum_dy / count as f32;

    println!("avg_dx = {}, avg_dy = {}", avg_dx, avg_dy);

    assert!(
        avg_dx > 0.01,
        "Expected positive X flow (movement to the right), got average dx = {}",
        avg_dx
    );

    assert!(
        avg_dx > avg_dy * 1.5,
        "Expected horizontal flow to dominate vertical flow, got average dx = {}, average dy magnitude = {}",
        avg_dx,
        avg_dy
    );

    Ok(())
}
