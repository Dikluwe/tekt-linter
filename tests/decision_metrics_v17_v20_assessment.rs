//! Black-box gate for Assessment 0014 (V17--V20).
//!
//! Producer identity: `verifier/v17-v20/0014`.
//! Authorized inputs:
//! - `00_nucleo/assessments/0014-decision-metrics-v17-v20.md`
//! - `00_nucleo/prompts/rules/wildcard-saturation.md`
//!   (SHA-256 `66c502de44ef21880a68fe798c74ef5f3a91b9fe7dd3e925c2722d99d25f6800`)
//!
//! Frozen black-box properties:
//! 1. Every classifier is silent outside Rust, including empty collections.
//! 2. V17 emits one Warning per arm exactly for `has_guard && guard_is_compound`;
//!    all four boolean combinations are covered.
//! 3. V18 emits one Warning per range arm except in path/module components named
//!    `lexer`, `numbering`, or `syntax`; substring and case lookalikes are not exempt.
//! 4. V19 emits one Info exactly when `or_alternatives > 1`, including boundaries
//!    1/2 and `u16::MAX`, and preserves the count in its message/evidence.
//! 5. V20 emits one Info exactly when `pattern_depth > 2`, including boundaries
//!    2/3 and `u8::MAX`, unless the expression is a regular same-type tuple table.
//!    Homogeneous, heterogeneous, catch-all, and near-table cases are distinguished.
//! 6. Two expressions with multiple arms preserve expression/arm order and exact
//!    per-arm cardinality.
//! 7. Each rule is invariant under systematic mutation of every irrelevant IR field.
//! 8. Every emitted diagnostic preserves snippet, path, line, and column; severity,
//!    message, and location are stable under irrelevant-field mutation.
//! 9. Unicode paths and maximum representable counters do not panic or create
//!    spurious diagnostics.
//!
//! SPEC-GAP: the authorized contract names the classifiers and IR fields but does
//! not publish any invocable Rust API: constructor/visibility details for
//! `DecisionExpr`, `DecisionArm`, and `Span`; the input carrying language and
//! path/module identity; classifier signatures; or the diagnostic result type and
//! accessors. A black-box adapter cannot be written without guessing or reading
//! forbidden L1--L4 sources. Keep this gate compile-blocking until the assessment/L0
//! publishes that API (or an independently authorized adapter contract is supplied).

compile_error!(
    "SPEC-GAP Assessment 0014: authorized L0 does not declare an invocable black-box API for V17-V20"
);
