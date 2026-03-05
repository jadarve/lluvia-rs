use crate::ComputeNode;

pub struct CommandEncoder {
    pub(crate) handle: wgpu::CommandEncoder,
}

impl CommandEncoder {
    pub(crate) fn new(handle: wgpu::CommandEncoder) -> Self {
        Self { handle }
    }

    pub async fn run_compute_node(&mut self, compute_node: &ComputeNode) {
        let mut cpass = self
            .handle
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });

        cpass.set_pipeline(&compute_node.handle);

        let bind_group = compute_node.get_bind_group().await;

        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(4, 1, 1);
    }

    pub async fn copy_buffer_to_buffer(&mut self, src: &crate::Buffer, dst: &crate::Buffer) {
        self.handle
            .copy_buffer_to_buffer(&src.handle, 0, &dst.handle, 0, src.size() as u64);
    }

    pub fn finish(self) -> crate::CommandBuffer {
        crate::CommandBuffer {
            handle: self.handle.finish(),
        }
    }
}
