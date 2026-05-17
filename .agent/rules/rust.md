---
trigger: always_on
---

# Rust Best Practices

When working on this Rust project, strictly adhere to the following best practices:

1. **Idiomatic Code:** Write idiomatic Rust. Pay close attention to compiler and Clippy warnings, as they often suggest more idiomatic patterns.
2. **Error Handling:** Avoid using `.unwrap()` and `.expect()` in production code. Use `Result` and the `?` operator to propagate errors gracefully. Define custom error types when appropriate.
3. **Memory Management:** Minimize unnecessary allocations. Prefer borrowing (`&T`, `&mut T`) over cloning (`.clone()`) unless ownership is strictly required. 
4. **Formatting:** Ensure all code is formatted using standard `cargo fmt`.
5. **Testing:** Write unit tests for your logic inside `#[cfg(test)]` modules within the same file. For larger component tests, rely on integration tests in the `tests/` directory.
6. **Documentation:** Document public modules, structs, traits, and functions using `///` doc comments. Include code examples where helpful.
7. **Dependencies:** Be mindful of adding new dependencies. Evaluate if a lightweight alternative exists or if the standard library is sufficient. Dependdencies must be added at workspace level and referenced on each crate/app.
8. **Checks:** Always run `cargo fmt` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` after making changes to ensure code is properly formatted and passes all linting rules.
