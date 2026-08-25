//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/shell/n16-summary.md
//! @prompt-hash 1a0fa53f
//! @layer L2
//! @updated 2026-08-18

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use crate::contracts::file_provider::SourceFile;

/// Tag de classificação da taxonomia N16 (ADR-0017 / Passo 0068 / Passo 0069).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum N16Tag {
    /// N16[α] — Impossibilidade estrutural / fechamento
    Alpha,
    /// N16[β] — Comportamento uniforme genuíno por contrato
    Beta,
    /// N16[γ] — Fallback deliberado e aberto / vigilância ativa
    Gamma,
}

impl N16Tag {
    pub fn as_str(&self) -> &'static str {
        match self {
            N16Tag::Alpha => "α",
            N16Tag::Beta => "β",
            N16Tag::Gamma => "γ",
        }
    }
}

/// Estatísticas acumuladas de tags N16 para um módulo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct N16ModuleStats {
    pub alpha: usize,
    pub beta: usize,
    pub gamma: usize,
}

impl N16ModuleStats {
    pub fn total(&self) -> usize {
        self.alpha + self.beta + self.gamma
    }

    pub fn gamma_pct(&self) -> f64 {
        let tot = self.total();
        if tot == 0 {
            0.0
        } else {
            (self.gamma as f64 / tot as f64) * 100.0
        }
    }

    pub fn gamma_pct_str(&self) -> String {
        let tot = self.total();
        if tot == 0 {
            "—".to_string()
        } else {
            percentage_tenths(self.gamma, tot)
        }
    }
}

pub type N16Stats = BTreeMap<String, N16ModuleStats>;

/// Extrai tag N16 a partir de uma linha de texto ou justificativa.
pub fn extract_n16_tag(text: &str) -> Option<N16Tag> {
    let mut remaining = text;
    let mut found = None;
    while let Some(start) = remaining.find("N16[") {
        let candidate = &remaining[start..];
        let matched = [
            ("N16[α]", N16Tag::Alpha),
            ("N16[β]", N16Tag::Beta),
            ("N16[γ]", N16Tag::Gamma),
            ("N16[A]", N16Tag::Alpha),
            ("N16[a]", N16Tag::Alpha),
            ("N16[B]", N16Tag::Beta),
            ("N16[b]", N16Tag::Beta),
            ("N16[C]", N16Tag::Gamma),
            ("N16[c]", N16Tag::Gamma),
        ]
        .into_iter()
        .find(|(token, _)| candidate.starts_with(token));
        if let Some((token, tag)) = matched {
            if found.is_some() {
                return None;
            }
            found = Some(tag);
            remaining = &candidate[token.len()..];
        } else {
            remaining = &candidate[4..];
        }
    }
    found
}

fn nominal_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Extrai o nome canônico do módulo para fins de agrupamento N16.
pub fn extract_n16_module_name(path: &Path) -> String {
    let path = path.to_string_lossy();
    let components: Vec<&str> = path
        .split(['/', '\\'])
        .filter(|component| !component.is_empty() && *component != ".")
        .collect();
    let Some(layer_index) = components.iter().position(|component| {
        matches!(
            *component,
            "00_nucleo" | "01_core" | "02_shell" | "03_infra" | "04_wiring"
        )
    }) else {
        return "other/".to_string();
    };
    let layer = components[layer_index];
    if layer != "01_core" {
        return format!("{layer}/");
    }
    let Some(src_offset) = components[layer_index + 1..]
        .iter()
        .position(|c| *c == "src")
    else {
        return "01_core/".to_string();
    };
    let module_index = layer_index + src_offset + 2;
    let Some(module) = components.get(module_index) else {
        return "01_core/".to_string();
    };
    if *module != ".." && module.contains('.') {
        return "01_core/".to_string();
    }
    if *module == "math" && components.get(module_index + 1) == Some(&"layout") {
        "math/layout/".to_string()
    } else {
        format!("{module}/")
    }
}

