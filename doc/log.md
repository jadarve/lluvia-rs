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