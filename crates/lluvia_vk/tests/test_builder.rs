#[cfg(test)]
mod tests {
    use anyhow::Result;
    use lluvia_vk::{self as ll, node::Node};

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

    struct MyScriptableNode {
        session: std::sync::Arc<ll::Session>,
        inner: Box<dyn ll::node::ComputeNodeBuilder>,
        descriptor: Option<ll::node::ComputeNodeDescriptor>,
        bindings: std::collections::HashMap<String, ll::node::NodePort>,
    }

    impl MyScriptableNode {
        fn new(session: std::sync::Arc<ll::Session>, inner: Box<dyn ll::node::ComputeNodeBuilder>) -> Self {
            Self {
                session,
                inner,
                descriptor: None,
                bindings: std::collections::HashMap::new(),
            }
        }
    }

    impl ll::node::ComputeNodeBuilder2 for MyScriptableNode {
        fn build_descriptor(&mut self) -> Result<&mut Self, ll::node::ComputeNodeBuilderError> {
            let mut args = std::collections::HashMap::new();
            args.insert("length".to_string(), ll::node::Argument::I32(128));
            self.descriptor = Some(self.inner.build_descriptor(args)?);
            Ok(self)
        }

        fn get_descriptor(&mut self) -> Result<ll::node::ComputeNodeDescriptor, ll::node::ComputeNodeBuilderError> {
            if self.descriptor.is_none() {
                self.build_descriptor()?;
            }

            Ok(self.descriptor.as_ref().unwrap().clone())
        }

        fn init_node(&self, node: &mut ll::node::ComputeNode) -> Result<(), ll::node::ComputeNodeError> {
            self.inner.init_node(node)
        }

        fn set_constant(
            &mut self,
            name: impl Into<String>,
            value: ll::node::Constant,
        ) -> Result<&mut Self, ll::node::ComputeNodeBuilderError> {
            self.descriptor = match self.descriptor.take() {
                Some(mut descriptor) => {
                    descriptor.constants.insert(name.into(), value);
                    Some(descriptor)
                }
                None => {
                    return Err(ll::node::ComputeNodeBuilderError::RuntimeError {
                        msg: "descriptor not initialized".to_string(),
                    });
                }
            };

            Ok(self)
        }

        fn bind(
            &mut self,
            name: &str,
            obj: ll::node::NodePort,
        ) -> Result<&mut Self, ll::node::ComputeNodeBuilderError> {
            self.bindings.insert(name.to_string(), obj);
            Ok(self)
        }

        fn build(&mut self) -> Result<ll::node::ComputeNode, ll::node::ComputeNodeBuilderError> {
            // FIXME: should not need 2 descriptors
            let descriptor = self.get_descriptor()?;

            // create the node
            let mut node = self
                .session
                .create_compute_node(descriptor)
                .map_err(|e| ll::node::ComputeNodeBuilderError::RuntimeError { msg: e.to_string() })?;

            // bind ports
            for (name, port) in self.bindings.drain() {
                node.bind(&name, port)
                    .map_err(|e| ll::node::ComputeNodeBuilderError::RuntimeError { msg: e.to_string() })?;
            }

            ///////////////////////////////////////////////////////////////////
            // This block is done by the script
            // node.push_constants = Some(d2.con)
            // push constants
            // let mut push_constants = ll::node::PushConstants::default();

            // for (_name, value) in d2.constants.drain() {
            //     match value {
            //         ll::node::Constant::Float(f) => push_constants.push_f32(f),
            //         ll::node::Constant::Int(i) => push_constants.push_i32(i),
            //         _ => {
            //             return Err(ll::node::ComputeNodeBuilderError::RuntimeError {
            //                 msg: "invalid constant type".to_string(),
            //             });
            //         }
            //     }
            // }

            // node.push_constants = Some(push_constants);

            // // FIXME: hardcoded to test
            // node.set_grid_shape(&ll::math::UVec3::new(128, 1, 1));

            self.inner
                .init_node(&mut node)
                .map_err(|e| ll::node::ComputeNodeBuilderError::RuntimeError { msg: e.to_string() })?;

            Ok(node)
        }
    }

    #[test]
    fn test_scriptable_node() -> Result<()> {
        use ll::node::ComputeNodeBuilder2;

        let session_descriptor = ll::SessionDescriptor::default();

        let session = ll::Session::new(session_descriptor)?;

        let device_buffer = session.create_buffer_device_local(512)?;
        let staging_buffer = session.create_buffer_host_visible(512)?;

        let inner_builder = session.load_compute_node_builder("lluvia/assign")?;
        let compute_node = MyScriptableNode::new(session.clone(), inner_builder)
            .build_descriptor()?
            .set_constant("offset", ll::node::Constant::Float(10.0))?
            .bind("out_buffer", ll::node::NodePort::Buffer(device_buffer.clone()))?
            .build()?;

        // two builders: one for descriptor, one for node. Transfer state.
        // let mut c = session.load_compute_node_builder("lluvia/assign")?
        //     .transition()?
        //     .;

        let mut builder_cb = session.create_command_buffer_builder()?;
        builder_cb.record_compute_node(&compute_node)?;
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
}
