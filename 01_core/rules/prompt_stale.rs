//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/prompt-stale.md
//! @prompt-hash 00000000
//! @layer L1
//! @updated 2026-03-14

use crate::entities::parsed_file::{
    FunctionSignature, InterfaceDelta, ParsedFile, PublicInterface, TypeSignature,
};
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V6 — PromptStale
///
/// Detects when the public interface of a source file has changed since the
/// last snapshot registered in the origin prompt. Pure L1 function — zero I/O.
pub fn check(file: &ParsedFile) -> Vec<Violation> {
    // V6 only applies to files that have a prompt header
    let header = match &file.prompt_header {
        Some(h) => h,
        None => return vec![], // V1 covers missing header
    };

    // Without a baseline snapshot there is nothing to compare against
    let snapshot = match &file.prompt_snapshot {
        Some(s) => s,
        None => return vec![], // first generation — no history yet
    };

    if &file.public_interface == snapshot {
        return vec![];
    }

    let delta = compute_delta(&file.public_interface, snapshot);

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
        location: Location {
            path: file.path.clone(),
            line: 1,
            column: 0,
        },
    }]
}

/// Compute a structural diff between the current interface and the snapshot.
pub fn compute_delta(current: &PublicInterface, snapshot: &PublicInterface) -> InterfaceDelta {
    InterfaceDelta {
        added_functions: added_fns(&current.functions, &snapshot.functions),
        removed_functions: added_fns(&snapshot.functions, &current.functions),
        added_types: added_types(&current.types, &snapshot.types),
        removed_types: added_types(&snapshot.types, &current.types),
        added_reexports: added_strs(&current.reexports, &snapshot.reexports),
        removed_reexports: added_strs(&snapshot.reexports, &current.reexports),
    }
}

fn added_fns(a: &[FunctionSignature], b: &[FunctionSignature]) -> Vec<FunctionSignature> {
    a.iter().filter(|f| !b.contains(f)).cloned().collect()
}

fn added_types(a: &[TypeSignature], b: &[TypeSignature]) -> Vec<TypeSignature> {
    a.iter().filter(|t| !b.contains(t)).cloned().collect()
}

fn added_strs(a: &[String], b: &[String]) -> Vec<String> {
    a.iter().filter(|s| !b.contains(s)).cloned().collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::{Language, Layer};
    use crate::entities::parsed_file::{
        FunctionSignature, PromptHeader, PublicInterface, TypeKind, TypeSignature,
    };
    use std::path::PathBuf;

    fn base_file() -> ParsedFile {
        ParsedFile {
            path: PathBuf::from("01_core/foo.rs"),
            layer: Layer::L1,
            language: Language::Rust,
            prompt_header: Some(PromptHeader {
                prompt_path: "00_nucleo/prompts/foo.md".to_string(),
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
        }
    }

    fn fn_sig(name: &str) -> FunctionSignature {
        FunctionSignature { name: name.to_string(), params: vec![], return_type: None }
    }

    fn type_sig(name: &str) -> TypeSignature {
        TypeSignature { name: name.to_string(), kind: TypeKind::Struct, members: vec![] }
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
    fn signature_change_generates_v6_with_both_entries() {
        // foo(a: String) -> bool  →  foo(a: Vec<String>) -> bool
        // Same name but different params — full PartialEq detects the change.
        // Delta must contain -fn foo (removed) AND +fn foo (added).
        let old_sig = FunctionSignature {
            name: "foo".to_string(),
            params: vec!["a: String".to_string()],
            return_type: Some("bool".to_string()),
        };
        let new_sig = FunctionSignature {
            name: "foo".to_string(),
            params: vec!["a: Vec<String>".to_string()],
            return_type: Some("bool".to_string()),
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
