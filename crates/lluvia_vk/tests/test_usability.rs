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

            void main() {

                const uint index = gl_GlobalInvocationID.x;
                outputBuffer[index] = index;
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

        let mut builder = session.create_command_buffer_builder()?;

        builder.record_compute_node(&node)?;

        // Copy from device buffer to staging buffer to verify results
        builder.copy_buffer(device_buffer.clone(), staging_buffer.clone())?;

        let command_buffer = builder.build_command_buffer()?;

        session.run(command_buffer)?;

        // Verify that the buffer was written to
        // The shader writes `outputBuffer[index] = index`
        let data = staging_buffer.read();
        let floats: &[f32] = bytemuck::cast_slice(&data);
        println!("{floats:?}");

        for (i, item) in floats.iter().enumerate() {
            assert_eq!(*item, i as f32);
        }

        Ok(())
    }

    #[test]
    fn test_load_assign_node() -> Result<()> {
        let session_descriptor = ll::SessionDescriptor::default();

        let session = ll::Session::new(session_descriptor)?;

        let builder = session
            .load_compute_node_builder("lluvia/assign")?
            .ok_or_else(|| anyhow::anyhow!("builder not found"))?;

        let node_descriptor = builder.get_descriptor();
        let mut compute_node = session.create_compute_node(node_descriptor)?;

        let device_buffer = session.create_buffer_device_local(512)?;
        let staging_buffer = session.create_buffer_host_visible(512)?;

        use ll::node::Node;
        compute_node.bind("out_buffer", ll::node::NodePort::Buffer(device_buffer.clone()))?;

        builder.init_node(&mut compute_node)?;

        let mut builder_cb = session.create_command_buffer_builder()?;
        builder_cb.record_compute_node(&compute_node)?;
        builder_cb.copy_buffer(device_buffer.clone(), staging_buffer.clone())?;

        let command_buffer = builder_cb.build_command_buffer()?;
        session.run(command_buffer)?;

        let data = staging_buffer.read();
        let floats: &[f32] = bytemuck::cast_slice(&data);

        for (i, item) in floats.iter().enumerate() {
            assert_eq!(*item, i as f32);
        }

        Ok(())
    }
}
