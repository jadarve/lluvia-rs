module core;

/**
 * @brief Specialization constants for the compute shader workgroup size.
 *
 * These define three specialization constants: `workgroup_x`, `workgroup_y`,
 * and `workgroup_z` mapping to Vulkan constant IDs 1, 2, and 3 respectively.
 * Shaders can use these constants to define their local workgroup dimensions dynamically.
 */
public [[vk::constant_id(1)]] const uint workgroup_x = 1;
public [[vk::constant_id(2)]] const uint workgroup_y = 1;
public [[vk::constant_id(3)]] const uint workgroup_z = 1;
