# CLAUDE.md — crystalline-lint

This file provides guidance to Claude Code when working in this repository.
For complete architecture reference, dependency tables, and examples: **read README.md first**.

---

## What This Project Is

`crystalline-lint` is an architectural linter written in Rust that enforces the
Crystalline Architecture standard. It validates its own codebase against its own
rules — zero violations on `cargo run -- .` is the primary correctness check.

---

## Commands

```bash
cargo build                            # Debug build
cargo build --release                  # Release build
cargo test                             # Run all tests
cargo test <module_path>               # Single module (e.g. cargo test rules::prompt_header)
cargo test -- --nocapture              # Show stdout during tests
cargo run -- .                         # Lint current directory
cargo run -- --fix-hashes .            # Rewrite stale @prompt-hash values
cargo run -- --fix-hashes --dry-run .  # Preview hash fixes without writing
cargo run -- --format sarif .          # SARIF output for GitHub Code Scanning
```

**Primary correctness check:**
```bash
cargo run -- .
# ✓ No violations found
```

---

## Mandatory Agent Protocol (Nucleation Lock)

Before writing ANY code in L1–L4:

1. Inspect `00_nucleo/prompts/` for the prompt corresponding to the component
2. **Prompt exists** → read it fully (context, constraints, verification criteria, revision history)
3. **No prompt exists** → STOP. Do not write code. Propose a structured prompt to the developer first
4. Generate code **and tests simultaneously** from the prompt's verification criteria
5. Log the revision in the prompt's history (date, reason, affected files)
6. Run `--fix-hashes` after editing any prompt in `00_nucleo/`

A nucleation is incomplete without a corresponding test. A component without
a prompt in L₀ is structurally illegitimate, even if functionally correct.

---

## Mandatory Lineage Header

Every file created or edited in L1–L4 must begin with:

```rust
//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/<name>.md
//! @prompt-hash <sha256[0..8]>
//! @layer L<n>
//! @updated YYYY-MM-DD
```

---

## Rust Constraints — All Layers

These are hard constraints. Reject code that does not follow them.

### Borrowing (Mandatory)

Prefer borrowed references over owned values. No `clone()` without a justifying
comment. No `Rc<RefCell<T>>` anywhere — use the borrow checker, not runtime
workarounds.

```rust
// ✅ Correct
fn check_v1(files: &[ParsedFile]) -> Vec<Violation> { ... }

// ❌ Wrong — unnecessary ownership transfer
fn check_v1(files: Vec<ParsedFile>) -> Vec<Violation> { ... }
```

### Lifetimes (Mandatory)

Explicit lifetimes when the compiler requires them. No lifetime elision that
hides structural relationships. No `'static` to avoid thinking about lifetimes.

### Enums over Booleans and Partial Options (Mandatory)

Illegal states must be unrepresentable. Replace `is_valid: bool` +
`error_message: Option<String>` — a struct that can be simultaneously valid
and carry an error — with an enum that makes the contradiction impossible.

```rust
// ✅ Correct — invalid state is unrepresentable
enum FileStatus {
    Valid,
    Invalid(String), // error only exists when state is Invalid
}

// ❌ Wrong — allows is_valid: true WITH an error message
struct FileStatus {
    is_valid: bool,
    error_message: Option<String>,
}
```

### Parse, Don't Validate (Mandatory)

Functions must not receive raw data and return `bool`. They must receive raw
data and return a new validated type. Downstream functions that receive the
validated type need no further validation — the type itself is the proof.

```rust
// ✅ Correct — the type proves validation already occurred
fn parse_header(raw: &str) -> Result<ValidatedHeader, HeaderError> { ... }
fn apply_rule(file: &ValidatedFile) -> Vec<Violation> { ... }

// ❌ Wrong — forces callers to validate in the middle of business logic
fn is_valid_header(raw: &str) -> bool { ... }
fn apply_rule(file: &ParsedFile) -> Vec<Violation> {
    if !file.has_valid_header() { ... } // validation leak
}
```

> If you need an `if` or `match` to check data validity inside business logic,
> the type system failed at an earlier boundary.

### Newtype Pattern for Domain Primitives (Mandatory)

Wrap primitives that represent domain concepts in single-field structs.
Prevents passing a `String` where a `FilePath` is expected, or a bare `f32`
where a `CoverageValue` is required.

```rust
// ✅ Correct
struct CoverageValue(f32);
struct FilePath(PathBuf);
struct PromptHash(String);

// ❌ Wrong — caller can pass any f32 or String
fn check_coverage(value: f32, path: String) { ... }
```

