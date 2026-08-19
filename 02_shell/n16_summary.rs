//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/linter-core.md
//! @prompt-hash d9053635
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
            N16Tag::Beta  => "β",
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
            "0.0%".to_string()
        } else if self.gamma == 0 && self.beta == 0 {
            "—".to_string()
        } else {
            format!("{:.1}%", self.gamma_pct())
        }
    }
}

pub type N16Stats = BTreeMap<String, N16ModuleStats>;

/// Extrai tag N16 a partir de uma linha de texto ou justificativa.
pub fn extract_n16_tag(text: &str) -> Option<N16Tag> {
    let start_idx = text.find("N16[")?;
    let rest = &text[start_idx + 4..];
    let end_idx = rest.find(']')?;
    let inner = rest[..end_idx].trim();

    match inner {
        "α" | "A" | "a" => Some(N16Tag::Alpha),
        "β" | "B" | "b" => Some(N16Tag::Beta),
        "γ" | "C" | "c" => Some(N16Tag::Gamma),
        _ => None,
    }
}

/// Extrai o nome canônico do módulo para fins de agrupamento N16.
pub fn extract_n16_module_name(path: &Path) -> String {
    let p_str = path.to_string_lossy().replace('\\', "/");

    if p_str.contains("math/layout/") || p_str.ends_with("math/layout.rs") || p_str.ends_with("math/layout") {
        return "math/layout/".to_string();
    }
    if p_str.contains("export/") {
        return "export/".to_string();
    }

    let all_comps: Vec<String> = p_str
        .split('/')
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty() && s != "." && s != "..")
        .collect();

    if all_comps.is_empty() {
        return "root/".to_string();
    }

    let layer_idx = all_comps.iter().position(|c| {
        c == "01_core" || c == "02_shell" || c == "03_infra" || c == "04_wiring" || c == "00_nucleo"
    });

    let comps = if let Some(idx) = layer_idx {
        &all_comps[idx..]
    } else {
        &all_comps[..]
    };

    // Camadas de nível superior
    if comps[0] == "03_infra" {
        return "03_infra/".to_string();
    }
    if comps[0] == "02_shell" {
        return "02_shell/".to_string();
    }
    if comps[0] == "04_wiring" {
        return "04_wiring/".to_string();
    }
    if comps[0] == "00_nucleo" {
        return "00_nucleo/".to_string();
    }

    // Camada 01_core
    if comps[0] == "01_core" {
        let mut idx = 1;
        if idx < comps.len() && comps[idx] == "src" {
            idx += 1;
        }
        if idx >= comps.len() {
            return "01_core/".to_string();
        }
        if comps[idx] == "compiler" || comps[idx] == "engine" {
            idx += 1;
            if idx >= comps.len() {
                return "01_core/".to_string();
            }
        }

        let mut mod_name = comps[idx].clone();
        if let Some(pos) = mod_name.find('.') {
            mod_name.truncate(pos);
        }
        return format!("{mod_name}/");
    }

    // Workspace geral: primeiro diretório
    let mut root_mod = comps[0].clone();
    if let Some(pos) = root_mod.find('.') {
        root_mod.truncate(pos);
    }
    format!("{root_mod}/")
}

/// Coleta e agrega todas as anotações N16 de fontes e de exceções declaradas.
pub fn collect_n16_stats(
    sources: &[SourceFile],
    exceptions: &HashMap<String, String>,
) -> N16Stats {
    let mut seen_locs: HashSet<(String, usize)> = HashSet::new();
    let mut stats: N16Stats = BTreeMap::new();

    // 1. Varrer linhas de código-fonte
    for sf in sources {
        let path_str = sf.path.to_string_lossy().to_string();
        for (idx, line) in sf.content.lines().enumerate() {
            let line_num = idx + 1;
            if let Some(tag) = extract_n16_tag(line) {
                seen_locs.insert((path_str.clone(), line_num));
                let module = extract_n16_module_name(&sf.path);
                let entry = stats.entry(module).or_default();
                match tag {
                    N16Tag::Alpha => entry.alpha += 1,
                    N16Tag::Beta  => entry.beta += 1,
                    N16Tag::Gamma => entry.gamma += 1,
                }
            }
        }
    }

    // 2. Varrer exceções de crystalline.toml (wildcard_exceptions)
    for (loc_key, justification) in exceptions {
        if let Some(tag) = extract_n16_tag(justification) {
            let parts: Vec<&str> = loc_key.split(':').collect();
            let path_str = parts[0].to_string();
            let line_num = parts.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);

            if seen_locs.insert((path_str.clone(), line_num)) {
                let module = extract_n16_module_name(Path::new(&path_str));
                let entry = stats.entry(module).or_default();
                match tag {
                    N16Tag::Alpha => entry.alpha += 1,
                    N16Tag::Beta  => entry.beta += 1,
                    N16Tag::Gamma => entry.gamma += 1,
                }
            }
        }
    }

    stats
}

