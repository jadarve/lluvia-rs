# Luau Best Practices

When working on Luau code in this project, strictly adhere to the following best practices:

1. **Strict Typing:** Always include `--!strict` at the top of every Luau file to enable strict type checking. Explicitly define types (`type` or `export type`) for all data structures, function parameters, and return values to leverage Luau's type system effectively.
2. **Local Variables:** Always declare variables using `local`. Avoid global variables entirely to prevent scope pollution and improve performance.
3. **Module Structure:** Structure reusable code and libraries as modules. A module should return a single table containing the exported functions and types, or a single entity if that is its sole purpose.
4. **Naming Conventions:**
   - Use `snake_case` for local variables, module instances, parameters, functions, and method calls on native objects/wrappers (e.g., `local compute_node = ...`, `node:configure_grid_shape(...)`). This aligns with the broader Rust conventions used in the project.
   - Use `PascalCase` for type aliases, classes, and exported interfaces (e.g., `export type ComputeNodeBuilder = ...`).
   - Use `UPPER_SNAKE_CASE` for constants.
   - Prefix unused variables with an underscore (e.g., `_key`, `_`).
5. **Modern Syntax:**
   - Use compound assignment operators (`+=`, `-=`, `*=`, `/=`, `%=`, `^=`, `..=`) for concise code.
   - Prefer if-then-else expressions (`local val = if condition then a else b`) over the traditional `a and b or c` logical operator idiom to avoid unexpected truthiness bugs.
   - Utilize string interpolation (`` `Hello {"world"}` ``) instead of string concatenation (`..`) for complex strings to enhance readability.
6. **Iteration:** Use generalized iteration (`for k, v in table do`) instead of the older `pairs()` or `ipairs()`. It is optimized in Luau and looks cleaner.
7. **Error Handling:** Use `assert(condition, "error message")` for invariants and conditions that must be true. Use `error("message")` to explicitly fail in invalid states.
8. **Imports/Requires:** Keep all `require()` calls at the top of the file, just below the `--!strict` directive. Utilize path aliases (like `@lib/...`) when available to maintain clean and reliable imports.
