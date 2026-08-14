//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/wildcard-saturation.md
//! @prompt-hash f9af51c3
//! @layer L1
//! @updated 2026-08-14

use std::borrow::Cow;
use std::collections::HashMap;

use crate::entities::layer::Language;
use crate::entities::rule_traits::{
    decision_arm_term_for, BodyForm, HasDecisionArms, ScrutineeForm,
};
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V16 — WildcardSaturation (ADR-0016).
///
/// Detecta braços catch-all que descartam informação de enums fechados de domínio,
/// permitindo que variantes futuras sejam adoptadas silenciosamente sem erro de compilação.
pub fn check<'a, T: HasDecisionArms<'a>>(
    file: &T,
    exceptions: &HashMap<String, String>,
) -> Vec<Violation<'a>> {
    // Escopo inicial: Rust
    if *file.language() != Language::Rust {
        return vec![];
    }

    let mut violations = Vec::new();
    let term = decision_arm_term_for(file.language());
    let path_str = file.path().to_string_lossy();

    // Rastrear spans de catch-all existentes no arquivo para validação de exceções obsoletas
    let mut catchall_lines = Vec::new();

    for expr in file.decision_exprs() {
        // Filtro: scrutinee aberto (chamadas de método, indexação ou literal) -> ISENTO
        if matches!(
            expr.scrutinee_form,
            ScrutineeForm::MethodCall | ScrutineeForm::Index | ScrutineeForm::Literal
        ) {
            continue;
        }

        // Enum candidato: >= 2 braços qualificados com prefixo no mesmo match
        let mut all_prefixes = Vec::new();
        for arm in &expr.arms {
            for p in &arm.qualified_prefixes {
                all_prefixes.push(*p);
            }
        }
        let is_candidate_enum = all_prefixes.len() >= 2;

        for arm in &expr.arms {
            if !arm.is_catchall {
                continue;
            }

            catchall_lines.push(arm.line);

            // Filtro de reincorporação: identifica se o identificador é usado no corpo
            if arm.bound_ident_used_in_body {
                continue;
            }

            // Filtro de barreira de erro: panic!, unreachable!, bail!, Err(...), etc.
            if arm.body_form == BodyForm::ErrorBarrier {
                continue;
            }

            if !is_candidate_enum {
                continue;
            }

            // Classificação do corpo
            let (level, msg) = match arm.body_form {
                BodyForm::EnumPath | BodyForm::LiteralOther => (
                    ViolationLevel::Warning,
                    format!(
                        "{} satura variantes futuras para `{}` — exige exaustividade nominal",
                        term, arm.body_snippet
                    ),
                ),
                BodyForm::LiteralNeutral => (
                    ViolationLevel::Warning,
                    format!(
                        "{} descarta informação com default neutro `{}` em enum de domínio",
                        term, arm.body_snippet
                    ),
                ),
                BodyForm::Call => (
                    ViolationLevel::Info,
                    format!("{} delega decisão para `{}`", term, arm.body_snippet),
                ),
                BodyForm::EmptyBlock | BodyForm::Continue => (
                    ViolationLevel::Warning,
                    format!(
                        "{} ignora variantes silenciosamente com `{}` em enum de domínio",
                        term, arm.body_snippet
                    ),
                ),
                BodyForm::Other => (
                    ViolationLevel::Warning,
                    format!(
                        "{} descarta informação em enum de domínio com `{}`",
                        term, arm.body_snippet
                    ),
                ),
                BodyForm::ErrorBarrier => continue,
            };

            // Verificar exceções declaradas em [wildcard_exceptions]
            let loc_key = format!("{}:{}", path_str, arm.line);
            if let Some(justification) = exceptions.get(&loc_key) {
                let trimmed = justification.trim();
                if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("ok") {
                    violations.push(Violation {
                        rule_id: "V16".to_string(),
                        level: ViolationLevel::Warning,
                        message: format!(
                            "Excepção de wildcard em '{}' sem justificativa válida: forneça a razão técnica.",
                            loc_key
                        ),
                        location: Location {
                            path: Cow::Borrowed(file.path()),
                            line: arm.line,
                            column: arm.column,
                        },
                    });
                }
                // Exceção declarada suprime a violação de saturação
                continue;
            }

            violations.push(Violation {
                rule_id: "V16".to_string(),
                level,
                message: msg,
                location: Location {
                    path: Cow::Borrowed(file.path()),
                    line: arm.line,
                    column: arm.column,
                },
            });
        }
    }

    // Detecção de spans obsoletos: chaves da tabela de exceções para este arquivo que não têm catchall activo
    for (key, _) in exceptions {
        if let Some((f_path, line_str)) = key.split_once(':') {
            let matches_path = path_str == f_path || path_str.ends_with(f_path) || f_path.ends_with(&*path_str);
            if matches_path {
                if let Ok(line_num) = line_str.parse::<usize>() {
                    if !catchall_lines.contains(&line_num) {
                        violations.push(Violation {
                            rule_id: "V16".to_string(),
                            level: ViolationLevel::Warning,
                            message: format!(
                                "Excepção de wildcard obsoleta: '{}' não contém braço catch-all activo.",
                                key
                            ),
                            location: Location {
                                path: Cow::Borrowed(file.path()),
                                line: line_num,
                                column: 0,
                            },
                        });
                    }
                }
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use crate::entities::layer::Layer;
    use crate::entities::rule_traits::{DecisionArm, DecisionExpr};
    
    struct MockFile {
        path: &'static Path,
        language: Language,
        exprs: Vec<DecisionExpr<'static>>,
    }

    impl HasDecisionArms<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn decision_exprs(&self) -> &[DecisionExpr<'static>] {
            &self.exprs
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &self.language
        }
    }

    #[test]
    fn v16_detects_deny_class_wildcard_saturation() {
        let arm1 = DecisionArm {
            pattern_snippet: "Unit::Pt",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec!["Unit"],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 10,
            column: 12,
        };
        let arm2 = DecisionArm {
            pattern_snippet: "Unit::Mm",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec!["Unit"],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 11,
            column: 12,
        };
        let arm3 = DecisionArm {
            pattern_snippet: "_",
            is_catchall: true,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec![],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::EnumPath,
            body_snippet: "Unit::Percent",
            line: 12,
            column: 12,
        };
        let expr = DecisionExpr {
            snippet_scrutinee: "unit",
            scrutinee_form: ScrutineeForm::Path,
            arms: vec![arm1, arm2, arm3],
            line: 9,
            column: 8,
        };
        let file = MockFile {
            path: Path::new("01_core/unit.rs"),
            language: Language::Rust,
            exprs: vec![expr],
        };
        let exceptions = HashMap::new();
        let viols = check(&file, &exceptions);
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].rule_id, "V16");
        assert_eq!(viols[0].level, ViolationLevel::Warning);
        assert!(viols[0].message.contains("wildcard `_ =>` satura"));
    }

    #[test]
    fn v16_reincorporation_is_exempt() {
        let arm1 = DecisionArm {
            pattern_snippet: "Unit::Pt",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec!["Unit"],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 10,
            column: 12,
        };
        let arm2 = DecisionArm {
            pattern_snippet: "Unit::Mm",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec!["Unit"],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 11,
            column: 12,
        };
        let arm3 = DecisionArm {
            pattern_snippet: "other",
            is_catchall: true,
            bound_ident_used_in_body: true,
            qualified_prefixes: vec![],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::Call,
            body_snippet: "f(other)",
            line: 12,
            column: 12,
        };
        let expr = DecisionExpr {
            snippet_scrutinee: "unit",
            scrutinee_form: ScrutineeForm::Path,
            arms: vec![arm1, arm2, arm3],
            line: 9,
            column: 8,
        };
        let file = MockFile {
            path: Path::new("01_core/unit.rs"),
            language: Language::Rust,
            exprs: vec![expr],
        };
        let exceptions = HashMap::new();
        let viols = check(&file, &exceptions);
        assert!(viols.is_empty());
    }

    #[test]
    fn v16_exception_suppresses_violation() {
        let arm1 = DecisionArm {
            pattern_snippet: "Unit::Pt",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec!["Unit"],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 10,
            column: 12,
        };
        let arm2 = DecisionArm {
            pattern_snippet: "Unit::Mm",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec!["Unit"],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 11,
            column: 12,
        };
        let arm3 = DecisionArm {
            pattern_snippet: "_",
            is_catchall: true,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec![],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::EnumPath,
            body_snippet: "Unit::Percent",
            line: 12,
            column: 12,
        };
        let expr = DecisionExpr {
            snippet_scrutinee: "unit",
            scrutinee_form: ScrutineeForm::Path,
            arms: vec![arm1, arm2, arm3],
            line: 9,
            column: 8,
        };
        let file = MockFile {
            path: Path::new("01_core/unit.rs"),
            language: Language::Rust,
            exprs: vec![expr],
        };
        let mut exceptions = HashMap::new();
        exceptions.insert("01_core/unit.rs:12".to_string(), "hub intencional: fallback documentado".to_string());
        let viols = check(&file, &exceptions);
        assert!(viols.is_empty());
    }

    #[test]
    fn v16_obsolete_exception_emits_warning() {
        let file = MockFile {
            path: Path::new("01_core/unit.rs"),
            language: Language::Rust,
            exprs: vec![],
        };
        let mut exceptions = HashMap::new();
        exceptions.insert("01_core/unit.rs:99".to_string(), "hub antigo".to_string());
        let viols = check(&file, &exceptions);
        assert_eq!(viols.len(), 1);
        assert!(viols[0].message.contains("obsoleta"));
    }
}
