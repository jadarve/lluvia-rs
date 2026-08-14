---
trigger: always_on
name: lluvia_nodes
description: "Guidelines and best practices for writing Lluvia nodes, including Luau builders, Slang shaders, and Rust verification tests."
version: "1.0"
---

# Lluvia Nodes Best Practices

When creating or modifying Lluvia nodes, strictly adhere to the guidelines below for the Luau builder, Slang shader, and Rust verification tests.

## 1. Luau Builder (`NodeName.luau`)

Each Lluvia node must have a Luau builder script defining its inputs, outputs, parameters, and initialization behavior.

- **Strict Type Checking:** Always start the file with `--!strict`.
- **Imports:** Require the lluvia library at the top: `local ll = require("@lib/ll.luau")`.
- **Arguments Type:** Define a type for arguments (e.g., `type RGBA2GrayArguments = { resolution: UVec2 }`).
- **Builder Table:**
  - Define the builder using type annotation: `local builder: ll.ComptueNodeBuidler = { ... }`.
  - Register the builder at the end of the file: `ll:register_compute_node_builder(builder)`.
  - Return `{}` at the end of the file to allow safe importing by other scripts.
- **Description Docstring:**
  - The `description` field must contain documentation about **Arguments** and **Constants** in a structured format as shown below:
    ```luau
    description = [[
        A short description of what the node does.

        Arguments
        ---------

        arg_name: Type, usage
            A descriptive explanation of the argument.

        Constants
        ---------

        constant_name: Type. Defaults to value.
            A descriptive explanation of the constant.
    ]]
    ```
- **Builder Functions:**
  - `build_descriptor = function(self, args: Arguments)`: Cast the arguments to the specific typed table, then use `ll.ComputeNodeDescriptorBuilder()` to define:
    - `function_name("main")`
    - `global_shape(UVec3(...))`
    - `program(ll.load_program("path/to/program"))`
    - Ports with `PortDescriptor(index, "name", direction, type)`
    - Constants with `:add_constant("name", default_value)`
  - `on_node_init = function(self, node)`: If constants are used, retrieve them using `node:get_constant("name")` and load them into `PushConstants` via `node.push_constants = pushConstants`.

---

## 2. Slang Shader (`NodeName.slang`)

Lluvia shaders should be written in Slang and conform to these guidelines:

- **Imports:** Import necessary Slang libraries (e.g., `import lluvia.core;` or `import lluvia.color;`).
- **Bindings:**
  - Match descriptor bindings exactly with the port indices defined in the Luau builder (e.g., `[[vk::binding(0, 0)]] RWTexture2D<uint4> in_rgba;`).
  - Use `[[format("format_name")]]` annotation to specify the texture format.
- **Compute Entry Point:**
  - Use `[shader("compute")]`.
  - Define workgroup dimensions dynamically using specialized constants via the `LLUVIA_WORKGROUP` macro and `[numthreads(workgroup_x, workgroup_y, workgroup_z)]`.
  - The entry point function signature must be `void main(uint3 threadId : SV_DispatchThreadID)`.
- **Boundary Checks:**
  - Always query dimensions dynamically (e.g., `out_image.GetDimensions(width, height)`) and perform safe boundary checks before any writes:
    ```slang
    if (coords.x >= width || coords.y >= height) {
        return;
    }
    ```

---

## 3. Rust Verification Tests (`tests/nodes/...`)

Every Lluvia compute node must be accompanied by a Rust integration test to verify its behavior.

- **Location:** Put integration tests in the appropriate subfolder under `crates/lluvia_vk/tests/nodes/` (e.g. `crates/lluvia_vk/tests/nodes/lluvia/color/`).
- **Error Handling:** Return `anyhow::Result<()>` and use the `?` operator for clean error propagation.
- **Test Structure:**
  - Create a `ll::Session`.
  - Load input data (either programmatically or using reference images from `tests/test-data/`).
  - Allocate GPU buffers/images with correct sizes, channel counts, and usages (`STORAGE`, `TRANSFER_DST`, `TRANSFER_SRC`).
  - Create image views using `ll::image::ImageViewDescriptor`.
  - Pack inputs into a `HashMap<String, Argument>` for node builder arguments.
  - Load the node builder: `session.load_compute_node_builder("path/to/builder")?`.
  - Build the descriptor, bind ports using `bind("port_name", ll::node::NodePort::...)`, and call `.build()?` to instantiate the node.
  - Record commands: Copy staging buffer/image, record the compute node with `record_compute_node()`, and copy the output back to a host-visible staging buffer/image.
  - Execute commands: Run the command buffer on the session.
  - Verify output data correctness using assertions.