/// Coleta e agrega todas as anotações N16 de fontes e de exceções declaradas,
/// garantindo deduplicação estrita quando a mesma localização (path:linha) possui
/// anotação tanto no código-fonte quanto no `crystalline.toml`.
pub fn collect_n16_stats(sources: &[SourceFile], exceptions: &HashMap<String, String>) -> N16Stats {
    let mut source_locs: HashSet<(String, usize)> = HashSet::new();
    let mut stats: N16Stats = BTreeMap::new();

    // 1. Varrer linhas de código-fonte
    for sf in sources {
        let source_path = nominal_path(&sf.path);
        for (idx, line) in sf.content.lines().enumerate() {
            let line_num = idx + 1;
            if let Some(tag) = extract_n16_tag(line) {
                if source_locs.insert((source_path.clone(), line_num)) {
                    increment(&mut stats, extract_n16_module_name(&sf.path), tag);
                }
            }
        }
    }

    // 2. Varrer exceções de crystalline.toml (wildcard_exceptions)
    for (loc_key, justification) in exceptions {
        if let Some(tag) = extract_n16_tag(justification) {
            let Some((raw_path, raw_line)) = loc_key.rsplit_once(':') else {
                continue;
            };
            let Ok(line_num) = raw_line.parse::<usize>() else {
                continue;
            };
            if !source_locs.contains(&(raw_path.to_string(), line_num)) {
                increment(
                    &mut stats,
                    extract_n16_module_name(Path::new(raw_path)),
                    tag,
                );
            }
        }
    }

    stats
}

fn increment(stats: &mut N16Stats, module: String, tag: N16Tag) {
    let entry = stats.entry(module).or_default();
    match tag {
        N16Tag::Alpha => entry.alpha += 1,
        N16Tag::Beta => entry.beta += 1,
        N16Tag::Gamma => entry.gamma += 1,
    }
}

fn percentage_tenths(numerator: usize, denominator: usize) -> String {
    let tenths = ((numerator as u128) * 1000 + denominator as u128 / 2) / denominator as u128;
    format!("{}.{:01}%", tenths / 10, tenths % 10)
}

