---
title: How to Inspect Variables, Addresses, and Layout in Rust
type: howto
id: "wiki:howto:rust-inspect-variables-and-layout"
status: draft
tags: [rust, debugging, learning, memory-layout]
---

schema_version: 1
state: |-
  ## When to use

  You want to *see* what Rust is doing: where a value lives, how a type is laid out, whether a `Vec` reallocated, what a value holds at a breakpoint, or whether code is unsound. This is a learning and debugging procedure, not a production pattern.

  ## The ladder

  Ask the smallest question first and escalate only when the cheap tool cannot answer it.

  | Question | Tool | Toolchain |
  |---|---|---|
  | What address is this value at? | `{:p}`, `&raw const x`, `std::ptr::addr_of!`, `ptr.addr()` | stable |
  | How is this type laid out? | `size_of`, `align_of`, `size_of_val`, `mem::offset_of!`, `Layout` | stable |
  | What did rustc actually choose? | `-Zprint-type-sizes`, rust-analyzer *View Memory Layout* | nightly for the flag |
  | What did the macro expand to / what asm ran? | `cargo expand`, `cargo asm` | expand nightly, asm stable |
  | What are the values right now? | `tracing`, `eprintln!`, lldb / CodeLLDB | stable |
  | Is the heap buffer moving or growing? | `.capacity()`, `.as_ptr()`, dhat, Instruments | stable |
  | Is this genuinely undefined behaviour? | Miri, ASan / TSan | nightly |

  ## 1. Addresses

  `{:p}` is `core::fmt::Pointer` (stable 1.6). It is implemented for `&T`, `&mut T`, `*const T`, `*mut T`, `Box<T>`, `Rc<T>`, `Arc<T>` and nothing else.

  ```rust
  let x: i32 = 42;
  println!("{:p}", &x);                    // address of x
  println!("{x:p}");                       // error: i32 does not implement Pointer
  println!("{:p}", &x as *const i32);      // same address as &x
  println!("{:p}", &&&&x);                 // different: a temporary slot holding the reference
  let v = vec![1u32, 2];
  println!("{:p}", v.as_ptr());            // Vec has no Pointer impl - use as_ptr()
  ```

  Rules that matter:

  - `addr_of!` (stable 1.51, now soft-deprecated in favour of `&raw const` / `&raw mut`, stable 1.82) exists so a field address can be taken **without forming a reference**; that is required for `#[repr(packed)]` fields and `MaybeUninit`, where `&x as *const T` would first assert alignment and validity.
  - `ptr::eq` compares address **plus metadata** (1.17); `ptr::addr_eq` compares address only (1.76). Two pointers can share an address with different provenance, so identical numbers do not make them interchangeable.
  - Do not round-trip through `usize` to get an address. `&x as *const T as usize` loses provenance (a strict-provenance violation Miri can flag); use `ptr.addr()` (stable 1.84). Fat pointers cannot be cast to `usize` at all.
  - Printed addresses change every run (ASLR) and are meaningless across platforms or builds. Treat them as opaque tokens for comparison within one run.

  ## 2. Layout

  ```rust
  use std::mem::{size_of, align_of, size_of_val, offset_of};
  println!("{} {} {}", size_of::<i32>(), align_of::<i32>(), offset_of!(R, b));
  println!("{:?}", std::alloc::Layout::new::<Vec<u8>>());
  ```

  - `size_of_val(&x)` is the size of the **pointee**. `size_of_val(sl)` for `sl: &str` is 5 for `"hello"`, while `size_of::<&str>()` is 16.
  - `offset_of!` is stable since 1.77, supports nested paths such as `offset_of!(Outer, inner.field)`, does **not** support array indexing, and needs the field visible at the call site (a private field gives E0616). Enum-variant offsets are still unstable.
  - Layouts are not stable across compilations. Never hardcode an offset; compute it.
  - `repr(Rust)` may reorder fields to reduce padding; `#[repr(C)]` fixes declaration order. Verified: `{a: u8, b: u64, c: u16}` is `a=0 b=8 c=16 size=24` under `repr(C)`, but `a=10 b=0 c=8 size=16` under `repr(Rust)`.

  ## 3. Getting the compiler actual answer

  `-Zprint-type-sizes` (nightly, internal flag with no tracking issue) prints real size, alignment, field order in memory, padding, and - when a niche was found - **no discriminant line**, which is how you confirm `Option<Box<T>>` is 8 bytes.

  ```bash
  CARGO_INCREMENTAL=0 cargo +nightly rustc -p wm-core --lib -- -Zprint-type-sizes 2>&1 | rg -A20 'type: `GraphSnapshot`'
  ```

  Gotchas: incremental compilation **suppresses** the output; `cargo check` prints nothing because codegen is required; `RUSTFLAGS=-Zprint-type-sizes` invalidates the entire build cache; and the crate has to actually be built.

  `cargo expand` (nightly; wraps `-Zunpretty=expanded`) shows macro output, and the rust-analyzer *Expand macro recursively* command does the same interactively without nightly. `cargo asm` (cargo-show-asm) works on stable and reports nothing for inlined or unmonomorphised functions.

  ## 4. Runtime values and the heap

  - `tracing` is the standard here (`tracing::info!` / `debug!`, with `RUST_LOG`). Raw pointers print via `?`; `{:p}` is not a `tracing` formatter.
  - lldb: `frame variable` prints locals with the toolchain pretty-printers (`Vec`, `String` and `Arc` render collapsed), `p <expr>`, `memory read -c N -f x -s SZ <addr-expr>`, `watchpoint set variable x`. In CodeLLDB, `sourceLanguages: ["rust"]` is what loads the formatters. Debug info is required - a breakpoint with no resolved locations silently runs to completion.
  - `.capacity()` is the reliable growth signal; a pointer change is the only proof that a copy happened. Verified: `Vec::with_capacity(1)` then two pushes moved capacity `1 -> 4` with **no** pointer change, because realloc grew in place.
  - dhat (`#[global_allocator]`) or `cargo instruments -t Allocations` answers why a peak allocation is large.

  ## 5. Undefined behaviour

  `cargo +nightly miri test` catches out-of-bounds access, use-after-free, uninitialised reads, invalid discriminants, data races, leaks, and - experimentally - aliasing violations. It is not a specification and explores one execution at a time.

  ## Repo-specific constraints

  These invalidate the generic advice, and are the reason this page exists.

  1. **`dbg!` is effectively banned here.** `Cargo.toml` sets `dbg_macro = "warn"` and rule `no-warnings` makes warnings defects. Use `tracing` or `eprintln!`.
  2. **`no-comments-in-code` forbids explanatory probes.** A `// keep this borrow alive` comment cannot be left in the source, so the reasoning must be expressed in structure.
  3. **`no-allow-attributes`** means no `#[allow(dead_code)]` on scratch inspection code, and `rust-anti-patterns` bans bare `unwrap()` / `expect()`, so tutorial snippets do not transfer verbatim.
  4. **Miri cannot run most of this workspace.** ONNX via `ort`, the tree-sitter C parsers, and the tokio IO driver all hit `unsupported operation: can not call foreign function`. Scope it to `cargo miri test -p wm-core` and gate FFI tests with `#[cfg_attr(miri, ignore)]`.
  5. **LeakSanitizer has no `aarch64-apple-darwin` target.** On Apple Silicon use `ASAN_OPTIONS=detect_leaks=1` or dhat.
  6. **This repo contains zero `unsafe`, zero raw pointers, zero `#[repr(...)]`, and no FFI.** The address and layout material is for reading and learning, not for editing here; address reasoning only pays off at refcount and identity level.
  7. **The default toolchain is nightly with no pin.** There is no `rust-toolchain.toml` and no `rust-version`, so `-Z` flags and Miri work with no setup - and builds are not reproducible.

  ## Where this pays off in this repo

  | Site | Why addresses and refcounts matter |
  |---|---|
  | `apps/wm-core/src/engine/engine_state_mediator.rs:22-27` | Four co-swapped `ArcSwap` fields; a held `Arc` snapshot is what makes lock-free reads sound |
  | `apps/wm-core/src/graph/mod.rs:277,354,404` and `apps/wm-core/src/page/services/page_crud_service.rs:110` | `.rcu()` copy-on-write; refcount above 1 during a swap window is the documented failure mode |
  | `apps/wm-core/src/page/services/page_update_builder_service.rs:15` | Per-path `Arc<Mutex<()>>` - the identity of the lock is the correctness mechanism |
  | `packages/wm-embed/src/services/vector_store_service.rs:14-48` | Two co-swapped maps, including `[u8;32]` hashes, that must stay consistent |
  | `packages/wm-embed/src/services/onnx/mod.rs:101-104` and `packages/wm-code-intel/src/helpers/parser_helper.rs:13-18` | Thread-local `RefCell` ONNX sessions and `OnceLock<Mutex<Parser>>` - ownership directly explains the behaviour |

  ## Verification

  ```bash
  cargo check -p wm-core
  CARGO_INCREMENTAL=0 cargo +nightly rustc -p wm-core --lib -- -Zprint-type-sizes 2>&1 | head
  cargo +nightly miri test -p wm-core
  cargo tree -e features -i ort
  ```

  ## Sources

  - `@wiki/concepts/rust-learning-research-sources`
  - `@wiki/patterns/borrow-errors-order-area-layout`
  - std: `core::ptr`, `core::fmt::Pointer`, `std::mem::offset_of!`, `core::alloc::Layout`, `std::macro::dbg!`
  - Rust Reference: type layout; Rust Performance Book: type sizes; Unstable Book: `print-type-sizes` and sanitizer; Miri README; CodeLLDB MANUAL; rust-analyzer Book
questions:
  - id: task_kind
    type: choice
    instructions: What kind of task does this howto cover?
    options:
    - setup
    - development
    - testing
    - release
    - debugging
    - operations
    - integration
  - id: prerequisites_complete
    type: noul
    instructions: All prerequisites for this howto are fully documented.
  - id: difficulty
    type: score
    instructions: What skill level does this howto require?
    levels:
    - beginner
    - intermediate
    - advanced
    - expert
  - id: has_verification
    type: noul
    instructions: This howto includes a verification step.
answers: {}
