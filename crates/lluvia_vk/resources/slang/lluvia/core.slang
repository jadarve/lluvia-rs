#ifndef LLUVIA_CORE_SLANG_
#define LLUVIA_CORE_SLANG_

/**
 * @brief Macro defining specialization constants for the compute shader workgroup size.
 *
 * This macro defines three specialization constants: `workgroup_x`, `workgroup_y`,
 * and `workgroup_z` mapping to Vulkan constant IDs 1, 2, and 3 respectively.
 * Shaders can use this macro to define their local workgroup dimensions dynamically.
 */
#define LLUVIA_WORKGROUP \
    [[vk::constant_id(1)]] const uint workgroup_x = 1; \
    [[vk::constant_id(2)]] const uint workgroup_y = 1; \
    [[vk::constant_id(3)]] const uint workgroup_z = 1;

#endif // LLUVIA_CORE_SLANG_
