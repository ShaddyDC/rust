# `place_macros` (Place Expression Desugaring & Handle Testing Tool)

This crate is an in-tree compiler tool (`src/tools/place_macros`) for developing,
testing, and validating place expression desugaring and place handle semantics
for the Rust **Field Projections** language feature proposal (tracking issue
[#145383](https://github.com/rust-lang/rust/issues/145383)).

## Motivation & Architecture

In Rust, a **place** represents a specific memory location, syntactically
denoted by a *place expression*:
- `$path` (local variable, parameter, static)
- `*$place` (dereference)
- `$place.$field` (field access)
- `$place[$expr]` (indexing)
- `($place)` (parentheses)

Under the Field Projections design, any place expression is desugared into a
**place handle** (`PlaceHandle`). Operations on places are performed through
traits implemented on these handles:
- [`LocalHandle`]: Points to local or static memory (`&raw mut` / `&raw const`).
- [`ProjectPlace`]: Projects a handle into a struct subplace (`place.field`).
- [`DerefPlace`]: Dereferences a pointer in a place (`*place`).
- [`IndexPlace`]: Indexes into a place (`place[idx]`).

### Integration with Compiler & Core Traits

This tool aligns directly with the place operations traits defined in
[`core::ops::place`](library/core/src/ops/place.rs):
- `PlaceProxy`: Proxy types (`&T`, `&mut T`, `*const T`, `*mut T`, `Box<T>`).
- `PlaceHandle`: Handle identifying a place in memory.
- `DerefPlace`: `*place` navigation.
- `ProjectPlace<S>`: `place.field` navigation via a `Subplace`.
- `Subplace`: Subplace metadata computing byte offset and target metadata:
  ```rust,ignore (illustrative signature)
  fn offset(
      self,
      metadata: <Self::Source as Pointee>::Metadata,
  ) -> (usize, <Self::Target as Pointee>::Metadata);
  ```

The desugaring targets the canonical traits in
[`core::ops::place`](library/core/src/ops/place.rs), without maintaining
redundant copies in this crate.

### Place Expressions vs. Terminal Place Operations

In the broader Field Projections architecture, place operations are divided into
two distinct categories:

1. **Place Expressions (Handled by this crate)**:
   Recursive operations that *form* or *navigate* to a place in memory,
   producing a `PlaceHandle`:
   - Paths (`LocalHandle`)
   - Dereferences (`DerefPlace`)
   - Field projections (`ProjectPlace`)
   - Indexing (`IndexPlace`)

2. **Terminal Operations (Not needed here)**:
   Operations that *consume* or *act on* the resulting `PlaceHandle`:
   - Borrowing (`&place`, `&mut place`) $\to$ `BorrowPlace::<Output>::borrow(handle)`
   - Writing / Assignment (`place = val`) $\to$ `WritePlace::write_place(handle, val)`
   - Reading / Moving (`let x = place`) $\to$ `ReadPlace::read_place(handle)`
   - Dropping (`drop(place)`) $\to$ `DropPlace::drop_place(handle)`

`BorrowPlace`, `WritePlace`, and `ReadPlace` are terminal actions emitted by
the compiler *after* the place expression has been fully evaluated. They do not
form place expressions, and therefore are not needed in `raw_handle!` or `type_of!`.

### `LocalHandle` Tradeoff & Intermediate Scope

`LocalHandle` is currently provided as a unified concrete handle to local memory.
This reflects a deliberate architectural tradeoff for this intermediate phase:

- **Intermediate scope**: In this milestone, `LocalHandle<T>` stores a `*mut T`
  and fulfills both shared and exclusive handle requirements. Static separation of
  permissions is deferred; callers must satisfy `unsafe` preconditions when
  performing mutations via `.as_mut_ptr()`.
- **Target design**: In the complete Field Projections implementation, handle types
  are partitioned by capability (`MutHandle<'a, T>`, `RefHandle<'a, T>`), and
  the borrow checker statically verifies access permissions using the terminal
  operation traits (`BorrowPlace`, `WritePlace`, `ReadPlace`).

## Provided Macros

### 1. `raw_handle!`
Recursively transforms a place expression into its corresponding handle chain:
- `$path` $\to$ `LocalHandle::new(&raw {const,mut} $path)`
- `*$place` $\to$ `DerefPlace::deref_place(raw_handle!($place))`
- `$place.$field` $\to$
  `ProjectPlace::<field_of!(type_of!($place), $field)>::project_place(...)`
- `$place[$expr]` $\to$ `IndexPlace::index_place(raw_handle!($place), $expr)`
- `($place)` $\to$ `raw_handle!($place)`

**Syntax options:**
- `raw_handle!(place.field)`: defaults to mutable `&raw mut`.
- `raw_handle!(mut place.field)` / `raw_handle!(const place.field)`: explicit mutability.
- `raw_handle!(var: Type => var.field)`: supplies the root type for executable Rust code.

### 2. `type_of!`
Computes the type representation of a place:
- `$path` $\to$ `typeof($path)` (or the bound type)
- `*$place` $\to$ `<type_of!($place) as PlaceProxy>::Target`
- `$place.$field` $\to$ `<field_of!(type_of!($place), $field) as Subplace>::Target`
- `$place[$expr]` $\to$ `<type_of!($place) as Indexable<typeof($expr)>>`
- `($place)` $\to$ `type_of!($place)`

### 3. Inspection & Diagnostics Macros
- `desugar_raw_handle!(expr)`: Produces a formatted string literal of the desugared handle.
- `desugar_type_of!(expr)`: Produces a formatted string literal of the desugared type.
- `desugar_place!(expr)`: Produces a comprehensive summary showing both desugarings.

## Running Tests & Showcase

As a member of the root compiler workspace, `place_macros` can be built and
tested from anywhere in the tree:

```bash
# Run place_macros unit & integration tests
cargo test -p place_macros

# Run the interactive showcase demo
cargo run -p place_macros --example showcase

# Run clippy checks
cargo clippy -p place_macros --all-targets -- -D warnings
```
