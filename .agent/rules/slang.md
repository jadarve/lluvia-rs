# Slang Best Practices

When working on Slang shader code in this project, strictly adhere to the following best practices:

1. **Compute Entry Point:**
   - Mark compute shaders with `[shader("compute")]`.
   - Set the workgroup dimensions dynamically using specialized constants via the `LLUVIA_WORKGROUP` macro and `[numthreads(workgroup_x, workgroup_y, workgroup_z)]`.
   - The entry point function should be `void main(uint3 threadId : SV_DispatchThreadID)`.

2. **Vulkan Bindings:**
   - Explicitly define descriptor bindings using `[[vk::binding(binding_index, set_index)]]`.
   - Unless using multiple descriptor sets, always use set index `0`.
   - Group push constants under a struct and annotate the instance with `[[vk::push_constant]]`.

3. **Type Conversions and Precision:**
   - Use explicit casting when converting between types (e.g. `(float)index` or `(float2)coords`).
   - For combined image samplers, declare them as `Sampler2D` (for float textures) or generic `Sampler2D<uint4>` (for integer/uint textures).
   - Use `.SampleLevel(coords, 0.0)` to sample from a combined image sampler in a compute shader to prevent derivative errors.

4. **Resource Layout:**
   - For uniform buffers representing structural data like `ll_camera`, group the fields in a structured way matching the host's memory alignment (e.g. following GLSL `std140` rules where matrices have columns padded to 4-component vectors).
   - Use `ConstantBuffer<T>` in Slang for uniform buffers.

5. **Safe Boundary Checks:**
   - Always query output or target image dimensions using `GetDimensions()` or other methods, and verify that the current thread's dispatch coordinates (e.g. `SV_DispatchThreadID`) fall within bounds before writing.
