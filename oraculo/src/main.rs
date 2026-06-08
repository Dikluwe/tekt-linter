//! Oráculo diferencial linter × lente (prompt 0058).
//!
//! Cruza DUAS computações independentes da estrutura de um workspace Rust:
//!   - **linter**: resolve imports por análise textual (`classify_import`), via o
//!     binário com `--emit-resolution` (JSON Lines, caixa-preta).
//!   - **lente** (`tekt-cargo-dsm`): resolve dependências pelo grafo do compilador
//!     (fork `cargo-modules`), via `lente_wiring::montar_grafo_workspace` (lib).
//!
//! Caminhos independentes → não compartilham pontos cegos. Onde discordam, um dos
//! dois tem um cego numa arquitetura real (o modo de falha da anamnese, virado p/ fora).
//!
//! Observável comum (v1): **arestas cross-crate first-party** `(crate-origem →
//! crate-alvo)` — o sinal forte (a aresta cross-crate é o que o linter cego punha
//! em `Unknown`). A camada é modo-comum (mesma projeção `[layers]` nos dois) — não
//! é sinal. Escopo removido SIMETRICAMENTE: arestas intra-crate, std/externas, e
//! qualquer alvo que não seja um crate-membro do workspace.
//!
//! Uso: `oraculo <workspace> <linter-bin>`
//!   - reporta cada discordância classificada; sai 0 sempre (é instrumentação).

use std::collections::{BTreeSet, HashMap};
use std::path::{Path as FsPath, PathBuf};
use std::process::Command;

use lente_core::entities::grafo::Relation;

/// Aresta cross-crate normalizada: (origem, alvo), nomes em forma de código (`_`).
type Edge = (String, String);

fn norm(name: &str) -> String {
    name.replace('-', "_")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("uso: oraculo <workspace> <linter-bin>");
        std::process::exit(2);
    }
    // Caminhos absolutos: a lente roda `cargo metadata` no cwd HERDADO em parte do
    // seu caminho de detecção-por-nome — então o oráculo fixa o cwd no workspace.
    let workspace = PathBuf::from(&args[1])
        .canonicalize()
        .expect("workspace inexistente");
    let linter_bin = PathBuf::from(&args[2])
        .canonicalize()
        .expect("binário do linter inexistente");
    std::env::set_current_dir(&workspace).expect("não consegui entrar no workspace");

    let (members, canon) = member_crates(&workspace);
    let member_names: BTreeSet<String> =
        members.iter().map(|(n, _)| n.clone()).collect();

    let linter = linter_edges(&linter_bin, &workspace, &members, &member_names);
    let lente = lente_edges(&workspace, &member_names, &canon);

    diff_and_report(&linter, &lente);
}

// ── Lado linter ────────────────────────────────────────────────────────────────

/// Roda o linter com `--emit-resolution` e projeta para arestas cross-crate
/// first-party (origem-crate → alvo-crate). Origem = crate dono do ficheiro;
/// alvo = `first_segment` do import quando é um crate-membro.
fn linter_edges(
    linter_bin: &FsPath,
    workspace: &FsPath,
    members: &[(String, PathBuf)],
    member_names: &BTreeSet<String>,
) -> BTreeSet<Edge> {
    let out = Command::new(linter_bin)
        .current_dir(workspace)
        .args(["--emit-resolution", "."])
        .output()
        .expect("falha ao rodar o linter --emit-resolution");
    let stdout = String::from_utf8_lossy(&out.stdout);

    let mut edges = BTreeSet::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let source = v["source"].as_str().unwrap_or("");
        let target = norm(v["first_segment"].as_str().unwrap_or(""));
        // alvo só conta se for um crate-membro (first-party); std/externas → fora (simétrico).
        if !member_names.contains(&target) {
            continue;
        }
        let Some(src_crate) = owner_crate(source, workspace, members) else {
            continue;
        };
        if src_crate == target {
            continue; // intra-crate → fora (simétrico)
        }
        edges.insert((src_crate, target));
    }
    edges
}