/// Formata o relatório consolidado N16 por módulo (Passo 0069).
pub fn format_n16_summary(stats: &N16Stats, min_sample_size: usize) -> String {
    let mut rows: Vec<(&String, &N16ModuleStats)> = stats.iter().collect();

    // Ordenação por γ absoluto decrescente, depois % γ decrescente, depois total decrescente, depois nome
    rows.sort_by(|(name_a, stats_a), (name_b, stats_b)| {
        stats_b.gamma.cmp(&stats_a.gamma)
            .then_with(|| {
                let pct_a = stats_a.gamma_pct();
                let pct_b = stats_b.gamma_pct();
                pct_b.partial_cmp(&pct_a).unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| stats_b.total().cmp(&stats_a.total()))
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
        if tot < min_sample_size && tot > 0 && s.gamma > 0 {
            let pp = (100.0 / tot as f64).round() as usize;
            small_sample_warnings.push(format!(
                "⚠ amostra pequena em `{}` (n={}) — percentual pouco confiável, 1 caso muda o resultado em ~{}pp",
                module_name, tot, pp
            ));
        }
    }

    let total_all = total_alpha + total_beta + total_gamma;
    let total_pct_str = if total_all == 0 {
        "0.0%".to_string()
    } else {
        format!("{:.1}%", (total_gamma as f64 / total_all as f64) * 100.0)
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

    #[test]
    fn extract_n16_tag_recognizes_greek_and_ascii_variants() {
        assert_eq!(extract_n16_tag("// neutro: N16[α] — fechamento"), Some(N16Tag::Alpha));
        assert_eq!(extract_n16_tag("// neutro: N16[β] — uniforme"), Some(N16Tag::Beta));
        assert_eq!(extract_n16_tag("// neutro: N16[γ] — fallback"), Some(N16Tag::Gamma));
        assert_eq!(extract_n16_tag("// neutro: N16[A] — fechamento"), Some(N16Tag::Alpha));
        assert_eq!(extract_n16_tag("// neutro: N16[B] — uniforme"), Some(N16Tag::Beta));
        assert_eq!(extract_n16_tag("// neutro: N16[C] — fallback"), Some(N16Tag::Gamma));
        assert_eq!(extract_n16_tag("// neutro: N16[a] — fechamento"), Some(N16Tag::Alpha));
        assert_eq!(extract_n16_tag("// neutro: N16[b] — uniforme"), Some(N16Tag::Beta));
        assert_eq!(extract_n16_tag("// neutro: N16[c] — fallback"), Some(N16Tag::Gamma));
        assert_eq!(extract_n16_tag("// neutro: N16[INVALID]"), None);
        assert_eq!(extract_n16_tag("// neutro: sem tag"), None);
    }

    #[test]
    fn extract_n16_module_name_maps_canonical_directories() {
        assert_eq!(extract_n16_module_name(Path::new("01_core/src/compiler/introspect/labelled.rs")), "introspect/");
        assert_eq!(extract_n16_module_name(Path::new("/abs/path/01_core/src/compiler/introspect.rs")), "introspect/");
        assert_eq!(extract_n16_module_name(Path::new("01_core/src/compiler/math/layout/attach.rs")), "math/layout/");
        assert_eq!(extract_n16_module_name(Path::new("01_core/src/compiler/math/layout/mod.rs")), "math/layout/");
        assert_eq!(extract_n16_module_name(Path::new("01_core/src/compiler/layout/columns.rs")), "layout/");
        assert_eq!(extract_n16_module_name(Path::new("01_core/src/entities/value.rs")), "entities/");
        assert_eq!(extract_n16_module_name(Path::new("01_core/src/compiler/stdlib/calc.rs")), "stdlib/");
        assert_eq!(extract_n16_module_name(Path::new("01_core/src/compiler/eval/math.rs")), "eval/");
        assert_eq!(extract_n16_module_name(Path::new("01_core/src/compiler/parse/math.rs")), "parse/");
        assert_eq!(extract_n16_module_name(Path::new("03_infra/src/export/stream.rs")), "export/");
        assert_eq!(extract_n16_module_name(Path::new("/abs/path/03_infra/src/font_metrics.rs")), "03_infra/");
    }

    #[test]
    fn format_n16_summary_sorts_by_gamma_descending() {
        let mut stats = N16Stats::new();
        stats.insert("introspect/".to_string(), N16ModuleStats { alpha: 0, beta: 1, gamma: 2 });
        stats.insert("layout/".to_string(), N16ModuleStats { alpha: 1, beta: 15, gamma: 4 });
        stats.insert("math/layout/".to_string(), N16ModuleStats { alpha: 0, beta: 1, gamma: 1 });
        stats.insert("entities/".to_string(), N16ModuleStats { alpha: 0, beta: 28, gamma: 0 });

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
        stats.insert("layout/".to_string(), N16ModuleStats { alpha: 1, beta: 15, gamma: 4 }); // n=20
        stats.insert("03_infra/".to_string(), N16ModuleStats { alpha: 0, beta: 11, gamma: 1 }); // n=12

        let out = format_n16_summary(&stats, 25);
        assert!(out.contains("⚠ amostra pequena em `layout/` (n=20) — percentual pouco confiável, 1 caso muda o resultado em ~5pp"));
        assert!(out.contains("⚠ amostra pequena em `03_infra/` (n=12) — percentual pouco confiável, 1 caso muda o resultado em ~8pp"));
    }
}
