//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/entities/analysis-policy.md
//! @prompt-hash 36578d9d
//! @layer L1
//! @updated 2026-09-26

use crate::entities::layer::{Language, Layer};
use crate::entities::parsed_file::ParsedFile;
use crate::entities::violation::{Violation, ViolationLevel};
use std::collections::BTreeMap;

pub fn rule_id(id: &str) -> bool {
    (0..=27).any(|n| id == format!("V{n}"))
}
pub fn language_name(language: &Language) -> &'static str {
    match language {
        Language::Rust => "rust",
        Language::TypeScript => "typescript",
        Language::Python => "python",
        Language::C => "c",
        Language::Cpp => "cpp",
        Language::Zig => "zig",
        Language::Go => "go",
        Language::Java => "java",
        Language::Elixir => "elixir",
        Language::Unknown => "unknown",
    }
}
pub fn level(value: &str) -> Option<ViolationLevel> {
    match value.to_ascii_lowercase().as_str() {
        "fatal" => Some(ViolationLevel::Fatal),
        "error" => Some(ViolationLevel::Error),
        "warning" => Some(ViolationLevel::Warning),
        "info" => Some(ViolationLevel::Info),
        _ => None,
    }
}
#[derive(Debug, Default)]
pub struct RulePolicy {
    pub level: Option<ViolationLevel>,
    pub languages: Option<Vec<String>>,
}
pub fn apply_policy(
    violations: &mut Vec<Violation<'_>>,
    files: &[ParsedFile<'_>],
    policy: &BTreeMap<String, RulePolicy>,
) {
    let languages_by_path: BTreeMap<_, _> = files
        .iter()
        .map(|file| (file.path, &file.language))
        .collect();
    violations.retain_mut(|finding| {
        if finding.level == ViolationLevel::Fatal
            || matches!(finding.rule_id.as_str(), "V0" | "V8" | "V10")
        {
            return true;
        }
        let Some(entry) = policy.get(&finding.rule_id) else {
            return true;
        };
        if let Some(languages) = &entry.languages {
            if finding.rule_id == "V22" && !languages.iter().any(|language| language == "rust") {
                return false;
            }
            if let Some(language) = languages_by_path.get(finding.location.path.as_ref()) {
                if !languages
                    .iter()
                    .any(|selected| selected == language_name(language))
                {
                    return false;
                }
            }
        }
        if let Some(level) = &entry.level {
            finding.level = level.clone();
        }
        true
    });
}
pub fn language_counts(files: &[ParsedFile<'_>]) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for file in files {
        *counts.entry(language_name(&file.language)).or_default() += 1;
    }
    counts
}
pub fn unknown_imports(files: &[ParsedFile<'_>]) -> usize {
    files
        .iter()
        .flat_map(|file| &file.imports)
        .filter(|import| import.target_layer == Layer::Unknown)
        .count()
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::violation::Location;
    use std::borrow::Cow;
    use std::path::Path;
    #[test]
    fn policy_changes_level_but_never_fatal() {
        let make = |id: &str, level| Violation {
            rule_id: id.into(),
            level,
            message: String::new(),
            location: Location {
                path: Cow::Borrowed(Path::new("a.rs")),
                line: 1,
                column: 0,
            },
        };
        let mut findings = vec![
            make("V1", ViolationLevel::Error),
            make("V0", ViolationLevel::Fatal),
        ];
        let policy = ["V1", "V0"]
            .into_iter()
            .map(|id| {
                (
                    id.into(),
                    RulePolicy {
                        level: Some(ViolationLevel::Info),
                        languages: None,
                    },
                )
            })
            .collect();
        apply_policy(&mut findings, &[], &policy);
        assert_eq!(findings[0].level, ViolationLevel::Info);
        assert_eq!(findings[1].level, ViolationLevel::Fatal);
    }
    #[test]
    fn identifiers_are_canonical() {
        assert!(rule_id("V27"));
        assert!(!rule_id("V01"));
        assert!(!rule_id("V28"));
    }
}