/// Lista de crates-membro (nome de pacote normalizado, diretório) via `cargo
/// metadata`, **mais** um mapa de canonicalização `nome-de-alvo → nome-de-pacote`.
/// Necessário porque o `cargo-modules` chaveia o grafo pelo nome do **alvo**
/// (ex.: o bin `lente` do pacote `lente_app`), não do pacote.
fn member_crates(workspace: &FsPath) -> (Vec<(String, PathBuf)>, HashMap<String, String>) {
    let out = Command::new("cargo")
        .current_dir(workspace)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .expect("falha ao rodar cargo metadata");
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("cargo metadata: JSON inválido");
    let mut members = Vec::new();
    let mut canon = HashMap::new();
    if let Some(pkgs) = v["packages"].as_array() {
        for p in pkgs {
            let name = norm(p["name"].as_str().unwrap_or(""));
            canon.insert(name.clone(), name.clone()); // pacote → ele mesmo
            // nomes de alvo (lib/bin) → nome do pacote
            if let Some(targets) = p["targets"].as_array() {
                for t in targets {
                    if let Some(tn) = t["name"].as_str() {
                        canon.insert(norm(tn), name.clone());
                    }
                }
            }
            if let Some(mp) = p["manifest_path"].as_str() {
                if let Some(dir) = FsPath::new(mp).parent() {
                    members.push((name.clone(), dir.to_path_buf()));
                }
            }
        }
    }
    (members, canon)
}

/// Crate dono de um ficheiro-fonte (prefixo de diretório mais longo).
/// `source` é relativo ao workspace (ex.: "./cli/src/main.rs").
fn owner_crate(
    source: &str,
    workspace: &FsPath,
    members: &[(String, PathBuf)],
) -> Option<String> {
    let abs = workspace.join(source.trim_start_matches("./"));
    let abs = abs.canonicalize().unwrap_or(abs);
    members
        .iter()
        .filter(|(_, dir)| {
            let dir = dir.canonicalize().unwrap_or_else(|_| dir.clone());
            abs.starts_with(&dir)
        })
        .max_by_key(|(_, dir)| dir.components().count())
        .map(|(name, _)| name.clone())
}

// ── Lado lente ───────────────────────────────────────────────────────────────

/// Arestas cross-crate first-party do grafo da lente: relação `Uses`, origem≠alvo,
/// ambos crates-membro. Origem/alvo = 1º segmento do `Path` canônico.
fn lente_edges(
    workspace: &FsPath,
    member_names: &BTreeSet<String>,
    canon: &HashMap<String, String>,
) -> BTreeSet<Edge> {
    let gw = lente_wiring::montar_grafo_workspace(workspace)
        .expect("lente: montar_grafo_workspace falhou (fork instalado? workspace compila?)");

    // Canonicaliza o 1º segmento (nome-de-alvo do cargo-modules → nome-de-pacote).
    let canonize = |seg: &str| -> String {
        let s = norm(seg);
        canon.get(&s).cloned().unwrap_or(s)
    };

    let mut edges = BTreeSet::new();
    for aresta in &gw.grafo.edges {
        if aresta.relation != Relation::Uses {
            continue; // Owns = contenção de módulo, não dependência
        }
        let src = canonize(first_seg(aresta.from.as_str()));
        let tgt = canonize(first_seg(aresta.to.as_str()));
        if src == tgt {
            continue;
        }
        if !member_names.contains(&src) || !member_names.contains(&tgt) {
            continue; // std/externas/fantasmas → fora (simétrico)
        }
        edges.insert((src, tgt));
    }
    edges
}

fn first_seg(path: &str) -> &str {
    path.split("::").next().unwrap_or(path)
}

// ── Diff + triagem ──────────────────────────────────────────────────────────

fn diff_and_report(linter: &BTreeSet<Edge>, lente: &BTreeSet<Edge>) {
    let so_lente: Vec<&Edge> = lente.difference(linter).collect();
    let so_linter: Vec<&Edge> = linter.difference(lente).collect();
    let acordo = linter.intersection(lente).count();

    println!("=== ORÁCULO DIFERENCIAL linter × lente ===");
    println!(
        "arestas cross-crate: linter={}, lente={}, acordo={}",
        linter.len(),
        lente.len(),
        acordo
    );
    println!();

    println!("[lente resolve, linter NÃO] — candidato a PONTO CEGO DO LINTER (sinal alto):");
    if so_lente.is_empty() {
        println!("  (nenhuma)");
    }
    for (s, t) in &so_lente {
        println!("  {s} -> {t}");
    }
    println!();

    println!("[linter resolve, lente NÃO] — macro/cfg/glob/re-export/artefato de projeção (triar):");
    if so_linter.is_empty() {
        println!("  (nenhuma)");
    }
    for (s, t) in &so_linter {
        println!("  {s} -> {t}");
    }
    println!();

    println!(
        "RESUMO: {} cego-linter, {} so-linter, {} acordo",
        so_lente.len(),
        so_linter.len(),
        acordo
    );
}
