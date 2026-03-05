#[cfg(test)]
mod tests {

    use vulkano::{
        VulkanLibrary,
        instance::{Instance, InstanceCreateInfo},
    };

    /// Enumerates all available Vulkan physical devices, printing their
    /// name and the list of supported device extensions.
    #[test]
    fn test_enumerate_devices() {
        let library = VulkanLibrary::new().expect("failed to load Vulkan library");

        println!("  Instance extensions:\n{:#?}", library.supported_extensions());

        let instance = Instance::new(library, InstanceCreateInfo::default()).expect("failed to create Vulkan instance");

        let physical_devices: Vec<_> = instance
            .enumerate_physical_devices()
            .expect("failed to enumerate physical devices")
            .collect();

        assert!(!physical_devices.is_empty(), "no Vulkan-capable devices found");

        for device in &physical_devices {
            let props = device.properties();
            println!("Device: {}", props.device_name);
            println!("  Type: {:?}", props.device_type);

            let extensions = device.supported_extensions();
            println!("  Supported extensions:\n{:#?}", extensions);
            println!();
        }

        println!("Total devices found: {}", physical_devices.len());
    }
}
