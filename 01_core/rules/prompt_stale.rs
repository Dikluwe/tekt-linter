//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/prompt-stale.md
//! @prompt-hash 4f4edb28
//! @layer L1
//! @updated 2026-03-14

use std::borrow::Cow;

use crate::entities::parsed_file::{
    FunctionSignature, InterfaceDelta, PublicInterface, TypeSignature,
};
use crate::entities::rule_traits::HasPublicInterface;
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
        location: Location {
            path: Cow::Borrowed(file.path()),
            line: 1,
            column: 0,
        },
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
    let mut added_functions = unmatched(&current.functions, &snapshot.functions);
    let mut removed_functions = unmatched(&snapshot.functions, &current.functions);
    let mut added_types = unmatched(&current.types, &snapshot.types);
    let mut removed_types = unmatched(&snapshot.types, &current.types);
    let mut added_reexports = unmatched(&current.reexports, &snapshot.reexports);
    let mut removed_reexports = unmatched(&snapshot.reexports, &current.reexports);

    added_functions.sort_by(function_order);
    removed_functions.sort_by(function_order);
    added_types.sort_by(type_order);
    removed_types.sort_by(type_order);
    added_reexports.sort_unstable();
    removed_reexports.sort_unstable();

    InterfaceDelta {
        added_functions,
        removed_functions,
        added_types,
        removed_types,
        added_reexports,
        removed_reexports,
    }
}

fn unmatched<T: Clone + PartialEq>(left: &[T], right: &[T]) -> Vec<T> {
    let mut consumed = vec![false; right.len()];
    left.iter()
        .filter_map(|item| {
            if let Some(index) = right
                .iter()
                .enumerate()
                .position(|(index, candidate)| !consumed[index] && candidate == item)
            {
                consumed[index] = true;
                None
            } else {
                Some(item.clone())
            }
        })
        .collect()
}

fn function_order(
    left: &FunctionSignature<'_>,
    right: &FunctionSignature<'_>,
) -> std::cmp::Ordering {
    (&left.name, &left.params, &left.return_type).cmp(&(
        &right.name,
        &right.params,
        &right.return_type,
    ))
}

fn type_order(left: &TypeSignature<'_>, right: &TypeSignature<'_>) -> std::cmp::Ordering {
    (&left.name, type_kind_rank(&left.kind), &left.members).cmp(&(
        &right.name,
        type_kind_rank(&right.kind),
        &right.members,
    ))
}

