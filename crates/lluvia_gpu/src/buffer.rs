bitflags::bitflags! {
    /// Buffer usage flags
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct BufferUsages: u32 {
        /// Buffer can be used as a vertex buffer
        const VERTEX = wgpu::BufferUsages::VERTEX.bits();
        /// Buffer can be used as an index buffer
        const INDEX = wgpu::BufferUsages::INDEX.bits();
        /// Buffer can be used as a uniform buffer
        const UNIFORM = wgpu::BufferUsages::UNIFORM.bits();
        /// Buffer can be used as a storage buffer
        const STORAGE = wgpu::BufferUsages::STORAGE.bits();
        /// Buffer can be used as a copy source
        const COPY_SRC = wgpu::BufferUsages::COPY_SRC.bits();
        /// Buffer can be used as a copy destination
        const COPY_DST = wgpu::BufferUsages::COPY_DST.bits();
        /// Buffer can be used as mapping source
        const MAP_READ = wgpu::BufferUsages::MAP_READ.bits();
        /// Buffer can be used as mapping destination
        const MAP_WRITE = wgpu::BufferUsages::MAP_WRITE.bits();
    }
}

impl Into<wgpu::BufferUsages> for BufferUsages {
    fn into(self) -> wgpu::BufferUsages {
        wgpu::BufferUsages::from_bits(self.bits()).unwrap()
    }
}

#[derive(bon::Builder, Debug, Clone)]
pub struct BufferDescriptor {
    /// The label for the buffer
    pub label: Option<String>,

    /// The size of the buffer in bytes
    pub size: usize,

    /// The usage flags for the buffer
    pub usage: BufferUsages,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    // usage flags
    pub(crate) handle: wgpu::Buffer,
}

impl Buffer {
    pub fn size(&self) -> usize {
        self.handle.size() as usize
    }
}
