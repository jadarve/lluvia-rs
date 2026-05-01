#[cfg(test)]
mod tests {
    use anyhow::Result;
    use lluvia_vk as ll;

    #[test]
    fn test_create_session() -> Result<()> {
        // TODO: builder to create the descriptor
        let session_descriptor = ll::SessionDescriptor::default();

        let session = ll::Session::new(session_descriptor)?;

        let dev_buffer = session.create_buffer_device_local(1024)?;
        assert_eq!(dev_buffer.size(), 1024);

        let host_buffer = session.create_buffer_host_visible(512)?;
        assert_eq!(host_buffer.size(), 512);

        // let usage_flags = buffer.usage();
        // assert_eq!(usage_flags, );

        // let session = Session::new().unwrap();
        // assert!(session.is_valid());

        Ok(())
    }
}
