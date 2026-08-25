//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/wildcard-saturation.md
//! @prompt-hash 3f6a3911
//! @layer L1
//! @updated 2026-08-14

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use crate::entities::layer::Language;
use crate::entities::rule_traits::{
    decision_arm_term_for, BodyForm, HasDecisionArms, ScrutineeForm,
};
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V16 — WildcardSaturation (ADR-0016 / ADR-0017 em `00_nucleo/adr/0017-v16-v21-diferenca-categorica.md`).
///
/// V16 nunca silencia por citação — um wildcard vigia todas as variantes futuras de um enum, não um valor fixo. Ver ADR para a distinção com V21.
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
        catchall_lines.extend(
            expr.arms
                .iter()
                .filter(|arm| arm.is_catchall)
                .map(|arm| arm.line),
        );

        // Filtro: scrutinee aberto (chamadas de método, indexação ou literal) -> ISENTO
        if matches!(
            expr.scrutinee_form,
            ScrutineeForm::MethodCall | ScrutineeForm::Index | ScrutineeForm::Literal
        ) {
            continue;
        }

        // Enum candidato: >= 2 braços qualificados com prefixo no mesmo match
        let mut prefix_arm_counts: HashMap<&str, usize> = HashMap::new();
        for arm in &expr.arms {
            let distinct_in_arm: HashSet<_> = arm
                .qualified_prefixes
                .iter()
                .copied()
                .filter(|prefix| !prefix.is_empty())
                .collect();
            for prefix in distinct_in_arm {
                *prefix_arm_counts.entry(prefix).or_default() += 1;
            }
        }
        let is_candidate_enum = prefix_arm_counts.values().any(|count| *count >= 2);

        for arm in &expr.arms {
            if !arm.is_catchall {
                continue;
            }

            // Filtro de reincorporação: identifica se o identificador é usado no corpo
            if arm.bound_ident_used_in_body {
                continue;
            }

            // Filtros de barreira ruidosa (ErrorBarrier / MessageProducer):
            // panic!, unreachable!, bail!, Err(...), format!("cannot..."), etc.
            if matches!(
                arm.body_form,
                BodyForm::ErrorBarrier | BodyForm::MessageProducer
            ) {
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
                BodyForm::ErrorBarrier | BodyForm::MessageProducer => continue,
            };

            // Verificar exceções / anotações de taxonomia declaradas em [wildcard_exceptions]
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

                // Validação de formato da taxonomia N16[α/β/γ] quando a tag estiver presente
                if trimmed.contains("N16[") {
                    let has_valid_tag = trimmed.contains("N16[α]")
                        || trimmed.contains("N16[β]")
                        || trimmed.contains("N16[γ]")
                        || trimmed.contains("N16[A]")
                        || trimmed.contains("N16[B]")
                        || trimmed.contains("N16[C]")
                        || trimmed.contains("N16[a]")
                        || trimmed.contains("N16[b]")
                        || trimmed.contains("N16[c]");
                    if !has_valid_tag {
                        violations.push(Violation {
                            rule_id: "V16".to_string(),
                            level: ViolationLevel::Warning,
                            message: format!(
                                "Tag N16 malformada em '{}': esperado N16[α], N16[β] ou N16[γ] (ou A/B/C), recebido '{}'",
                                loc_key, trimmed
                            ),
                            location: Location {
                                path: Cow::Borrowed(file.path()),
                                line: arm.line,
                                column: arm.column,
                            },
                        });
                    }
                }
            }

            // V16 nunca silencia por citação/anotação (ADR-0017): o sinal mantém-se sempre visível.

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
    let mut stale_exceptions: Vec<_> = exceptions
        .keys()
        .filter_map(|key| {
            let (exception_path, line) = key.rsplit_once(':')?;
            let line = line.parse::<usize>().ok()?;
            (exception_path == path_str && !catchall_lines.contains(&line)).then_some((key, line))
        })
        .collect();
    stale_exceptions.sort_by(|(left, _), (right, _)| left.cmp(right));

    for (key, line) in stale_exceptions {
        violations.push(Violation {
            rule_id: "V16".to_string(),
            level: ViolationLevel::Warning,
            message: format!(
                "Excepção de wildcard obsoleta: '{}' não contém braço catch-all activo.",
                key
            ),
            location: Location {
                path: Cow::Borrowed(file.path()),
                line,
                column: 0,
            },
        });
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::Layer;
    use crate::entities::rule_traits::{DecisionArm, DecisionExpr};
    use std::path::Path;

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
    fn v16_valid_n16_tag_does_not_silence_violation() {
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
        exceptions.insert(
            "01_core/unit.rs:12".to_string(),
            "N16[α]: impossibilidade estrutural".to_string(),
        );
        let viols = check(&file, &exceptions);
        // ADR-0017: V16 nunca silencia por citação/anotação — o aviso mantém-se visível, sem erro extra de formato
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].rule_id, "V16");
        assert!(viols[0].message.contains("wildcard `_ =>` satura"));
    }

    #[test]
    fn v16_malformed_n16_tag_emits_warning_and_does_not_silence() {
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
        exceptions.insert(
            "01_core/unit.rs:12".to_string(),
            "N16[INVALID]: tag incorreta".to_string(),
        );
        let viols = check(&file, &exceptions);
        assert_eq!(viols.len(), 2);
        assert!(viols[0].message.contains("Tag N16 malformada"));
        assert!(viols[1].message.contains("wildcard `_ =>` satura"));
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
