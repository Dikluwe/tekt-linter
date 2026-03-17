//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/prompt-stale.md
//! @prompt-hash 5c27ee71
//! @layer L1
//! @updated 2026-03-14

use std::borrow::Cow;

use crate::entities::rule_traits::HasPublicInterface;
use crate::entities::parsed_file::{
    FunctionSignature, InterfaceDelta, PublicInterface, TypeSignature,
};
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V6 — PromptStale
///
/// Detects when the public interface of a source file has changed since the
/// last snapshot registered in the origin prompt. Pure L1 function — zero I/O.
pub fn check<'a, T: HasPublicInterface<'a>>(file: &T) -> Vec<Violation<'a>> {
    // V6 only applies to files that have a prompt header
    let header = match file.prompt_header() {
        Some(h) => h,
        None => return vec![], // V1 covers missing header
    };

    // Without a baseline snapshot there is nothing to compare against
    let snapshot = match file.prompt_snapshot() {
        Some(s) => s,
        None => return vec![], // first generation — no history yet
    };

    let current = file.public_interface();

    if current == snapshot {
        return vec![];
    }

    let delta = compute_delta(current, snapshot);

    if delta.is_empty() {
        return vec![];
    }

    vec![Violation {
        rule_id: "V6".to_string(),
        level: ViolationLevel::Warning,
        message: format!(
            "Prompt potencialmente desatualizado: interface pública mudou \
             desde a última revisão de '{}'. Delta: {}",
            header.prompt_path,
            delta.describe()
        ),
        location: Location { path: Cow::Borrowed(file.path()), line: 1, column: 0 },
    }]
}

/// Computa diferença entre interface atual e snapshot do prompt.
/// Usa PartialEq completo sobre FunctionSignature e TypeSignature —
/// name + params + return_type devem ser todos iguais.
/// Mudança de assinatura aparece como remoção + adição.
pub fn compute_delta<'a>(
    current: &PublicInterface<'a>,
    snapshot: &PublicInterface<'a>,
) -> InterfaceDelta<'a> {
    InterfaceDelta {
        added_functions: added_fns(&current.functions, &snapshot.functions),
        removed_functions: added_fns(&snapshot.functions, &current.functions),
        added_types: added_types(&current.types, &snapshot.types),
        removed_types: added_types(&snapshot.types, &current.types),
        added_reexports: added_strs(&current.reexports, &snapshot.reexports),
        removed_reexports: added_strs(&snapshot.reexports, &current.reexports),
    }
}

fn added_fns<'a>(
    a: &[FunctionSignature<'a>],
    b: &[FunctionSignature<'a>],
) -> Vec<FunctionSignature<'a>> {
    a.iter().filter(|f| !b.contains(f)).cloned().collect()
}

fn added_types<'a>(
    a: &[TypeSignature<'a>],
    b: &[TypeSignature<'a>],
) -> Vec<TypeSignature<'a>> {
    a.iter().filter(|t| !b.contains(t)).cloned().collect()
}

