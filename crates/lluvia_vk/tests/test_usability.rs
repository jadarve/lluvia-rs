#[cfg(test)]
mod tests {
    use anyhow::Result;
    use lluvia_vk as ll;

    #[cfg(test)]
    mod vs {
        vulkano_shaders::shader!(
            ty: "compute",
            src: r"
            #version 450

            #ifndef ASSIGN_COMP_
            #define ASSIGN_COMP_


            // Specialization constants to set the local workgroup size.
            layout (
                local_size_x_id = 1, local_size_x = 1,
                local_size_y_id = 2, local_size_y = 1,
                local_size_z_id = 3, local_size_z = 1
            ) in;

            layout(binding = 0) buffer out0 {
                float outputBuffer[];
            };

            layout(push_constant) uniform PushConsts {
                float offset;
            } pushConsts;

            void main() {

                const uint index = gl_GlobalInvocationID.x;
                outputBuffer[index] = index + pushConsts.offset;
            }

            #endif // ASSIGN_COMP_
        ",
        );
    }

    #[test]
    fn test_create_session() -> Result<()> {
        // TODO: builder to create the descriptor
        let session_descriptor = ll::SessionDescriptor::default();

        let session = ll::Session::new(session_descriptor)?;

        let dev_buffer = session.create_buffer_device_local(1024)?;
        assert_eq!(dev_buffer.size(), 1024);

        let host_buffer = session.create_buffer_host_visible(512)?;
        assert_eq!(host_buffer.size(), 512);

        let data = vec![1u8; 512];
        host_buffer.write(&data);
        let read_data = host_buffer.read();
        assert_eq!(data, read_data);

        // TODO: explose API to tell buffer is host visible.

        // let usage_flags = buffer.usage();
        // assert_eq!(usage_flags, );

        // let session = Session::new().unwrap();
        // assert!(session.is_valid());

        Ok(())
    }

    #[test]
    fn test_compute_node() -> Result<()> {
        let session_descriptor = ll::SessionDescriptor::default();

        let session = ll::Session::new(session_descriptor)?;

        let device_buffer = session.create_buffer_device_local(512)?;
        let staging_buffer = session.create_buffer_host_visible(512)?;

        let sh = vs::load(session.device())?;
        let program = session.create_program_from_shader_module(sh)?;

        let descriptor = ll::node::ComputeNodeDescriptor::default()
            .program(program)
            .function_name("main")
            .local_shape(&lluvia_vk::math::UVec3::new(32, 1, 1))
            .grid_shape(&lluvia_vk::math::UVec3::new(4, 1, 1))
            .add_port(ll::node::PortDescriptor {
                binding: 0,
                name: "out0".to_string(),
                direction: ll::node::PortDirection::Out,
                port_type: ll::node::PortType::Buffer,
            });

        let mut node = session.create_compute_node(descriptor)?;

        use ll::node::Node;
        node.bind("out0", ll::node::NodePort::Buffer(device_buffer.clone()))?;

        // Set push constants with 11.0 offset to match assertions
        let mut push_constants = ll::node::PushConstants::default();
        push_constants.push_f32(11.0);
        node.push_constants = Some(push_constants);

        let mut builder = session.create_command_buffer_builder()?;

        builder.record_compute_node(&node)?;

        // Copy from device buffer to staging buffer to verify results
        builder.copy_buffer(device_buffer.clone(), staging_buffer.clone())?;

        let command_buffer = builder.build_command_buffer()?;

        session.run(command_buffer)?;

        // Verify that the buffer was written to
        // The shader writes `outputBuffer[index] = index + offset`
        let data = staging_buffer.read();
        let floats: &[f32] = bytemuck::cast_slice(&data);
        println!("{floats:?}");

        for (i, item) in floats.iter().enumerate() {
            assert_eq!(*item, i as f32 + 11.0);
        }

        Ok(())
    }

    #[test]
    fn test_load_assign_node() -> Result<()> {
        let session_descriptor = ll::SessionDescriptor::default();

        let session = ll::Session::new(session_descriptor)?;

        let builder = session.load_compute_node_builder("lluvia/assign")?;

        let node_descriptor = builder.get_descriptor()?;
        let mut compute_node = session.create_compute_node(node_descriptor)?;

        let device_buffer = session.create_buffer_device_local(512)?;
        let staging_buffer = session.create_buffer_host_visible(512)?;

        use ll::node::Node;
        compute_node.bind("out_buffer", ll::node::NodePort::Buffer(device_buffer.clone()))?;

        // Set the offset constant to 10.0
        compute_node.set_constant("offset", ll::node::Constant::Float(10.0));

        // this is different to Lluvia Cpp. There, the compute_node instance holds the reference to the builder
        // so that when the node is initialized, the builder is called.
        // Here the builder and the compute_node are independent.
        builder.init_node(&mut compute_node)?;

        let mut builder_cb = session.create_command_buffer_builder()?;
        builder_cb.record_compute_node(&compute_node)?;
        builder_cb.copy_buffer(device_buffer.clone(), staging_buffer.clone())?;

        let command_buffer = builder_cb.build_command_buffer()?;
        session.run(command_buffer)?;

        let data = staging_buffer.read();
        let floats: &[f32] = bytemuck::cast_slice(&data);

        for (i, item) in floats.iter().enumerate() {
            assert_eq!(*item, i as f32 + 10.0);
        }

        Ok(())
    }

