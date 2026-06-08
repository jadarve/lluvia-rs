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

    #[allow(dead_code)]
    struct NodeDescriptorStageBuilder {
        session: std::sync::Arc<ll::Session>,
        inner: Box<dyn ll::node::ComputeNodeBuilder>,
    }

    #[allow(dead_code)]
    struct ComputeNodeStageBuilder {
        descriptor: ll::node::ComputeNodeDescriptor,
    }

    #[test]
    fn test_moni() -> Result<()> {
        let session_descriptor = ll::SessionDescriptor::default();
        let session = ll::Session::new(session_descriptor)?;

        let _device_buffer = session.create_buffer_device_local(512)?;
        let _staging_buffer = session.create_buffer_host_visible(512)?;

        let sh = vs::load(session.device())?;
        let program = session.create_program_from_shader_module(sh)?;

        let _desc = ll::node::ComputeNodeDescriptor::builder()
            .add_constant("dummy", ll::node::Constant::Float(1.0))
            .add_port(
                ll::node::PortDescriptor::builder()
                    .binding(0)
                    .direction(ll::node::PortDirection::In)
                    .name("dummy")
                    .port_type(ll::node::PortType::Buffer)
                    .build(),
            )
            .global_shape(lluvia_vk::math::UVec3::new(32, 1, 1)) // I don't really care about local and grid, I want to set global
            .program(program) // FIXME: program is optional, but a descriptor without it has no use.
            .build();

        Ok(())
    }

    // #[test]
    // fn test_scriptable_node() -> Result<()> {
    //     use ll::node::ComputeNodeBuilder2;

    //     let session_descriptor = ll::SessionDescriptor::default();

    //     let session = ll::Session::new(session_descriptor)?;

    //     let device_buffer = session.create_buffer_device_local(512)?;
    //     let staging_buffer = session.create_buffer_host_visible(512)?;

    //     let inner_builder = session.load_compute_node_builder("lluvia/assign")?;
    //     let compute_node = MyScriptableNode::new(session.clone(), inner_builder)
    //         .build_descriptor()?
    //         .set_constant("offset", ll::node::Constant::Float(10.0))?
    //         .bind("out_buffer", ll::node::NodePort::Buffer(device_buffer.clone()))?
    //         .build()?;

    //     // two builders: one for descriptor, one for node. Transfer state.
    //     // let mut c = session.load_compute_node_builder("lluvia/assign")?
    //     //     .transition()?
    //     //     .;

    //     let mut builder_cb = session.create_command_buffer_builder()?;
    //     builder_cb.record_compute_node(&compute_node)?;
    //     builder_cb.copy_buffer(device_buffer.clone(), staging_buffer.clone())?;

    //     let command_buffer = builder_cb.build_command_buffer()?;
    //     session.run(command_buffer)?;

    //     let data = staging_buffer.read();
    //     let floats: &[f32] = bytemuck::cast_slice(&data);

    //     for (i, item) in floats.iter().enumerate() {
    //         assert_eq!(*item, i as f32 + 10.0, "index {i}");
    //     }

    //     Ok(())
    // }
}
