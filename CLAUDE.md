# Claude Project Instructions

## Workflow

- After generating new code and correcting any cargo check errors and warnings:
  1. Run cargo test and clear all errors.
  2. Run cargo clippy and clear all warnings.
  3. Commit the changes to git using best practices for code auditing.
  4. Push the changes to their respective github branch.
- Avoid running cargo clean often, to take advantage of incremental compilation during development.

## Linting

- When running any linter (e.g. clippy or markdownlint), rather than deny all warnings, let them complete so you can fix them all in a single pass.
- After editing a markdown file, run markdownlint and either fix the error or add an exception, as appropriate in the context.
- Do not run cargo clippy or cargo test after changes to markdown files, as they don't affect the Rust code.

## API structure

- In lib.rs, export the visibility of all types at the root level with pub use statements.
  - Keep the mod statements private so there is only one way for users to import the type.
  - In modules, import types from the crate level with use crate::{type1, type2} statements.

## Derive Policies

- Data structures should derive Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, and Hash if possible.
- Use derive_more to derive Display, FromStr, From, Deref, DerefMut, AsRef, and AsMut when appropriate.
- For enums with no fields, use strum to derive EnumIter.

## Serialization

- Derive `Serialize` and `Deserialize` for types that need to be persisted or transmitted (project state, configuration, etc.).
- Use `#[serde(skip)]` for fields that should not be serialized (runtime state, caches, UI state, texture handles).
- Use `#[serde(default)]` for fields that should use their `Default` value when missing during deserialization.
- Use `#[serde(default = "function_name")]` to specify a custom default function for a field.
- Use `#[serde(rename = "name")]` when the serialized field name should differ from the Rust field name.
- Group related `#[serde(skip)]` attributes with comments explaining why they're not serialized (e.g., "// Runtime state (not serialized)").
- For complex serialization needs, implement custom `Serialize`/`Deserialize` instead of using derives.

## Feature Flags

- Use `#[cfg(feature = "feature-name")]` to conditionally compile code based on features.
- Document feature-gated public APIs with a note in the documentation: `/// Available with the`feature-name`feature.`
- Available features:
  - `backend-eframe` - eframe/wgpu rendering backend (enabled by default)
  - `text-detection` - OpenCV-based text detection
  - `logo-detection` - OpenCV-based logo detection
  - `ocr` - Tesseract-based OCR text extraction
  - `dev` - Enables all optional features for development
- When adding new feature-gated code, ensure the crate still compiles with only default features.
- Use `cargo check --no-default-features` to verify the crate works without optional features.
- Use `cargo check --all-features` to verify all features compile together.

## Documentation

- Use `///` for item documentation (functions, structs, enums, fields, methods).
- Use `//!` for module-level documentation at the top of files.
- All public types, functions, and methods must have documentation (enforced by `#![warn(missing_docs)]`).
- Document:
  - **What** the item does (concise first line)
  - **Why** it exists or when to use it (for non-obvious cases)
  - **Parameters and returns** for functions (when not obvious from types)
  - **Examples** for complex APIs or non-obvious usage
  - **Errors** that can be returned (for Result-returning functions)
- Keep documentation concise but informative - avoid stating the obvious from the signature.

## Logging and Tracing

- Use the `tracing` crate for all logging (never `println!` in library code).
- Choose appropriate log levels:
  - `trace!()` - Very detailed, fine-grained information (loop iterations, individual calculations)
  - `debug!()` - General debugging information (function entry/exit, state changes)
  - `info!()` - Important runtime information (initialization, major events)
  - `warn!()` - Warnings about unusual but recoverable conditions
  - `error!()` - Errors that should be investigated
- Use structured logging with fields: `debug!(count = items.len(), "Processing items")`
- Use `#[instrument]` macro on functions for automatic entry/exit logging with arguments
- Use `?` prefix for Debug formatting in field values: `debug!(value = ?self.field())`
- Binary applications can use `println!` for user-facing output, but use `tracing` for diagnostics

## Testing

- Do not place mod tests in the module next to the code. Place unit tests in the tests directory.

## Error Handling

- Use unique error types for different sources to create encapsulation around error conditions for easier isolation.
  - For specific errors types capturing initial error condition, wrap enums in a struct that include the line and file where the error occurred using the line! and file! macros.
  - The idiom is to call the enumeration something like MyErrorKind, and the wrapper struct MyError.
  - The idiom for MyError is to have fields kind, line and file.
  - Error struct `file` fields should use `&'static str` (not `String`) to match the return type of the `file!()` macro, reducing allocations.
  - Omit the enum type and kind field when a static message conveys sufficient information, but still include the line and file.
  - Implement a specific error message in the display impl for each variant of the enum, then wrap this msg in the display impl for the wrapper. E.g. If the display for MyErrorKind is e, then MyError displays "My Error: {e} at line {line} in {file}" so the user can see the whole context.
  - Use the derive_more crate to implement Display and Error when convenient.
  - Expand and improve error structs and enums as necessary to capture sufficient information about the error conditions to gain insight into the nature of the problem.
- After creating a new unique error type, add a variant to the crate level error enum using the new error name as a variant type, including the new error type as a field (e.g. `FormErrorKind::Canvas(CanvasError)`)
  - Use `#[derive(Debug, derive_more::From)]` on the crate-level error enum to automatically generate From implementations for all error variants.
  - The display impl for the crate-level enum should forward the impl from the original error (e.g. If the display value of NewError is e, then the display for CrateErrorKind is "{e}").
  - The display impl for the wrapper struct around the crate-level enum should include the display value of its kind field (e.g. If the display value of CrateErrorKind is e, then CrateError displays "Form Error: {e}").