/// Formata o relatório consolidado N16 por módulo (Passo 0069).
pub fn format_n16_summary(stats: &N16Stats, min_sample_size: usize) -> String {
    let mut rows: Vec<(&String, &N16ModuleStats)> = stats.iter().collect();

    // Ordenação normativa: γ absoluto decrescente, depois nome por bytes UTF-8.
    rows.sort_by(|(name_a, stats_a), (name_b, stats_b)| {
        stats_b
            .gamma
            .cmp(&stats_a.gamma)
            .then_with(|| name_a.cmp(name_b))
    });

    let mut out = String::new();
    out.push_str("| Módulo | Total | α | β | γ | % γ |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");

    let mut total_alpha = 0;
    let mut total_beta = 0;
    let mut total_gamma = 0;
    let mut small_sample_warnings = Vec::new();

    for (module_name, s) in &rows {
        let tot = s.total();
        total_alpha += s.alpha;
        total_beta += s.beta;
        total_gamma += s.gamma;

        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} |\n",
            module_name,
            tot,
            s.alpha,
            s.beta,
            s.gamma,
            s.gamma_pct_str()
        ));

        // Linha de aviso obrigatória para qualquer módulo com total < min_sample_size
        if tot < min_sample_size && tot > 0 {
            let pp = (100usize + tot / 2) / tot;
            small_sample_warnings.push(format!(
                "⚠ amostra pequena em `{}` (n={}) — percentual pouco confiável, 1 caso muda o resultado em ~{}pp",
                module_name, tot, pp
            ));
        }
    }

    let total_all = total_alpha + total_beta + total_gamma;
    let total_pct_str = if total_all == 0 {
        "—".to_string()
    } else {
        percentage_tenths(total_gamma, total_all)
    };

    out.push_str(&format!(
        "| **Total** | **{}** | **{}** | **{}** | **{}** | **{}** |\n",
        total_all, total_alpha, total_beta, total_gamma, total_pct_str
    ));

    if !small_sample_warnings.is_empty() {
        out.push('\n');
        for w in small_sample_warnings {
            out.push_str(&w);
            out.push('\n');
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::{Language, Layer};
    use std::path::PathBuf;

    #[test]
    fn extract_n16_tag_recognizes_greek_and_ascii_variants() {
        assert_eq!(
            extract_n16_tag("// neutro: N16[α] — fechamento"),
            Some(N16Tag::Alpha)
        );
        assert_eq!(
            extract_n16_tag("// neutro: N16[β] — uniforme"),
            Some(N16Tag::Beta)
        );
        assert_eq!(
            extract_n16_tag("// neutro: N16[γ] — fallback"),
            Some(N16Tag::Gamma)
        );
        assert_eq!(
            extract_n16_tag("// neutro: N16[A] — fechamento"),
            Some(N16Tag::Alpha)
        );
        assert_eq!(
            extract_n16_tag("// neutro: N16[B] — uniforme"),
            Some(N16Tag::Beta)
        );
        assert_eq!(
            extract_n16_tag("// neutro: N16[C] — fallback"),
            Some(N16Tag::Gamma)
        );
        assert_eq!(
            extract_n16_tag("// neutro: N16[a] — fechamento"),
            Some(N16Tag::Alpha)
        );
        assert_eq!(
            extract_n16_tag("// neutro: N16[b] — uniforme"),
            Some(N16Tag::Beta)
        );
        assert_eq!(
            extract_n16_tag("// neutro: N16[c] — fallback"),
            Some(N16Tag::Gamma)
        );
        assert_eq!(extract_n16_tag("// neutro: N16[INVALID]"), None);
        assert_eq!(extract_n16_tag("// neutro: sem tag"), None);
    }

    #[test]
    fn extract_n16_module_name_maps_canonical_directories() {
        assert_eq!(
            extract_n16_module_name(Path::new("01_core/src/compiler/introspect/labelled.rs")),
            "compiler/"
        );
        assert_eq!(
            extract_n16_module_name(Path::new("/abs/path/01_core/src/compiler/introspect.rs")),
            "compiler/"
        );
        assert_eq!(
            extract_n16_module_name(Path::new("01_core/src/compiler/math/layout/attach.rs")),
            "compiler/"
        );
        assert_eq!(
            extract_n16_module_name(Path::new("01_core/src/compiler/math/layout/mod.rs")),
            "compiler/"
        );
        assert_eq!(
            extract_n16_module_name(Path::new("01_core/src/compiler/layout/columns.rs")),
            "compiler/"
        );
        assert_eq!(
            extract_n16_module_name(Path::new("01_core/src/entities/value.rs")),
            "entities/"
        );
        assert_eq!(
            extract_n16_module_name(Path::new("01_core/src/compiler/stdlib/calc.rs")),
            "compiler/"
        );
        assert_eq!(
            extract_n16_module_name(Path::new("01_core/src/compiler/eval/math.rs")),
            "compiler/"
        );
        assert_eq!(
            extract_n16_module_name(Path::new("01_core/src/compiler/parse/math.rs")),
            "compiler/"
        );
        assert_eq!(
            extract_n16_module_name(Path::new("03_infra/src/export/stream.rs")),
            "03_infra/"
        );
        assert_eq!(
            extract_n16_module_name(Path::new("/abs/path/03_infra/src/font_metrics.rs")),
            "03_infra/"
        );
    }

    #[test]
    fn collect_n16_stats_deduplicates_overlapping_source_and_toml() {
        // Cenário de sobreposição: mesmo arquivo e linha têm comentário no código E entrada no toml
        let source_code = "fn foo() {\n    match x {\n        _ => None, // neutro: N16[β] — comentário inline\n    }\n}";
        let sources = vec![SourceFile {
            path: PathBuf::from("/workspace/01_core/src/compiler/stdlib/calc.rs"),
            content: source_code.to_string(),
            language: Language::Rust,
            layer: Layer::L1,
            has_adjacent_test: true,
        }];

        let mut exceptions = HashMap::new();
        // Entrada no TOML referenciando a mesma linha 3
        exceptions.insert(
            "01_core/src/compiler/stdlib/calc.rs:3".to_string(),
            "N16[β]: entrada redundante no toml".to_string(),
        );
        // Entrada no TOML em linha diferente (linha 10)
        exceptions.insert(
            "01_core/src/compiler/stdlib/calc.rs:10".to_string(),
            "N16[γ]: caso exclusivo do toml".to_string(),
        );

        let stats = collect_n16_stats(&sources, &exceptions);
        let compiler = stats.get("compiler/").expect("expected compiler/");
        // Paths absoluto e relativo são identidades nominais distintas no P0094.
        assert_eq!(compiler.total(), 3);
        assert_eq!(compiler.beta, 2);
        assert_eq!(compiler.gamma, 1);
    }

    #[test]
    fn format_n16_summary_sorts_by_gamma_descending() {
        let mut stats = N16Stats::new();
        stats.insert(
            "introspect/".to_string(),
            N16ModuleStats {
                alpha: 0,
                beta: 1,
                gamma: 2,
            },
        );
        stats.insert(
            "layout/".to_string(),
            N16ModuleStats {
                alpha: 1,
                beta: 15,
                gamma: 4,
            },
        );
        stats.insert(
            "math/layout/".to_string(),
            N16ModuleStats {
                alpha: 0,
                beta: 1,
                gamma: 1,
            },
        );
        stats.insert(
            "entities/".to_string(),
            N16ModuleStats {
                alpha: 0,
                beta: 28,
                gamma: 0,
            },
        );

        let out = format_n16_summary(&stats, 5);

        // Layout (4 gamma) deve vir antes de introspect (2 gamma) e math/layout (1 gamma)
        let layout_idx = out.find("`layout/`").unwrap();
        let introspect_idx = out.find("`introspect/`").unwrap();
        let math_idx = out.find("`math/layout/`").unwrap();
        let entities_idx = out.find("`entities/`").unwrap();

        assert!(layout_idx < introspect_idx);
        assert!(introspect_idx < math_idx);
        assert!(math_idx < entities_idx);

        // Total e percentuais
        assert!(out.contains("| **Total** | **53** | **1** | **45** | **7** | **13.2%** |"));

        // Avisos de amostra pequena (n=3 e n=2)
        assert!(out.contains("⚠ amostra pequena em `introspect/` (n=3) — percentual pouco confiável, 1 caso muda o resultado em ~33pp"));
        assert!(out.contains("⚠ amostra pequena em `math/layout/` (n=2) — percentual pouco confiável, 1 caso muda o resultado em ~50pp"));
        assert!(!out.contains("⚠ amostra pequena em `layout/`"));
    }

    #[test]
    fn format_n16_summary_with_custom_min_sample_size() {
        let mut stats = N16Stats::new();
        stats.insert(
            "layout/".to_string(),
            N16ModuleStats {
                alpha: 1,
                beta: 15,
                gamma: 4,
            },
        ); // n=20
        stats.insert(
            "03_infra/".to_string(),
            N16ModuleStats {
                alpha: 0,
                beta: 11,
                gamma: 1,
            },
        ); // n=12

        let out = format_n16_summary(&stats, 25);
        assert!(out.contains("⚠ amostra pequena em `layout/` (n=20) — percentual pouco confiável, 1 caso muda o resultado em ~5pp"));
        assert!(out.contains("⚠ amostra pequena em `03_infra/` (n=12) — percentual pouco confiável, 1 caso muda o resultado em ~8pp"));
    }
}