fn added_strs<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<&'a str> {
    a.iter().filter(|s| !b.contains(s)).copied().collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::{Language, Layer};
    use crate::entities::parsed_file::{
        FunctionSignature, ParsedFile, PromptHeader, PublicInterface, TypeKind, TypeSignature,
    };
    use std::path::Path;

    fn base_file() -> ParsedFile<'static> {
        ParsedFile {
            path: Path::new("01_core/foo.rs"),
            layer: Layer::L1,
            language: Language::Rust,
            prompt_header: Some(PromptHeader {
                prompt_path: "00_nucleo/prompts/foo.md",
                prompt_hash: None,
                current_hash: None,
                layer: Layer::L1,
                updated: None,
            }),
            prompt_file_exists: true,
            has_test_coverage: true,
            imports: vec![],
            tokens: vec![],
            public_interface: PublicInterface::empty(),
            prompt_snapshot: None,
            declared_traits: vec![],
            implemented_traits: vec![],
            declarations: vec![],
        }
    }

    fn fn_sig(name: &'static str) -> FunctionSignature<'static> {
        FunctionSignature { name, params: vec![], return_type: None }
    }

    fn type_sig(name: &'static str) -> TypeSignature<'static> {
        TypeSignature { name, kind: TypeKind::Struct, members: vec![] }
    }

    #[test]
    fn no_snapshot_returns_empty() {
        let file = base_file();
        assert!(check(&file).is_empty());
    }

    #[test]
    fn identical_interface_returns_empty() {
        let iface = PublicInterface {
            functions: vec![fn_sig("check")],
            types: vec![],
            reexports: vec![],
        };
        let mut file = base_file();
        file.public_interface = iface.clone();
        file.prompt_snapshot = Some(iface);
        assert!(check(&file).is_empty());
    }

    #[test]
    fn added_function_generates_v6() {
        let mut file = base_file();
        file.public_interface = PublicInterface {
            functions: vec![fn_sig("check"), fn_sig("validate")],
            types: vec![],
            reexports: vec![],
        };
        file.prompt_snapshot = Some(PublicInterface {
            functions: vec![fn_sig("check")],
            types: vec![],
            reexports: vec![],
        });
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V6");
        assert!(violations[0].message.contains("+fn validate"));
    }

    #[test]
    fn removed_function_generates_v6() {
        let mut file = base_file();
        file.public_interface = PublicInterface {
            functions: vec![fn_sig("check")],
            types: vec![],
            reexports: vec![],
        };
        file.prompt_snapshot = Some(PublicInterface {
            functions: vec![fn_sig("check"), fn_sig("old_fn")],
            types: vec![],
            reexports: vec![],
        });
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("-fn old_fn"));
    }

    #[test]
    fn no_prompt_header_returns_empty() {
        let mut file = base_file();
        file.prompt_header = None;
        file.prompt_snapshot = Some(PublicInterface::empty());
        assert!(check(&file).is_empty());
    }

    #[test]
    fn added_type_generates_v6() {
        let mut file = base_file();
        file.public_interface = PublicInterface {
            functions: vec![],
            types: vec![type_sig("Foo")],
            reexports: vec![],
        };
        file.prompt_snapshot = Some(PublicInterface::empty());
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("+type Foo"));
    }

    #[test]
    fn signature_change_generates_v6_with_both_entries() {
        // foo(a: String) -> bool  →  foo(a: Vec<String>) -> bool
        // Same name but different params — full PartialEq detects the change.
        // Delta must contain -fn foo (removed) AND +fn foo (added).
        let old_sig = FunctionSignature {
            name: "foo",
            params: vec!["a: String"],
            return_type: Some("bool"),
        };
        let new_sig = FunctionSignature {
            name: "foo",
            params: vec!["a: Vec<String>"],
            return_type: Some("bool"),
        };
        let mut file = base_file();
        file.public_interface =
            PublicInterface { functions: vec![new_sig], types: vec![], reexports: vec![] };
        file.prompt_snapshot =
            Some(PublicInterface { functions: vec![old_sig], types: vec![], reexports: vec![] });
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V6");
        let msg = &violations[0].message;
        assert!(msg.contains("+fn foo"), "delta deve conter +fn foo, got: {msg}");
        assert!(msg.contains("-fn foo"), "delta deve conter -fn foo, got: {msg}");
    }

    #[test]
    fn delta_describe_formats_correctly() {
        let delta = InterfaceDelta {
            added_functions: vec![fn_sig("new_fn")],
            removed_functions: vec![],
            added_types: vec![],
            removed_types: vec![type_sig("OldType")],
            added_reexports: vec![],
            removed_reexports: vec![],
        };
        let desc = delta.describe();
        assert!(desc.contains("+fn new_fn"));
        assert!(desc.contains("-type OldType"));
    }

    #[test]
    fn delta_is_empty_when_no_changes() {
        let delta = InterfaceDelta {
            added_functions: vec![],
            removed_functions: vec![],
            added_types: vec![],
            removed_types: vec![],
            added_reexports: vec![],
            removed_reexports: vec![],
        };
        assert!(delta.is_empty());
    }
}