- If a function or method returns a single unique error type, use that type. If the body contains more than one error type in its result types, convert the unique error types to the crate level type, and use the crate level error in the return type of the function or method signature.

### Error Handling Example

```rust
// Module-level error
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasErrorKind {
    ImageLoad(String),
    NoFormImageLoaded,
}

impl std::fmt::Display for CanvasErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CanvasErrorKind::ImageLoad(msg) => write!(f, "Failed to load image: {}", msg),
            CanvasErrorKind::NoFormImageLoaded => write!(f, "No form image loaded"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CanvasError {
    pub kind: CanvasErrorKind,
    pub line: u32,
    pub file: &'static str,
}

impl std::fmt::Display for CanvasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Canvas Error: {} at line {} in {}", self.kind, self.line, self.file)
    }
}

impl std::error::Error for CanvasError {}

// Crate-level error
#[derive(Debug, derive_more::From)]
pub enum FormErrorKind {
    Canvas(CanvasError),
    // ... other variants
}

impl std::fmt::Display for FormErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormErrorKind::Canvas(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Debug)]
pub struct FormError(Box<FormErrorKind>);

impl std::fmt::Display for FormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Form Error: {}", self.0)
    }
}

impl From<CanvasError> for FormError {
    fn from(err: CanvasError) -> Self {
        FormError(Box::new(FormErrorKind::from(err)))
    }
}
```

## Module Organization

### Module Structure

- When a module file exceeds ~500-1000 lines, consider splitting it into a module directory with focused submodules organized by responsibility (e.g., core, io, tools, rendering).
- Create a mod.rs file to re-export the public API and keep internal organization private.
- Only put mod and export statements in the mod.rs file, not types, traits or impl blocks.

### Visibility and Export Patterns

**Module declarations:**
- Use private `mod` declarations (not `pub mod`) in both lib.rs and module mod.rs files
- Keep internal module structure hidden from external users

```rust
// src/lib.rs or src/mymodule/mod.rs
mod error;           // Private module
mod models;          // Private module
mod internal_helper; // Private module
```

**Module-level exports (mod.rs):**
- Re-export public types from submodules using `pub use`
- This creates the public API for the module

```rust
// src/mymodule/mod.rs
mod error;
mod models;
mod helper;

pub use error::{MyError, MyErrorKind, MyResult};
pub use models::{Model, NewModel, ModelRow};
// helper module stays private, not exported
```

**Crate-level exports (lib.rs):**
- Re-export ALL public types from all modules at the crate root
- This ensures a single, consistent import path throughout the codebase

```rust
// src/lib.rs
mod mymodule;

pub use mymodule::{
    Model, MyError, MyErrorKind, MyResult, NewModel, ModelRow,
};
```

### Import Patterns

**For crate-level types (exported from lib.rs):**
- Always use `use crate::{Type1, Type2}` syntax
- Never use module paths like `crate::module::Type`
- Never use `super::` paths
- Never use wildcard imports like `use module::*`

```rust
// ✅ GOOD: Import from crate root
use crate::{Model, MyError, MyResult};

// ❌ BAD: Module path imports
use crate::mymodule::Model;

// ❌ BAD: Super paths
use super::models::Model;

// ❌ BAD: Wildcard imports
use crate::mymodule::*;
```

**For internal module helpers (not exported at crate level):**
- Use explicit module paths: `use crate::module::helper::function`
- For schema tables or module-private items: `use crate::module::schema::table_name`

```rust
// ✅ GOOD: Internal helper functions
use crate::database::schema::{users, posts};
use crate::database::conversions::{row_to_model, model_to_row};
```

### Complete Example

```rust
// src/database/mod.rs
mod error;
mod models;
mod conversions;  // Internal helpers
mod schema;       // Diesel schema

pub use error::{DatabaseError, DatabaseErrorKind, DatabaseResult};
pub use models::{User, NewUser, UserRow};

// src/lib.rs
mod database;

pub use database::{
    DatabaseError, DatabaseErrorKind, DatabaseResult,
    User, NewUser, UserRow,
};

// src/database/conversions.rs
use crate::{User, UserRow, DatabaseResult};  // Crate-level types
use crate::database::schema::users;          // Internal schema

pub fn row_to_user(row: UserRow) -> DatabaseResult<User> {
    // ...
}

// src/database/repository.rs
use crate::{User, UserRow, DatabaseResult};  // Crate-level types
use crate::database::conversions::row_to_user;  // Internal helper
use crate::database::schema::users;             // Internal schema
```

### Benefits

This pattern provides:
1. **Single import path** - All types imported as `use crate::{Type}`
2. **No ambiguity** - Only one way to import each type
3. **Clean public API** - Internal module structure is hidden
4. **Easier refactoring** - Module reorganization doesn't break imports
5. **Better IDE support** - Auto-completion works consistently

### Cross-Module Communication

- Add helper methods (setters, mut accessors) to core structs for clean cross-module communication instead of directly accessing fields.

## Common Refactoring Patterns

- **State Machine Extraction**: When multiple boolean flags represent mutually exclusive states, extract them into an enum state machine to prevent invalid state combinations.
- **Borrow Checker**: When encountering borrow checker errors with simultaneous immutable and mutable borrows, extract needed values before taking mutable references (e.g., `let value = *self.field(); /* then mutably borrow */`).

## Unsafe

- Use the forbid unsafe lint at the top level of lib.rs to prevent unsafe code.
