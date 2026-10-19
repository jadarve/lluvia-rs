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
            let properties = device.properties();
            // println!("Device: {}", props.device_name);
            // println!("  Type: {:?}", props.device_type);

            // let extensions = device.supported_extensions();
            // println!("  Supported extensions:\n{:#?}", extensions);
            // println!();

            // Get subgroup size (typically 32 or 64; default to 32 if None)
            let subgroup_size = properties.subgroup_size.unwrap_or(32);
            println!("  Subgroup size: {}", subgroup_size);

            // Get the maximum allowed workgroup size dimensions and invocations
            let max_invocations = properties.max_compute_work_group_invocations;
            let max_workgroup_size = properties.max_compute_work_group_size; // [u32; 3]
            println!("  Max invocations: {}", max_invocations);
            println!("  Max workgroup size: {:?}", max_workgroup_size);
        }

        println!("Total devices found: {}", physical_devices.len());
    }
}