    #[test]
    fn test_luau_load_program() -> Result<()> {
        let session = ll::Session::new(ll::SessionDescriptor::default())?;

        // Evaluate a Luau snippet that calls ll.load_program via the native
        // global injected by set_session.  The `lluvia/assign/assign` shader
        // is embedded in the crate, so it must be found.
        let script = r#"
            local ll = require("@lib/ll.luau")
            local program = ll.load_program("lluvia/assign/assign")
            assert(program ~= nil, "load_program returned nil")
        "#;

        session
            .run_script(script)
            .map_err(|e| anyhow::anyhow!("Luau error: {e}"))?;

        Ok(())
    }

    #[test]
    fn test_rgba2gray() -> Result<()> {
        let session = ll::Session::new(ll::SessionDescriptor::default())?;

        // Load the RGBA2Gray node builder from Luau
        let builder = session.load_compute_node_builder("lluvia/color/RGBA2Gray")?;
        let node_descriptor = builder.get_descriptor()?;
        let mut compute_node = session.create_compute_node(node_descriptor)?;

        // Load reference input image using image crate
        let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/koala.jpg");
        let img = image::open(&input_path)?;
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();

        // Create staging buffers and GPU images
        let img_in_size = (width * height * 4) as u64;
        let staging_in = session.create_buffer_host_visible(img_in_size)?;
        staging_in.write(rgba_img.as_raw());

        let img_in_desc = ll::image::ImageDescriptor::default()
            .width(width)
            .height(height)
            .channel_count(ll::image::ChannelCount::C4)
            .channel_type(ll::image::ChannelType::Uint8)
            .usage(
                vulkano::image::ImageUsage::STORAGE
                    | vulkano::image::ImageUsage::TRANSFER_DST
                    | vulkano::image::ImageUsage::TRANSFER_SRC,
            );

        let img_in = session.create_image(img_in_desc)?;

        let view_desc = ll::image::ImageViewDescriptor::default();
        let view_in = img_in.create_image_view(&view_desc)?;

        let img_out_desc = ll::image::ImageDescriptor::default()
            .width(width)
            .height(height)
            .channel_count(ll::image::ChannelCount::C1)
            .channel_type(ll::image::ChannelType::Uint8)
            .usage(
                vulkano::image::ImageUsage::STORAGE
                    | vulkano::image::ImageUsage::TRANSFER_DST
                    | vulkano::image::ImageUsage::TRANSFER_SRC,
            );
        let img_out = session.create_image(img_out_desc)?;
        let view_out = img_out.create_image_view(&view_desc)?;

        // Bind the image views
        use ll::node::Node;
        compute_node.bind("in_rgba", ll::node::NodePort::ImageView(view_in))?;
        compute_node.bind("out_gray", ll::node::NodePort::ImageView(view_out))?;

        // Initialize node (calls the Luau builder's on_node_init)
        builder.init_node(&mut compute_node)?;

        // Set the grid shape based on image dimensions and workgroup local shape (32, 32, 1)
        let grid_x = width.div_ceil(32);
        let grid_y = height.div_ceil(32);
        compute_node.set_grid_shape(&lluvia_vk::math::UVec3::new(grid_x, grid_y, 1));

        // Create staging buffer for the output single-channel image
        let img_out_size = (width * height) as u64;
        let staging_out = session.create_buffer_host_visible(img_out_size)?;

        // Build command buffer: copy input data to GPU image, run compute node, copy output image back to staging buffer
        let mut builder_cb = session.create_command_buffer_builder()?;
        builder_cb.copy_buffer_to_image(staging_in, img_in)?;
        builder_cb.record_compute_node(&compute_node)?;
        builder_cb.copy_image_to_buffer(img_out, staging_out.clone())?;

        let command_buffer = builder_cb.build_command_buffer()?;
        session.run(command_buffer)?;

        // Save output image using the image crate
        let out_data = staging_out.read();
        let gray_image = image::GrayImage::from_raw(width, height, out_data)
            .ok_or_else(|| anyhow::anyhow!("Failed to construct output GrayImage"))?;

        let output_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-data/output_koala_gray.jpg");
        gray_image.save(&output_path)?;

        assert!(output_path.exists());

        Ok(())
    }
}