fn type_kind_rank(kind: &crate::entities::parsed_file::TypeKind) -> u8 {
    use crate::entities::parsed_file::TypeKind;
    match kind {
        TypeKind::Struct => 0,
        TypeKind::Enum => 1,
        TypeKind::Trait => 2,
        TypeKind::Class => 3,
        TypeKind::Interface => 4,
        TypeKind::TypeAlias => 5,
    }
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
            prompt_refs: vec![],
            has_test_coverage: true,
            imports: vec![],
            tokens: vec![],
            public_interface: PublicInterface::empty(),
            prompt_snapshot: None,
            declared_traits: vec![],
            implemented_traits: vec![],
            blanket_impl_traits: vec![],
            declarations: vec![],
            static_declarations: vec![],
            module_decls: vec![],
            decision_exprs: vec![],
            constants: vec![],
            semantic_observations: vec![],
        }
    }

    fn fn_sig(name: &'static str) -> FunctionSignature<'static> {
        FunctionSignature {
            name,
            params: vec![],
            return_type: None,
        }
    }

    fn type_sig(name: &'static str) -> TypeSignature<'static> {
        TypeSignature {
            name,
            kind: TypeKind::Struct,
            members: vec![],
        }
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
        assert!(violations[0].message.contains("+struct Foo"));
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
        file.public_interface = PublicInterface {
            functions: vec![new_sig],
            types: vec![],
            reexports: vec![],
        };
        file.prompt_snapshot = Some(PublicInterface {
            functions: vec![old_sig],
            types: vec![],
            reexports: vec![],
        });
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V6");
        let msg = &violations[0].message;
        assert!(
            msg.contains("+fn foo"),
            "delta deve conter +fn foo, got: {msg}"
        );
        assert!(
            msg.contains("-fn foo"),
            "delta deve conter -fn foo, got: {msg}"
        );
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
        assert!(desc.contains("-struct OldType"));
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

    #[test]
    fn permutation_with_duplicate_multiplicity_has_empty_delta() {
        let current = PublicInterface {
            functions: vec![fn_sig("b"), fn_sig("a"), fn_sig("a")],
            types: vec![type_sig("B"), type_sig("A"), type_sig("A")],
            reexports: vec!["b", "a", "a"],
        };
        let snapshot = PublicInterface {
            functions: vec![fn_sig("a"), fn_sig("b"), fn_sig("a")],
            types: vec![type_sig("A"), type_sig("A"), type_sig("B")],
            reexports: vec!["a", "a", "b"],
        };
        assert!(compute_delta(&current, &snapshot).is_empty());
    }

    #[test]
    fn one_extra_duplicate_produces_one_entry_per_family() {
        let current = PublicInterface {
            functions: vec![fn_sig("a"), fn_sig("a")],
            types: vec![type_sig("A"), type_sig("A")],
            reexports: vec!["a", "a"],
        };
        let snapshot = PublicInterface {
            functions: vec![fn_sig("a")],
            types: vec![type_sig("A")],
            reexports: vec!["a"],
        };
        let delta = compute_delta(&current, &snapshot);
        assert_eq!(delta.added_functions, vec![fn_sig("a")]);
        assert_eq!(delta.added_types, vec![type_sig("A")]);
        assert_eq!(delta.added_reexports, vec!["a"]);
    }

    #[test]
    fn one_removed_duplicate_produces_one_entry_per_family() {
        let current = PublicInterface {
            functions: vec![fn_sig("a")],
            types: vec![type_sig("A")],
            reexports: vec!["a"],
        };
        let snapshot = PublicInterface {
            functions: vec![fn_sig("a"), fn_sig("a")],
            types: vec![type_sig("A"), type_sig("A")],
            reexports: vec!["a", "a"],
        };
        let delta = compute_delta(&current, &snapshot);
        assert_eq!(delta.removed_functions, vec![fn_sig("a")]);
        assert_eq!(delta.removed_types, vec![type_sig("A")]);
        assert_eq!(delta.removed_reexports, vec!["a"]);
    }

    #[test]
    fn delta_groups_are_sorted_by_all_signature_fields() {
        let function = |params, return_type| FunctionSignature {
            name: "same",
            params,
            return_type,
        };
        let typed = |kind, members| TypeSignature {
            name: "Same",
            kind,
            members,
        };
        let current = PublicInterface {
            functions: vec![
                function(vec!["z"], None),
                function(vec!["a"], Some("z")),
                function(vec!["a"], None),
            ],
            types: vec![
                typed(TypeKind::Trait, vec!["a"]),
                typed(TypeKind::Struct, vec!["z"]),
                typed(TypeKind::Struct, vec!["a"]),
            ],
            reexports: vec!["z", "a"],
        };
        let delta = compute_delta(&current, &PublicInterface::empty());
        assert_eq!(
            delta.added_functions,
            vec![
                function(vec!["a"], None),
                function(vec!["a"], Some("z")),
                function(vec!["z"], None),
            ]
        );
        assert_eq!(
            delta.added_types,
            vec![
                typed(TypeKind::Struct, vec!["a"]),
                typed(TypeKind::Struct, vec!["z"]),
                typed(TypeKind::Trait, vec!["a"]),
            ]
        );
        assert_eq!(delta.added_reexports, vec!["a", "z"]);
    }
}
