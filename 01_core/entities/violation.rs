//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/violation-types.md
//! @prompt-hash 28b2c451
//! @layer L1
//! @updated 2026-03-14

use std::borrow::Cow;
use std::path::Path;

/// Fatal: erros de infraestrutura que impedem análise completa (V0).
/// Fatal não pode ser suprimido por --fail-on — bloqueia CI
/// independentemente de configuração.
/// Error: violações arquiteturais bloqueantes (V1–V4).
/// Warning: divergências não bloqueantes por padrão (V5–V6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationLevel {
    Fatal,
    Error,
    Warning,
}

/// ADR-0005: path usa Cow<'a, Path>.
/// Borrowed(&'a Path) — violações normais (V1–V6), path referencia o SourceFile.
/// Owned(PathBuf)     — erros de infraestrutura (V0, PARSE), path é owned.
/// Elimina Box::leak() nos conversores em L4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location<'a> {
    pub path: Cow<'a, Path>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation<'a> {
    pub rule_id: String,  // "V0"–"V6", "PARSE" — gerado pela regra
    pub level: ViolationLevel,
    pub message: String,  // formatado pela regra
    pub location: Location<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::path::Path;

    fn make_violation(rule_id: &str, level: ViolationLevel) -> Violation<'static> {
        Violation {
            rule_id: rule_id.to_string(),
            level,
            message: "test message".to_string(),
            location: Location {
                path: Cow::Borrowed(Path::new("01_core/foo.rs")),
                line: 5,
                column: 3,
            },
        }
    }

    #[test]
    fn violation_clone_and_eq() {
        let v = make_violation("V1", ViolationLevel::Error);
        assert_eq!(v.clone(), v);
    }

    #[test]
    fn violation_levels_are_distinct() {
        assert_ne!(ViolationLevel::Error, ViolationLevel::Warning);
        assert_ne!(ViolationLevel::Fatal, ViolationLevel::Error);
        assert_ne!(ViolationLevel::Fatal, ViolationLevel::Warning);
    }

    #[test]
    fn location_eq() {
        let a = Location { path: Cow::Borrowed(Path::new("foo.rs")), line: 1, column: 0 };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn violation_debug_is_readable() {
        let v = make_violation("V3", ViolationLevel::Warning);
        let s = format!("{:?}", v);
        assert!(s.contains("V3"));
        assert!(s.contains("Warning"));
    }

    #[test]
    fn all_rule_ids_representable() {
        for id in ["V0", "V1", "V2", "V3", "V4", "V5", "V6"] {
            let v = make_violation(id, ViolationLevel::Error);
            assert_eq!(v.rule_id, id);
        }
    }

    #[test]
    fn fatal_level_is_distinct() {
        let v = make_violation("V0", ViolationLevel::Fatal);
        assert_eq!(v.level, ViolationLevel::Fatal);
        assert_ne!(v.level, ViolationLevel::Error);
        assert_ne!(v.level, ViolationLevel::Warning);
    }
}
