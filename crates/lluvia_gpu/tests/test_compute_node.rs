use anyhow::Result;
use lluvia_gpu as llgpu;

const SHADER_CODE_WGSL: &str = r#"
@group(0) @binding(0)
var<storage, read_write> outputBuffer: array<f32>;

@compute @workgroup_size(256)
fn main(
  @builtin(global_invocation_id)
  global_id : vec3u,

  @builtin(local_invocation_id)
  local_id : vec3u,
) {
  // Avoid accessing the buffer out of bounds
  if (global_id.x >= 1024u) {
    return;
  }

  outputBuffer[global_id.x] = f32(global_id.x);
}
"#;

// FIXME: Should be able to run in WASM environment
#[tokio::test]
async fn test_compute_node() -> Result<()> {
    let session = llgpu::Session::new().await;
    assert!(session.is_ok());
    let session = session.unwrap();

    let shader_desc = llgpu::ShaderModuleDescriptor::builder()
        .label("shader".to_string())
        .code(llgpu::ShaderCode::Wgsl(SHADER_CODE_WGSL.to_string()))
        .build();

    let shader_module = session.create_shader_module(&shader_desc);

    let compute_desc = llgpu::ComputeNodeDescriptor::builder()
        .label("some desc".to_string())
        .entry_point("main".to_string())
        .shader_module(shader_module)
        .ports(vec![llgpu::PortDescriptor::builder()
            .name("outputBuffer".to_string())
            .direction(llgpu::PortDirection::Output)
            .port_type(llgpu::PortType::Buffer)
            .binding(0)
            .build()])
        .build();

    let compute_node = session.create_compute_node(&compute_desc);

    // then I can create a buffer and bind it to the compute node
    let buffer_desc = llgpu::BufferDescriptor::builder()
        .label("test_buffer".to_owned())
        .size(1024)
        .usage(llgpu::BufferUsages::STORAGE | llgpu::BufferUsages::COPY_DST)
        .build();
    let buffer = session.create_buffer_from_descriptor(&buffer_desc).await?;

    compute_node.bind("outputBuffer", &buffer).await;

    // link the node as an operation in a command encoder
    let mut command_encoder = session.create_command_encoder();

    command_encoder.run_compute_node(&compute_node).await;

    // run the node
    let command_buffer = command_encoder.finish();

    session.run_command_buffer(&command_buffer);

    Ok(())
}
