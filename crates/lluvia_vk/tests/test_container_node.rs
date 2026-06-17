#[cfg(test)]
mod tests {
    use anyhow::Result;
    use lluvia_vk::{
        self as ll,
        node::{Argument, Constant, NodePort},
    };
    use std::collections::HashMap;

    #[test]
    fn test_load_assign_container_node() -> Result<()> {
        const LENGTH: u64 = 128;
        const BUFFER_SIZE: u64 = LENGTH * std::mem::size_of::<f32>() as u64;
        const OFFSET: f32 = 10.0;

        let session_descriptor = ll::SessionDescriptor::default();
        let session = ll::Session::new(session_descriptor)?;

        let device_buffer = session.create_buffer_device_local(BUFFER_SIZE)?;
        let staging_buffer = session.create_buffer_host_visible(BUFFER_SIZE)?;

        let args: HashMap<String, Argument> = HashMap::from([("length".to_string(), i32::try_from(LENGTH)?.into())]);

        // Load the container node builder and build the node
        let builder = session.load_container_node_builder("lluvia/assign_container")?;

        let mut container_node = builder
            .build_descriptor(args)?
            .bind("out_buffer", NodePort::Buffer(device_buffer.clone()))?
            .set_constant("offset", Constant::Float(OFFSET))
            .build()?;

        // Record commands
        let mut builder_cb = session.create_command_buffer_builder()?;

        // Because of this record function, the container node keeps a weak interpreter reference.
        // That's not good as links container nodes to the interpreter.
        // I could use the builder to record the node instead.
        builder_cb.record_container_node(&mut container_node)?;
        builder_cb.copy_buffer(device_buffer, staging_buffer.clone())?;

        let command_buffer = builder_cb.build_command_buffer()?;
        session.run(command_buffer)?;

        // Verify output
        let data = staging_buffer.read();
        let floats: &[f32] = bytemuck::cast_slice(&data);

        for (i, item) in floats.iter().enumerate() {
            assert_eq!(*item, i as f32 + OFFSET, "Mismatch at index {i}");
        }

        Ok(())
    }
}
