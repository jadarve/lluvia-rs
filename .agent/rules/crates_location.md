# Rules

All crates in this project's workspace MUST be located inside the `crates/` folder.

This helps maintain a clean directory structure at the root level. When generating a new crate, run `cargo new --lib crates/crate_name` and ensure `members` inside the workspace `Cargo.toml` picks it up (e.g. `members = ["crates/*"]`).
