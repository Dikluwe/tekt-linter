//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/violation-types.md
//! @prompt-hash 028f6e75
//! @layer L1
//! @updated 2026-03-13

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub rule_id: String,
    pub level: ViolationLevel,
    pub message: String,
    pub location: Location,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_violation(rule_id: &str, level: ViolationLevel) -> Violation {
        Violation {
            rule_id: rule_id.to_string(),
            level,
            message: "test message".to_string(),
            location: Location {
                path: PathBuf::from("01_core/foo.rs"),
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
    }

    #[test]
    fn location_eq() {
        let a = Location { path: PathBuf::from("foo.rs"), line: 1, column: 0 };
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
        for id in ["V1", "V2", "V3", "V4", "V5"] {
            let v = make_violation(id, ViolationLevel::Error);
            assert_eq!(v.rule_id, id);
        }
    }
}
