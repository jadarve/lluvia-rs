---
trigger: always_on
name: rust_developer_agent
description: "Permissions for Rust development, linting, testing, and continuous integration task automation."
version: "1.0"
---

# Allowed Capabilities
- Compiling code using `cargo build`
- Formatting code style via `cargo fmt` and `cargo fmt --all`
- Running general test suites with `cargo test`
- Executing specific targeted tests: `cargo test --package ...`
- Running code analysis with `cargo clippy`
- Running strict linting: `cargo clippy --workspace --all-targets --all-features -- -D warnings`

# Denied/Restricted Capabilities
- Commands modifying critical system configurations
- Unauthorized external shell script executions

# Guidelines
You are fully authorized to auto-execute the listed cargo compilation, formatting, linting, and testing commands without manual user confirmation to speed up the development lifecycle.
