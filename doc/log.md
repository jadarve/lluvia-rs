## 2026-05-01

* Added initial configuration for Luau types.
* I should aim for a end-to-end example using compute-pipelines
  * No scripting.
  * Initially using buffers: assign, copy.
  * Use glsl first, then slang.

## 2026-05-02

* Got working compute-node example using glsl.
* Two possible branches to continue:
  * Add Luau support.
  * Python wrappers.

## 2026-05-16

* Python wrappers, see pyo3-example repo.
  * See https://github.com/PyO3/pyo3/pull/6020 for native enum support.
  * Got a basic version.
    * Missing docstrings.
    * Missing tests.
* Luau support

## 2026-06-07

* Added support for Arguments in ComputeNodeBuilder.
* Builder pattern looks fine for now.
* Ready to work on Container nodes.
* Still want to do render nodes.
* Check dispatching command buffers from different threads to queues.
* Check Slang shaders

## 2026-06-20

* Review interpreter mod.r
  - submodule for global functions.
  - Enum from string implementations.
* For compute nodes
  - consider convention for 1D, 2D, 3D shapes. Currently using `resolution` for 2D.
* Support UVec2 constants. See ImagePyramid_r8ui.luau