---

## Rust Constraints — By Layer

### L1 — Core (Pure Logic)

**Absolute:** zero I/O, zero external crates (stdlib only).

| Pattern | Rule |
|---------|------|
| Errors | `thiserror` with typed enums — no `anyhow`, no `Box<dyn Error>` |
| Types | Traits and enums (ADTs) for all domain modeling |
| Concurrency | None — L1 is stateless and pure |
| State | No `Mutex`, `Arc`, `Atomic`, `RefCell` |
| Traits | Sealed when they define contracts not meant for external implementation |
| Typestate | Use for multi-step operations where order must be enforced at compile time |

**Sealed Traits** prevent external crates from implementing contract traits with
unexpected behavior. Apply to traits in `contracts/` that define L3 boundaries.

```rust
// ✅ Correct — sealed trait, external implementation impossible
mod private { pub trait Sealed {} }
pub trait FileProvider: private::Sealed {
    fn files(&self) -> &[ParsedFile];
}
```

**Typestate** for operations with mandatory ordering:

```rust
// ✅ Correct — cannot call apply_rules on an unparsed file
struct Unparsed;
struct Parsed;
struct ParsedFile<State> { inner: RawFile, _state: PhantomData<State> }

fn parse(raw: ParsedFile<Unparsed>) -> Result<ParsedFile<Parsed>, ParseError> { ... }
fn apply_rules(file: &ParsedFile<Parsed>) -> Vec<Violation> { ... }
```

### L2 — Shell (CLI & Formatting)

| Pattern | Rule |
|---------|------|
| Errors | `anyhow` permitted for CLI error propagation |
| Types | Enums for output formats (`Text`, `Sarif`) |
| Concurrency | None — sequential CLI execution |
| Sealed Traits | Not required |
| Typestate | Not required |
| Imports | L1 only — never L3 |

### L3 — Infra (I/O Implementations)

| Pattern | Rule |
|---------|------|
| Errors | `thiserror` — typed I/O errors that map to L1 domain errors |
| Types | Implements L1 traits — no new domain types |
| Concurrency | `Arc<Mutex<T>>` or channels permitted for parallel file walking |
| Sealed Traits | Not applicable — implements, does not define |
| Typestate | Not required |
| Imports | L1 only — never L2 or L4 |

```rust
// ✅ Correct — concurrency contained in L3
pub struct ParallelFileWalker {
    results: Arc<Mutex<Vec<ParsedFile>>>,
}
```

### L4 — Wiring (Composition Root)

| Pattern | Rule |
|---------|------|
| Errors | `anyhow` for top-level propagation |
| Logic | Near-zero — any business `if/else` here is a structural defect |
| Concurrency | Thread spawning permitted here only |
| Imports | All layers except Lab |

---

## Test Atomization Rules

One test per behavior. No shared setup. No test depends on another test's output.
Tests use mock structs implementing L1 traits — never real filesystem or I/O.

```rust
// ✅ Correct — one behavior per test, mock-based
struct MockFileProvider { files: Vec<ParsedFile> }
impl FileProvider for MockFileProvider { ... }

#[test]
fn v1_flags_file_without_prompt_header() {
    let provider = MockFileProvider { files: vec![file_without_header()] };
    assert_eq!(check_v1(&provider).len(), 1);
}

#[test]
fn v1_passes_file_with_valid_prompt_header() {
    let provider = MockFileProvider { files: vec![file_with_valid_header()] };
    assert!(check_v1(&provider).is_empty());
}

// ❌ Wrong — multiple behaviors, real I/O
#[test]
fn test_v1_all_cases() {
    let files = std::fs::read_dir("fixtures/").unwrap();
    // ...
}
```

Tests are co-located in the same file using `#[cfg(test)]` blocks.
Never separate `_test.rs` files.

---

## Workflows

Structured operations live in `.agents/workflows/`:

| Workflow | Purpose |
|----------|---------|
| `init-legado.md` | Initialize a legacy project for Tekt migration |
| `gerar-spec.md` | Generate a new L₀ structured prompt |
| `integrar-legado.md` | Refactor legacy file into the appropriate layer |
| `auditar-spec.md` | Audit prompt quality and completeness |
| `clivar-modulo.md` | Split a large module across appropriate layers |

---

## Reference

For complete layer descriptions, dependency diagrams, violation definitions (V1–V6),
and `crystalline.toml` configuration: **README.md**
