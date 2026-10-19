#[cfg(test)]
mod tests {
    use anyhow::Result;
    use lluvia_vk::{self as ll, node::Argument};
    use std::collections::HashMap;

    fn create_camera_uniform_buffer_data(width: f32, height: f32) -> Vec<u8> {
        // K = [ [width, 0, 0.5*(width-1)],
        //       [0, height, 0.5*(height-1)],
        //       [0, 0, 1] ]
        // Aligned to std140 column-major:
        // Column 0: width, 0, 0, 0
        // Column 1: 0, height, 0, 0
        // Column 2: 0.5*(width-1), 0.5*(height-1), 1.0, 0
        let mut data = vec![0u8; 128];

        // Column 0 of K (index 0..16)
        let col0_k = [width, 0.0, 0.0, 0.0];
        data[0..16].copy_from_slice(bytemuck::cast_slice(&col0_k));

        // Column 1 of K (index 16..32)
        let col1_k = [0.0, height, 0.0, 0.0];
        data[16..32].copy_from_slice(bytemuck::cast_slice(&col1_k));

        // Column 2 of K (index 32..48)
        let col2_k = [0.5 * (width - 1.0), 0.5 * (height - 1.0), 1.0, 0.0];
        data[32..48].copy_from_slice(bytemuck::cast_slice(&col2_k));

        // Kinv is the inverse of K:
        // Kinv = [ [1/width, 0, -0.5*(width-1)/width],
        //          [0, 1/height, -0.5*(height-1)/height],
        //          [0, 0, 1] ]
        // Column 0 of Kinv (index 48..64)
        let col0_kinv = [1.0 / width, 0.0, 0.0, 0.0];
        data[48..64].copy_from_slice(bytemuck::cast_slice(&col0_kinv));

        // Column 1 of Kinv (index 64..80)
        let col1_kinv = [0.0, 1.0 / height, 0.0, 0.0];
        data[64..80].copy_from_slice(bytemuck::cast_slice(&col1_kinv));

        // Column 2 of Kinv (index 80..96)
        let col2_kinv = [-0.5 * (width - 1.0) / width, -0.5 * (height - 1.0) / height, 1.0, 0.0];
        data[80..96].copy_from_slice(bytemuck::cast_slice(&col2_kinv));

        // radialDistortion (index 96..112): [0.5, 0, 0, 0]
        let radial = [0.5f32, 0.0, 0.0, 0.0];
        data[96..112].copy_from_slice(bytemuck::cast_slice(&radial));

        // tangentialDistortion (index 112..128): [0.1, 0, 0, 0]
        let tangential = [0.1f32, 0.0, 0.0, 0.0];
        data[112..128].copy_from_slice(bytemuck::cast_slice(&tangential));

        data
    }

    #[test]
    fn test_camera_undistort_good_use() -> Result<()> {
        let session = ll::Session::new(ll::SessionDescriptor::default())?;

        // Load reference input image using image crate
        let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/koala.jpg");
        let img = image::open(&input_path)?;
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();

        // Create staging buffers and GPU images
        let img_in_size = (width * height * 4) as u64;
        let staging_in = session.create_buffer_host_visible(img_in_size)?;
        staging_in.write(rgba_img.as_raw());

        // Allocate image views and uniform buffers
        let img_in_desc = ll::image::ImageDescriptor::builder()
            .width(width)
            .height(height)
            .depth(1)
            .channel_count(ll::image::ChannelCount::C4)
            .channel_type(ll::image::ChannelType::Uint8)
            .usage(
                vulkano::image::ImageUsage::STORAGE
                    | vulkano::image::ImageUsage::SAMPLED
                    | vulkano::image::ImageUsage::TRANSFER_DST
                    | vulkano::image::ImageUsage::TRANSFER_SRC,
            )
            .build();
        let img_in = session.create_image(img_in_desc)?;
        let view_in_desc = ll::image::ImageViewDescriptor::builder()
            .filter_mode(ll::image::ImageFilterMode::Nearest)
            .address_mode(ll::image::ImageAddressMode::ClampToEdge)
            .normalized_coordinates(false)
            .is_sampled(true)
            .build();
        let view_in = img_in.create_image_view(&view_in_desc)?;

        let camera_data = create_camera_uniform_buffer_data(width as f32, height as f32);
        let in_camera = session.create_buffer_host_visible(camera_data.len() as u64)?;
        in_camera.write(&camera_data);

        let img_out_desc = ll::image::ImageDescriptor::builder()
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
        let img_out = session.create_image(img_out_desc)?;
        let view_out_desc = ll::image::ImageViewDescriptor::default();
        let view_out = img_out.create_image_view(&view_out_desc)?;

        // Load the builder
        let builder = session.load_compute_node_builder("lluvia/camera/CameraUndistort_rgba8ui")?;
        let args: HashMap<String, Argument> =
            HashMap::from([("resolution".to_string(), ll::math::UVec2::new(width, height).into())]);

        let compute_node_builder = match builder.build_descriptor(args) {
            Ok(b) => b,
            Err(e) => {
                panic!("build_descriptor failed with: {:?}", e);
            }
        };

        let mut compute_node = compute_node_builder
            .bind("in_rgba", ll::node::NodePort::ImageView(view_in))?
            .bind("in_camera", ll::node::NodePort::Buffer(in_camera))?
            .bind("out_rgba", ll::node::NodePort::ImageView(view_out))?
            .set_constant("camera_model", ll::node::Constant::Int(0))
            .build()?;

        // Create staging buffer for the output C4 image
        let img_out_size = (width * height * 4) as u64;
        let staging_out = session.create_buffer_host_visible(img_out_size)?;

        let mut builder_cb = session.create_command_buffer_builder()?;
        builder_cb.copy_buffer_to_image(staging_in, img_in)?;
        builder_cb.record_compute_node(&mut compute_node)?;
        builder_cb.copy_image_to_buffer(img_out, staging_out.clone())?;

        let command_buffer = builder_cb.build_command_buffer()?;
        session.run(command_buffer)?;

        // Save output image using the image crate
        let out_data = staging_out.read();
        let out_image = image::RgbaImage::from_raw(width, height, out_data)
            .ok_or_else(|| anyhow::anyhow!("Failed to construct output RgbaImage"))?;

        let rgb_image = image::DynamicImage::ImageRgba8(out_image).into_rgb8();

        let output_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/output_koala_undistort.jpg");
        rgb_image.save(&output_path)?;

        assert!(output_path.exists());

        Ok(())
    }
}
