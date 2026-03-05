---
name: Run Clippy and Fix Lints
description: How to run cargo clippy and solve the lint problems it reports.
---

# Clippy Skill

This skill outlines how to run `cargo clippy` systematically to identify and resolve linting issues in the Rust codebase.

## Execution Steps:

1. **Run Clippy:**
   Use the `run_command` tool to execute the following command in the `/home/juan/git/rust_antigravity` workspace root:
   ```bash
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   ```
   This command enforces that all warnings are treated as errors to maintain a high bar for code quality.

2. **Analyze the Output:**
   If the command fails, carefully read the terminal output to understand the lint warnings. Clippy usually provides excellent suggestions, explanations, and even exact replacement strings.

3. **Locate and Fix Issues:**
   - Use the `view_file` tool to inspect the specific lines of code raising the warnings.
   - Use the `replace_file_content` or `multi_replace_file_content` tools to apply Clippy's suggested fixes or rewrite the code to be more idiomatic. Be mindful of preserving the surrounding logic.

4. **Iterate:**
   After applying fixes, run the clippy command again to verify the issues are truly resolved. Repeat the process until the command passes successfully with no warnings.
