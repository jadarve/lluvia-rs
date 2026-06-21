use anyhow::Result;
use lluvia_vk::{self as ll, math, node::Argument};
use std::collections::HashMap;

#[test]
fn test_image_downsample_x() -> Result<()> {
    let session = ll::Session::new(ll::SessionDescriptor::default())?;

    let width = 10u32;
    let height = 10u32;

    // Create input data
    let mut input_data = vec![0u8; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            input_data[(y * width + x) as usize] = (x * 10 + y) as u8;
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

    // Arguments for resolution
    let args: HashMap<String, Argument> =
        HashMap::from([("resolution".to_string(), math::UVec2::new(width, height).into())]);

    let builder_down = session.load_compute_node_builder("lluvia/imgproc/ImageDownsampleX_r8ui")?;

    let out_width = width / 2;
    let img_out_desc = ll::image::ImageDescriptor::builder()
        .width(out_width)
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
    let img_out = session.create_image(img_out_desc)?;
    let view_out = img_out.create_image_view(&view_desc)?;

    let mut node_down = builder_down
        .build_descriptor(args)?
        .bind("in_gray", ll::node::NodePort::ImageView(view_in))?
        .bind("out_gray", ll::node::NodePort::ImageView(view_out))?
        .build()?;

    let staging_out = session.create_buffer_host_visible((out_width * height) as u64)?;

    // Run DownsampleX
    let mut builder_cb = session.create_command_buffer_builder()?;
    builder_cb.copy_buffer_to_image(staging_in, img_in)?;
    builder_cb.record_compute_node(&mut node_down)?;
    builder_cb.copy_image_to_buffer(img_out, staging_out.clone())?;
    let command_buffer = builder_cb.build_command_buffer()?;
    session.run(command_buffer)?;

    let output_data = staging_out.read();

    // Verify values matching downsampling arithmetic
    for y in 0..height {
        for x in 0..out_width {
            let img_0 = input_data[(y * width + 2 * x) as usize] as u32;
            let img_m = if x == 0 {
                img_0
            } else {
                input_data[(y * width + 2 * x - 1) as usize] as u32
            };
            let img_p = if x == out_width - 1 {
                img_0
            } else {
                input_data[(y * width + 2 * x + 1) as usize] as u32
            };

            let expected = (img_0 >> 1) + ((img_m + img_p) >> 2);
            let actual = output_data[(y * out_width + x) as usize] as u32;

            assert_eq!(
                actual, expected,
                "Value mismatch at (x={}, y={}): expected {}, got {}",
                x, y, expected, actual
            );
        }
    }

    Ok(())
}
