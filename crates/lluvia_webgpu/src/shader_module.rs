#[derive(Debug, Clone)]
pub enum ShaderCode {
    Wgsl(String),
}

#[derive(bon::Builder, Debug, Clone)]
pub struct ShaderModuleDescriptor {
    pub(crate) label: Option<String>,
    pub(crate) code: ShaderCode,
}

#[derive(Debug, Clone)]
pub struct ShaderModule {
    pub(crate) handle: wgpu::ShaderModule,
}
