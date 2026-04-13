//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/forbidden-import.md
//! @prompt-hash d6bde6a2
//! @layer L1
//! @updated 2026-03-14

use std::borrow::Cow;

use crate::entities::rule_traits::HasImports;
use crate::entities::layer::Layer;
use crate::entities::parsed_file::Import;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V3 — Forbidden import (gravity inversion).
/// Compares file.layer with import.target_layer using the permission matrix.
/// Layer::Unknown is never a violation (external crates).
/// One violation per forbidden import.
///
/// Permission matrix (forbidden target_layers per source layer):
/// L1 → L2, L3, L4, Lab
/// L2 → L3, L4, Lab
/// L3 → L2, L4, Lab
/// L4 → Lab
/// L0, Lab → no restrictions
pub fn check<'a, T: HasImports<'a>>(file: &T) -> Vec<Violation<'a>> {
    file.imports()
        .iter()
        .filter(|import| is_forbidden(file.layer(), &import.target_layer))
        .map(|import| make_violation(file, import))
        .collect()
}

fn is_forbidden(source: &Layer, target: &Layer) -> bool {
    if *target == Layer::Unknown {
        return false;
    }
    match source {
        Layer::L1 => matches!(target, Layer::L2 | Layer::L3 | Layer::L4 | Layer::Lab),
        Layer::L2 => matches!(target, Layer::L3 | Layer::L4 | Layer::Lab),
        Layer::L3 => matches!(target, Layer::L2 | Layer::L4 | Layer::Lab),
        Layer::L4 => matches!(target, Layer::Lab),
        Layer::L0 | Layer::Lab | Layer::Unknown => false,
    }
}

fn make_violation<'a, T: HasImports<'a>>(file: &T, import: &Import<'a>) -> Violation<'a> {
    Violation {
        rule_id: "V3".to_string(),
        level: ViolationLevel::Error,
        message: format!(
            "Inversão de gravidade: {:?} não pode importar de {:?} ('{}')",
            file.layer(), import.target_layer, import.path
        ),
        location: Location { path: Cow::Borrowed(file.path()), line: import.line, column: 0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::Layer;
    use crate::entities::parsed_file::{Import, ImportKind};
    use std::path::Path;

    struct MockFile {
        layer: Layer,
        imports: Vec<Import<'static>>,
        path: &'static Path,
    }

    impl HasImports<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &self.layer
        }
        fn imports(&self) -> &[Import<'static>] {
            &self.imports
        }
        fn path(&self) -> &'static Path {
            self.path
        }
    }

    fn base_file(layer: Layer) -> MockFile {
        MockFile {
            layer,
            imports: vec![],
            path: Path::new("src/foo.rs"),
        }
    }

    fn import(path: &'static str, line: usize, target_layer: Layer) -> Import<'static> {
        Import { path, line, kind: ImportKind::Direct, target_layer, target_subdir: None }
    }

    #[test]
    fn l2_importing_l3_is_violation() {
        let mut file = base_file(Layer::L2);
        file.imports.push(import("crate::infra::db", 4, Layer::L3));
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V3");
        assert_eq!(violations[0].location.line, 4);
    }

    #[test]
    fn l1_importing_unknown_is_not_violation() {
        let mut file = base_file(Layer::L1);
        file.imports.push(import("reqwest::Client", 2, Layer::Unknown));
        assert!(check(&file).is_empty());
    }

    #[test]
    fn l4_importing_l1_is_allowed() {
        let mut file = base_file(Layer::L4);
        file.imports.push(import("crate::core::rules", 7, Layer::L1));
        assert!(check(&file).is_empty());
    }

    #[test]
    fn l3_two_imports_only_one_forbidden() {
        let mut file = base_file(Layer::L3);
        file.imports.push(import("crate::shell::api", 3, Layer::L2)); // forbidden
        file.imports.push(import("crate::core::entities", 7, Layer::L1)); // allowed
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].location.line, 3);
    }

    #[test]
    fn lab_has_no_import_restrictions() {
        let mut file = base_file(Layer::Lab);
        file.imports.push(import("crate::core::foo", 1, Layer::L1));
        file.imports.push(import("crate::shell::bar", 2, Layer::L2));
        file.imports.push(import("crate::infra::baz", 3, Layer::L3));
        assert!(check(&file).is_empty());
    }

    #[test]
    fn l4_importing_lab_is_forbidden() {
        let mut file = base_file(Layer::L4);
        file.imports.push(import("lab::experiment", 5, Layer::Lab));
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V3");
    }

    #[test]
    fn violation_message_contains_layers_and_path() {
        let mut file = base_file(Layer::L1);
        file.imports.push(import("crate::infra::db", 9, Layer::L3));
        let violations = check(&file);
        assert!(violations[0].message.contains("L1"));
        assert!(violations[0].message.contains("L3"));
        assert!(violations[0].message.contains("crate::infra::db"));
    }

    // ── Critérios adicionais: V3 deve funcionar identicamente para qualquer ImportKind

    #[test]
    fn l2_importing_l3_with_named_kind_is_violation() {
        // Import { kind: ImportKind::Named, target_layer: L3 } em L2 → Violation V3
        // V3 não usa ImportKind na sua lógica — proíbe por layer, não por kind
        let mut file = base_file(Layer::L2);
        file.imports.push(Import {
            path: "crate::infra::db",
            line: 4,
            kind: ImportKind::Named,
            target_layer: Layer::L3,
            target_subdir: None,
        });
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V3");
    }

    #[test]
    fn l2_importing_l3_with_glob_kind_is_violation() {
        // Import { kind: ImportKind::Glob, target_layer: L3 } em L2 → Violation V3
        let mut file = base_file(Layer::L2);
        file.imports.push(Import {
            path: "crate::infra::*",
            line: 5,
            kind: ImportKind::Glob,
            target_layer: Layer::L3,
            target_subdir: None,
        });
        assert_eq!(check(&file).len(), 1);
    }

    #[test]
    fn l2_importing_l3_with_alias_kind_is_violation() {
        // Import { kind: ImportKind::Alias, target_layer: L3 } em L2 → Violation V3
        let mut file = base_file(Layer::L2);
        file.imports.push(Import {
            path: "crate::infra::db as db_infra",
            line: 6,
            kind: ImportKind::Alias,
            target_layer: Layer::L3,
            target_subdir: None,
        });
        assert_eq!(check(&file).len(), 1);
    }
}
