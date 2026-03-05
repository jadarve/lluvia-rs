use anyhow::Result;
use lluvia_webgpu as llgpu;

// FIXME: Should be able to run in WASM environment
#[tokio::test]
async fn test_session() -> Result<()> {
    let session = llgpu::Session::new().await;
    assert!(session.is_ok());
    // let session = session.unwrap();

    // let buffer_desc = llgpu::BufferDescriptor::builder()
    //     .label("test_buffer".to_owned())
    //     .size(1024)
    //     .usage(
    //         llgpu::BufferUsages::VERTEX
    //             | llgpu::BufferUsages::INDEX
    //             | llgpu::BufferUsages::MAP_READ,
    //     )
    //     .build();

    // let buffer = session.create_buffer_from_descriptor(&buffer_desc).await?;
    // assert_eq!(buffer.size(), 1024);

    Ok(())
}
