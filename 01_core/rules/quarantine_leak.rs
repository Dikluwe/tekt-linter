//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/quarantine-leak.md
//! @prompt-hash a2e5043c
//! @layer L1
//! @updated 2026-03-16

use std::borrow::Cow;

use crate::entities::rule_traits::HasImports;
use crate::entities::layer::Layer;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V10 — Quarantine Leak.
///
/// Código de produção (L1–L4) nunca pode importar de `lab/`.
/// O lab é quarentena intencional — sem garantias arquiteturais.
/// A assimetria é absoluta: lab pode importar produção, produção não
/// pode importar lab.
///
/// Fatal — não configurável. Aplica-se a L1, L2, L3 e L4.
/// Lab, L0 e Unknown são isentos.
pub fn check<'a, T: HasImports<'a>>(file: &T) -> Vec<Violation<'a>> {
    if matches!(file.layer(), Layer::Lab | Layer::L0 | Layer::Unknown) {
        return vec![];
    }

    file.imports()
        .iter()
        .filter(|import| import.target_layer == Layer::Lab)
        .map(|import| Violation {
            rule_id: "V10".to_string(),
            level: ViolationLevel::Fatal,
            message: format!(
                "Quarentena violada: '{}' é código de produção e não pode \
                 importar de lab/. Migrar o símbolo para a camada apropriada \
                 antes de usar em produção.",
                import.path
            ),
            location: Location {
                path: Cow::Borrowed(file.path()),
                line: import.line,
                column: 0,
            },
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

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
        fn layer(&self) -> &Layer { &self.layer }
        fn imports(&self) -> &[Import<'static>] { &self.imports }
        fn path(&self) -> &'static Path { self.path }
    }

    fn base_file(layer: Layer) -> MockFile {
        MockFile { layer, imports: vec![], path: Path::new("src/foo.rs") }
    }

    fn lab_import(line: usize) -> Import<'static> {
        Import { path: "crate::lab::algo", line, kind: ImportKind::Direct, target_layer: Layer::Lab, target_subdir: None }
    }

    fn non_lab_import() -> Import<'static> {
        Import { path: "crate::entities::Layer", line: 1, kind: ImportKind::Direct, target_layer: Layer::L1, target_subdir: None }
    }

    #[test]
    fn l1_importing_lab_is_fatal() {
        let mut file = base_file(Layer::L1);
        file.imports.push(lab_import(3));
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V10");
        assert_eq!(violations[0].level, ViolationLevel::Fatal);
        assert_eq!(violations[0].location.line, 3);
    }

    #[test]
    fn l2_importing_lab_is_fatal() {
        let mut file = base_file(Layer::L2);
        file.imports.push(lab_import(7));
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].level, ViolationLevel::Fatal);
    }

    #[test]
    fn l3_importing_lab_is_fatal() {
        let mut file = base_file(Layer::L3);
        file.imports.push(lab_import(5));
        assert_eq!(check(&file).len(), 1);
    }

    #[test]
    fn l4_importing_lab_is_fatal() {
        let mut file = base_file(Layer::L4);
        file.imports.push(lab_import(2));
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].level, ViolationLevel::Fatal);
    }

    #[test]
    fn lab_importing_production_is_exempt() {
        let mut file = base_file(Layer::Lab);
        file.imports.push(non_lab_import());
        assert!(check(&file).is_empty());
    }

    #[test]
    fn l0_is_exempt() {
        let mut file = base_file(Layer::L0);
        file.imports.push(lab_import(1));
        assert!(check(&file).is_empty());
    }

    #[test]
    fn unknown_layer_is_exempt() {
        let mut file = base_file(Layer::Unknown);
        file.imports.push(lab_import(1));
        assert!(check(&file).is_empty());
    }

    #[test]
    fn non_lab_import_in_l2_is_not_violation() {
        let mut file = base_file(Layer::L2);
        file.imports.push(non_lab_import());
        assert!(check(&file).is_empty());
    }

    #[test]
    fn unknown_target_layer_in_l1_is_not_violation() {
        let mut file = base_file(Layer::L1);
        file.imports.push(Import {
            path: "reqwest::Client",
            line: 4,
            kind: ImportKind::Direct,
            target_layer: Layer::Unknown,
            target_subdir: None,
        });
        assert!(check(&file).is_empty());
    }

    #[test]
    fn violation_message_contains_import_path() {
        let mut file = base_file(Layer::L1);
        file.imports.push(lab_import(1));
        let violations = check(&file);
        assert!(violations[0].message.contains("crate::lab::algo"));
    }

    #[test]
    fn multiple_lab_imports_produce_one_violation_each() {
        let mut file = base_file(Layer::L3);
        file.imports.push(lab_import(2));
        file.imports.push(lab_import(9));
        assert_eq!(check(&file).len(), 2);
    }
}
