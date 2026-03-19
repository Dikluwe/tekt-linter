//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/wiring-logic-leak.md
//! @prompt-hash e7a9d264
//! @layer L1
//! @updated 2026-03-16

use std::borrow::Cow;

use crate::entities::rule_traits::HasWiringPurity;
use crate::entities::layer::Layer;
use crate::entities::parsed_file::{Declaration, DeclarationKind, WiringConfig};
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V12 — Wiring Logic Leak.
///
/// L4 tem um único propósito: instanciar e injetar. Não cria tipos,
/// não contém lógica de negócio. Structs de adapter são toleradas
/// quando `allow_adapter_structs = true` (padrão). Enums e impl sem
/// trait são sempre proibidos em L4 — indicam lógica embutida no fio.
///
/// Warning — não bloqueia CI por padrão (configurável via [rules]).
/// Aplica-se apenas a arquivos com `layer == L4`.
pub fn check<'a, T: HasWiringPurity<'a>>(file: &T, config: &WiringConfig) -> Vec<Violation<'a>> {
    if *file.layer() != Layer::L4 {
        return vec![];
    }

    file.declarations()
        .iter()
        .filter(|d| is_forbidden(d, config))
        .map(|d| Violation {
            rule_id: "V12".to_string(),
            level: ViolationLevel::Warning,
            message: format!(
                "Lógica no fio: {} '{}' declarado em L4. \
                 L4 não cria tipos — mover para L2 ou L3.",
                declaration_kind_str(&d.kind),
                d.name,
            ),
            location: Location {
                path: Cow::Borrowed(file.path()),
                line: d.line,
                column: 0,
            },
        })
        .collect()
}

fn is_forbidden(d: &Declaration, config: &WiringConfig) -> bool {
    match d.kind {
        DeclarationKind::Enum      => true,                           // enums nunca pertencem a L4
        DeclarationKind::Struct    => !config.allow_adapter_structs,  // configurável
        DeclarationKind::Impl      => true,                           // impl sem trait = lógica de negócio
        // ADR-0009: linguagens OO
        DeclarationKind::Class     => !config.allow_adapter_structs,  // Class ≡ Struct (ADR-0009)
        DeclarationKind::Interface => true,                           // interfaces pertencem a L1/L2
        DeclarationKind::TypeAlias => true,                           // type aliases pertencem a L1/L2
    }
}

fn declaration_kind_str(kind: &DeclarationKind) -> &'static str {
    match kind {
        DeclarationKind::Struct    => "struct",
        DeclarationKind::Enum      => "enum",
        DeclarationKind::Impl      => "impl",
        DeclarationKind::Class     => "class",
        DeclarationKind::Interface => "interface",
        DeclarationKind::TypeAlias => "type",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::Layer;
    use crate::entities::parsed_file::{Declaration, DeclarationKind, WiringConfig};
    use std::path::Path;

    struct MockFile {
        layer: Layer,
        declarations: Vec<Declaration<'static>>,
        path: &'static Path,
    }

    impl HasWiringPurity<'static> for MockFile {
        fn layer(&self) -> &Layer { &self.layer }
        fn declarations(&self) -> &[Declaration<'static>] { &self.declarations }
        fn path(&self) -> &'static Path { self.path }
    }

    fn l4_file(declarations: Vec<Declaration<'static>>) -> MockFile {
        MockFile { layer: Layer::L4, declarations, path: Path::new("04_wiring/main.rs") }
    }

    fn decl(kind: DeclarationKind, name: &'static str, line: usize) -> Declaration<'static> {
        Declaration { kind, name, line }
    }

    fn allow_structs() -> WiringConfig { WiringConfig { allow_adapter_structs: true } }
    fn deny_structs() -> WiringConfig { WiringConfig { allow_adapter_structs: false } }

    #[test]
    fn struct_forbidden_when_allow_adapter_structs_false() {
        let file = l4_file(vec![decl(DeclarationKind::Struct, "L3HashRewriter", 5)]);
        let violations = check(&file, &deny_structs());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V12");
        assert_eq!(violations[0].level, ViolationLevel::Warning);
        assert!(violations[0].message.contains("L3HashRewriter"));
    }

    #[test]
    fn struct_allowed_when_allow_adapter_structs_true() {
        let file = l4_file(vec![decl(DeclarationKind::Struct, "L3HashRewriter", 5)]);
        assert!(check(&file, &allow_structs()).is_empty());
    }

    #[test]
    fn enum_always_forbidden_in_l4() {
        let file = l4_file(vec![decl(DeclarationKind::Enum, "OutputMode", 10)]);
        let violations = check(&file, &allow_structs());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].level, ViolationLevel::Warning);
        assert!(violations[0].message.contains("OutputMode"));
    }

    #[test]
    fn impl_without_trait_always_forbidden() {
        let file = l4_file(vec![decl(DeclarationKind::Impl, "L3HashRewriter", 20)]);
        let violations = check(&file, &allow_structs());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].level, ViolationLevel::Warning);
        assert!(violations[0].message.contains("impl"));
    }

    #[test]
    fn non_l4_file_is_exempt() {
        let file = MockFile {
            layer: Layer::L3,
            declarations: vec![decl(DeclarationKind::Struct, "FileWalker", 1)],
            path: Path::new("03_infra/walker.rs"),
        };
        assert!(check(&file, &allow_structs()).is_empty());
    }

    #[test]
    fn l4_with_no_declarations_returns_empty() {
        assert!(check(&l4_file(vec![]), &allow_structs()).is_empty());
    }

    #[test]
    fn violation_line_matches_declaration_line() {
        let file = l4_file(vec![decl(DeclarationKind::Enum, "Fmt", 42)]);
        assert_eq!(check(&file, &allow_structs())[0].location.line, 42);
    }

    #[test]
    fn multiple_forbidden_declarations_each_produce_violation() {
        let file = l4_file(vec![
            decl(DeclarationKind::Enum, "Mode", 3),
            decl(DeclarationKind::Impl, "Config", 10),
        ]);
        assert_eq!(check(&file, &allow_structs()).len(), 2);
    }

    // ── ADR-0009: linguagens OO ────────────────────────────────────────────────

    #[test]
    fn class_without_implements_allowed_when_allow_adapter_structs_true() {
        let file = l4_file(vec![decl(DeclarationKind::Class, "Formatter", 5)]);
        assert!(check(&file, &allow_structs()).is_empty());
    }

    #[test]
    fn class_without_implements_forbidden_when_allow_adapter_structs_false() {
        let file = l4_file(vec![decl(DeclarationKind::Class, "Formatter", 5)]);
        let violations = check(&file, &deny_structs());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Formatter"));
        assert!(violations[0].message.contains("class"));
    }

    #[test]
    fn interface_always_forbidden_in_l4() {
        let file = l4_file(vec![decl(DeclarationKind::Interface, "InternalConfig", 7)]);
        let violations = check(&file, &allow_structs());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("InternalConfig"));
        assert!(violations[0].message.contains("interface"));
    }

    #[test]
    fn type_alias_always_forbidden_in_l4() {
        let file = l4_file(vec![decl(DeclarationKind::TypeAlias, "Mode", 9)]);
        let violations = check(&file, &allow_structs());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Mode"));
        assert!(violations[0].message.contains("type"));
    }

    #[test]
    fn class_with_allow_structs_and_interface_both_in_l4() {
        // Class allowed, Interface forbidden
        let file = l4_file(vec![
            decl(DeclarationKind::Class, "Adapter", 2),
            decl(DeclarationKind::Interface, "Config", 10),
        ]);
        let violations = check(&file, &allow_structs());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Config"));
    }
}
