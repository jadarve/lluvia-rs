#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::Result;
    use lluvia_vk::{self as ll};

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

        let descriptor = ll::node::ComputeNodeDescriptor::builder()
            .program(program)
            .function_name("main")
            .global_shape(lluvia_vk::math::UVec3::new(128, 1, 1))
            .add_port(ll::node::PortDescriptor {
                binding: 0,
                name: "out0".to_string(),
                direction: ll::node::PortDirection::Out,
                port_type: ll::node::PortType::Buffer,
            })
            .build();

        let mut node = session.create_compute_node(descriptor)?;

        use ll::node::Node;
        node.bind("out0", ll::node::NodePort::Buffer(device_buffer.clone()))?;

        // Set push constants with 11.0 offset to match assertions
        let mut push_constants = ll::node::PushConstants::default();
        push_constants.push_f32(11.0);
        node.push_constants = Some(push_constants);

        let mut builder = session.create_command_buffer_builder()?;

        builder.record_compute_node(&mut node)?;

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
        const LENGTH: u64 = 128;
        const BUFFER_SIZE: u64 = LENGTH * std::mem::size_of::<f32>() as u64;
        const OFFSET: f32 = 10.0;

        let session_descriptor = ll::SessionDescriptor::default();

        let session = ll::Session::new(session_descriptor)?;

        let device_buffer = session.create_buffer_device_local(BUFFER_SIZE)?;
        let staging_buffer = session.create_buffer_host_visible(BUFFER_SIZE)?;

        let args: HashMap<String, ll::node::Argument> =
            HashMap::from([("length".to_string(), i32::try_from(LENGTH)?.into())]);

        let builder = session.load_compute_node_builder("lluvia/assign")?;

        let mut compute_node = builder
            .build_descriptor(args)?
            .bind("out_buffer", ll::node::NodePort::Buffer(device_buffer.clone()))?
            .set_constant("offset", ll::node::Constant::Float(OFFSET))
            .build()?;

        let mut builder_cb = session.create_command_buffer_builder()?;
        builder_cb.record_compute_node(&mut compute_node)?;
        builder_cb.copy_buffer(device_buffer.clone(), staging_buffer.clone())?;

        let command_buffer = builder_cb.build_command_buffer()?;
        session.run(command_buffer)?;

        let data = staging_buffer.read();
        let floats: &[f32] = bytemuck::cast_slice(&data);

        for (i, item) in floats.iter().enumerate() {
            assert_eq!(*item, i as f32 + 10.0, "index {i}");
        }

        Ok(())
    }

    #[test]
    fn test_load_assign_slang_node() -> Result<()> {
        const LENGTH: u64 = 128;
        const BUFFER_SIZE: u64 = LENGTH * std::mem::size_of::<f32>() as u64;
        const OFFSET: f32 = 10.0;

        let session_descriptor = ll::SessionDescriptor::default();

        let session = ll::Session::new(session_descriptor)?;

        let device_buffer = session.create_buffer_device_local(BUFFER_SIZE)?;
        let staging_buffer = session.create_buffer_host_visible(BUFFER_SIZE)?;

        let args: HashMap<String, ll::node::Argument> =
            HashMap::from([("length".to_string(), i32::try_from(LENGTH)?.into())]);

        let builder = session.load_compute_node_builder("lluvia/assign_slang")?;

        let mut compute_node = builder
            .build_descriptor(args)?
            .bind("out_buffer", ll::node::NodePort::Buffer(device_buffer.clone()))?
            .set_constant("offset", ll::node::Constant::Float(OFFSET))
            .build()?;

        let mut builder_cb = session.create_command_buffer_builder()?;
        builder_cb.record_compute_node(&mut compute_node)?;
        builder_cb.copy_buffer(device_buffer.clone(), staging_buffer.clone())?;

        let command_buffer = builder_cb.build_command_buffer()?;
        session.run(command_buffer)?;

        let data = staging_buffer.read();
        let floats: &[f32] = bytemuck::cast_slice(&data);

        for (i, item) in floats.iter().enumerate() {
            assert_eq!(*item, i as f32 + 10.0, "index {i}");
        }

        Ok(())
    }

    #[test]
    fn test_load_assign2_node() -> Result<()> {
        let session_descriptor = ll::SessionDescriptor::default();

        let session = ll::Session::new(session_descriptor)?;

        let device_buffer = session.create_buffer_device_local(512)?;
        let staging_buffer = session.create_buffer_host_visible(512)?;

        let builder = session.load_compute_node_builder("lluvia/assign2")?;

        let mut compute_node = builder
            .build_descriptor(std::collections::HashMap::new())?
            .bind("out_buffer", ll::node::NodePort::Buffer(device_buffer.clone()))?
            .set_constant("offset", ll::node::Constant::Float(10.0))
            .build()?;

        let mut builder_cb = session.create_command_buffer_builder()?;
        builder_cb.record_compute_node(&mut compute_node)?;
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
}
