use anyhow::Result;
use lluvia_vk::{self as ll, node::Argument};
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
