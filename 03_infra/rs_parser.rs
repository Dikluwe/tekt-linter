//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/parsers/rust.md
//! @prompt-hash 6d5d9318
//! @layer L3
//! @updated 2026-06-09

use std::borrow::Cow;
use std::collections::HashSet;

use tree_sitter::{Node, Parser as TsParser};

use crate::contracts::file_provider::SourceFile;
use crate::contracts::language_parser::LanguageParser;
use crate::contracts::parse_error::ParseError;
use crate::contracts::prompt_reader::PromptReader;
use crate::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crate::entities::layer::{Language, Layer};
use crate::entities::parsed_file::{
    Declaration, DeclarationKind, FunctionSignature, Import, ImportKind, ModuleDecl, ParsedFile,
    PromptHeader, PublicInterface, StaticDeclaration, Token, TokenKind, TypeKind, TypeSignature,
};
use crate::entities::rule_traits::{
    BodyForm, Citation, CitationKind, ConstantKind, DecisionArm, DecisionExpr, ScrutineeForm,
    SemanticObservation, SemanticObservationKind, SourceConstant,
};
use crate::infra::config::{CrystallineConfig, SemanticContractsConfig};
use crate::infra::crate_registry::{CrateRegistry, MemberCrate};

// ── RustParser ────────────────────────────────────────────────────────────────

pub struct RustParser<R: PromptReader, S: PromptSnapshotReader> {
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
    /// Registro membro→camada do workspace-alvo. Vazio ⇒ classificação legada.
    pub registry: CrateRegistry,
}

impl<R: PromptReader, S: PromptSnapshotReader> RustParser<R, S> {
    pub fn new(
        prompt_reader: R,
        snapshot_reader: S,
        config: CrystallineConfig,
        registry: CrateRegistry,
    ) -> Self {
        Self {
            prompt_reader,
            snapshot_reader,
            config,
            registry,
        }
    }
}

impl<R: PromptReader, S: PromptSnapshotReader> LanguageParser for RustParser<R, S> {
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        if file.content.is_empty() {
            return Err(ParseError::EmptySource {
                path: file.path.clone(),
            });
        }

        if file.language != Language::Rust {
            return Err(ParseError::UnsupportedLanguage {
                path: file.path.clone(),
                language: file.language.clone(),
            });
        }

        let mut ts_parser = TsParser::new();
        ts_parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(|_| ParseError::SyntaxError {
                path: file.path.clone(),
                line: 0,
                column: 0,
                message: "Failed to load Rust grammar".to_string(),
            })?;

        let tree = ts_parser
            .parse(file.content.as_bytes(), None)
            .ok_or_else(|| ParseError::SyntaxError {
                path: file.path.clone(),
                line: 0,
                column: 0,
                message: "Parser returned None — possible timeout".to_string(),
            })?;

        let root = tree.root_node();

        if root.has_error() {
            let (line, column) = find_first_error_pos(root);
            return Err(ParseError::SyntaxError {
                path: file.path.clone(),
                line,
                column,
                message: "Syntax error detected in AST".to_string(),
            });
        }

        let source = file.content.as_bytes();

        // ── Header ────────────────────────────────────────────────────────────
        let (mut prompt_header, prompt_refs) = extract_header(&file.content);

        let prompt_file_exists = prompt_header
            .as_ref()
            .map(|h| self.prompt_reader.exists(h.prompt_path))
            .unwrap_or(false);

        if let Some(ref mut header) = prompt_header {
            header.current_hash = self.prompt_reader.read_hash(header.prompt_path);
        }

        // ── Imports ───────────────────────────────────────────────────────────
        // Contexto per-crate: o membro dono deste ficheiro fornece as deps
        // declaradas, distinguindo externo real de item local (ADR-0009 / 0052).
        let owner = self.registry.owner_of(file.path.as_path());
        let imports = extract_imports(root, source, &self.config, &self.registry, owner);

        // ── Tokens ────────────────────────────────────────────────────────────
        let tokens = extract_tokens(root, source);

        // ── Test coverage ─────────────────────────────────────────────────────
        let has_cfg_test = has_test_attribute(root, source);
        let is_decl_only = is_declaration_only(root, source);
        let has_test_coverage = has_cfg_test || file.has_adjacent_test || is_decl_only;

        // ── PublicInterface (V6) ───────────────────────────────────────────────
        let public_interface = extract_public_interface(root, source);
        let prompt_snapshot = prompt_header
            .as_ref()
            .and_then(|h| self.snapshot_reader.read_snapshot(h.prompt_path));

        // ── Declared traits (V11) ──────────────────────────────────────────
        let declared_traits =
            if file.layer == Layer::L1 && path_contains_segment(file.path.as_path(), "contracts") {
                extract_declared_traits(root, source)
            } else {
                vec![]
            };

        // ── Implemented traits (V11) ───────────────────────────────────────
        let implemented_traits = if matches!(file.layer, Layer::L2 | Layer::L3) {
            extract_implemented_traits(root, source)
        } else {
            vec![]
        };

        // ── Blanket impl traits (V11 — ADR-0015) ──────────────────────────
        let blanket_impl_traits = if matches!(file.layer, Layer::L1 | Layer::L2 | Layer::L3) {
            extract_blanket_impls(root, source)
        } else {
            vec![]
        };

        // ── Declarations (V12) ─────────────────────────────────────────────
        let declarations = extract_declarations(root, source);

        // ── Static declarations (V13) ──────────────────────────────────────
        let static_declarations = extract_static_declarations(root, source);

        // ── Module declarations (ADR-0013) ─────────────────────────────────
        let module_decls = extract_module_decls(root, source, &file.layer);

        // ── Decision expressions (V16–V20 — ADR-0016) ───────────────────────
        let decision_exprs = extract_decision_exprs(root, source);

        // ── Source constants (V21 — ADR-0016 / unsourced-constant.md) ────────
        let constants = extract_constants(root, source);

        // ── Semantic preservation observations (V23–V25 — ADR-0018) ───────
        let semantic_observations =
            extract_semantic_observations(root, source, file.path.as_path(), &self.config.semantic);

        Ok(ParsedFile {
            path: file.path.as_path(),
            layer: file.layer.clone(),
            language: file.language.clone(),
            prompt_header,
            prompt_file_exists,
            prompt_refs,
            has_test_coverage,
            imports,
            tokens,
            public_interface,
            prompt_snapshot,
            declared_traits,
            implemented_traits,
            blanket_impl_traits,
            declarations,
            static_declarations,
            module_decls,
            decision_exprs,
            constants,
            semantic_observations,
        })
    }
}

// ── Header extraction ─────────────────────────────────────────────────────────

fn extract_header<'a>(source: &'a str) -> (Option<PromptHeader<'a>>, Vec<&'a str>) {
    let mut prompt_path: Option<&'a str> = None;
    let mut prompt_hash: Option<&'a str> = None;
    let mut layer: Option<Layer> = None;
    let mut updated: Option<&'a str> = None;
    // V15: todos os valores `@prompt` do bloco, em ordem — len() >= 2 é
    // MultiPromptHeader (um ficheiro, um prompt).
    let mut prompt_refs: Vec<&'a str> = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("//!") {
            break;
        }
        let content = trimmed.trim_start_matches("//!").trim();

        if let Some(val) = content.strip_prefix("@prompt-hash ") {
            prompt_hash = Some(val.trim());
        } else if let Some(val) = content.strip_prefix("@prompt ") {
            prompt_refs.push(val.trim());
            // Comportamento de último-valor preservado para V1/V5 — é V15
            // quem bloqueia o caso ambíguo multi-@prompt.
            prompt_path = Some(val.trim());
        } else if let Some(val) = content.strip_prefix("@layer ") {
            layer = Some(parse_layer_tag(val.trim()));
        } else if let Some(val) = content.strip_prefix("@updated ") {
            updated = Some(val.trim());
        }
    }

    let header = prompt_path.map(|path| PromptHeader {
        prompt_path: path,
        prompt_hash,
        current_hash: None, // filled in after header extraction
        layer: layer.unwrap_or(Layer::Unknown),
        updated,
    });
    (header, prompt_refs)
}

fn parse_layer_tag(tag: &str) -> Layer {
    match tag {
        "L0" => Layer::L0,
        "L1" => Layer::L1,
        "L2" => Layer::L2,
        "L3" => Layer::L3,
        "L4" => Layer::L4,
        "Lab" | "lab" => Layer::Lab,
        _ => Layer::Unknown,
    }
}

// ── Import extraction ─────────────────────────────────────────────────────────

fn extract_imports<'a>(
    root: Node,
    source: &'a [u8],
    config: &CrystallineConfig,
    registry: &CrateRegistry,
    owner: Option<&MemberCrate>,
) -> Vec<Import<'a>> {
    let mut imports = Vec::new();
    // `cfg_test` arranca a false na raiz; vira true ao descer por um item
    // `#[cfg(test)]` (0061) — marca a origem de cada import (test vs produção).
    collect_imports(root, source, config, registry, owner, false, &mut imports);

    // Segunda fase (cego #2, 0060): referências cross-crate por caminho qualificado
    // FORA do `use`/`extern crate` — expressão, tipo, atributo/macro. Dedup por 1º
    // segmento contra os imports já coletados: um crate visto por `use` E por caminho
    // inline não pode virar duas arestas (secção C). `seen` parte dos `use` emitidos.
    let mut seen: HashSet<String> = imports.iter().map(|i| first_segment(i.path)).collect();
    collect_path_refs(
        root,
        source,
        config,
        registry,
        owner,
        false,
        &mut seen,
        &mut imports,
    );

    imports
}

/// `true` se `node` é um atributo `cfg(test)` — `#[cfg(test)]` (externo) ou
/// `#![cfg(test)]` (interno). Reusa o critério de `check_cfg_test` (0061).
fn is_cfg_test_attribute(node: Node, source: &[u8]) -> bool {
    matches!(node.kind(), "attribute_item" | "inner_attribute_item")
        && node_text(node, source).contains("cfg(test)")
}

/// Visita cada filho de `node` calculando se está em escopo de teste (0061). Um
/// `#[cfg(test)]` na grammar é **irmão** que decora o item imediatamente seguinte
/// (verificado: o `attribute_item` precede o `mod_item`, não é seu filho) — então o
/// flag pendente só se aplica ao próximo item não-atributo. O `#![cfg(test)]` interno
/// é tratado pela mesma via: como `is_cfg_test_attribute` casa `inner_attribute_item`
/// e atributos internos vêm sempre no início do bloco, o pendente marca todos os
/// itens seguintes. O escopo herdado por `cfg_test` propaga sempre.
fn for_each_child_in_test_scope(
    node: Node,
    source: &[u8],
    cfg_test: bool,
    mut visit: impl FnMut(Node, bool),
) {
    let mut pending = false;
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            let is_attr = matches!(child.kind(), "attribute_item" | "inner_attribute_item");
            if is_cfg_test_attribute(child, source) {
                pending = true;
            }
            visit(child, cfg_test || pending);
            // O atributo externo consome-se ao passar o item que decora.
            if !is_attr {
                pending = false;
            }
        }
    }
}

/// Coleta referências cross-crate por caminho qualificado fora do `use`/`extern
/// crate` — `scoped_identifier` (expressão), `scoped_type_identifier` (tipo),
/// caminhos em tipos genéricos, e qualificações de chamada (todas `scoped_*`),
/// além de `token_tree` de atributos/macros (parte B). Resolve o 1º segmento pelo
/// MESMO `classify_import`; emite só se cross-crate first-party ou externo de
/// verdade. Local (`crate::`/`self::`/`super::`) e `std`/`core`/`alloc` ficam fora
/// — a guarda contra trocar o falso-negativo por falso-positivo (0058: 0 só-linter).
fn collect_path_refs<'a>(
    node: Node,
    source: &'a [u8],
    config: &CrystallineConfig,
    registry: &CrateRegistry,
    owner: Option<&MemberCrate>,
    cfg_test: bool,
    seen: &mut HashSet<String>,
    imports: &mut Vec<Import<'a>>,
) {
    // `use`/`extern crate` já foram coletados na 1ª fase com o `ImportKind`
    // correcto — não reprocessar as suas sub-árvores como path-refs.
    if matches!(node.kind(), "use_declaration" | "extern_crate_declaration") {
        return;
    }

    match node.kind() {
        // Parte A — posições estruturadas: expressão e tipo. Caminhos aninhados
        // (`a::b` dentro de `a::b::c`) reincidem no mesmo 1º segmento → dedup os absorve.
        "scoped_identifier" | "scoped_type_identifier" => {
            let line = node.start_position().row + 1;
            try_emit_path_ref(
                node_text(node, source),
                line,
                config,
                registry,
                owner,
                cfg_test,
                seen,
                imports,
            );
        }
        // Parte B — atributo/macro: conteúdo vem como `token_tree`; varrer por
        // sequências `ident :: ident`. Limite honesto: caminhos gerados DENTRO do
        // corpo de uma macro que a grammar não estrutura ficam invisíveis (residual).
        "token_tree" => {
            scan_token_tree(
                node, source, config, registry, owner, cfg_test, seen, imports,
            );
        }
        _ => {}
    }

    // O escopo de teste de cada filho é decidido pelos atributos `#[cfg(test)]`
    // irmãos (0061), não herdado em bloco — ver `for_each_child_in_test_scope`.
    for_each_child_in_test_scope(node, source, cfg_test, |child, child_cfg| {
        collect_path_refs(
            child, source, config, registry, owner, child_cfg, seen, imports,
        );
    });
}

/// `true` se o 1º segmento é caminho local (`crate`/`self`/`super`/`Self`) ou
/// stdlib (`std`/`core`/`alloc`) — nunca uma aresta cross-crate. Excluir ANTES de
/// classificar: senão `crate::shell::X` inline viraria uma aresta intra-crate espúria.
fn is_local_or_std_first_segment(path: &str) -> bool {
    matches!(
        first_segment(path).as_str(),
        "crate" | "self" | "super" | "Self" | "std" | "core" | "alloc"
    )
}

/// Resolve e (se cross-crate de verdade e ainda não vista) emite uma aresta para
/// um caminho qualificado fora do `use`. `path` é fatia do buffer (`&'a str`).
fn try_emit_path_ref<'a>(
    path: &'a str,
    line: usize,
    config: &CrystallineConfig,
    registry: &CrateRegistry,
    owner: Option<&MemberCrate>,
    cfg_test: bool,
    seen: &mut HashSet<String>,
    imports: &mut Vec<Import<'a>>,
) {
    if is_local_or_std_first_segment(path) {
        return;
    }
    let key = first_segment(path);
    if key.is_empty() || seen.contains(&key) {
        return;
    }
    // Reusar a resolução existente — não duplicar lógica de camada. `classify_import`
    // só devolve `Resolved` para membro first-party, self-import, ou dep externa
    // declarada; item local (não membro/dep/std) cai em `LocalItem` e não vira aresta.
    if let ImportClass::Resolved(target_layer) = classify_import(path, config, registry, owner) {
        let target_subdir = resolve_subdir(path, config, &target_layer);
        seen.insert(key);
        imports.push(Import {
            path,
            line,
            kind: ImportKind::Direct,
            target_layer,
            target_subdir,
            is_test_origin: cfg_test,
        });
    }
}

/// Varre os filhos directos de um `token_tree` (atributo/macro) por inícios de
/// caminho — um `identifier` seguido de `::` e **não** precedido por `::` (i.e. o
/// começo de um caminho, não um segmento intermédio) — e resolve o **1º segmento**
/// (secção B). Granularidade de crate: emite apenas o nome do crate, sem
/// reconstruir o caminho completo a partir dos tokens. O `classify_import`/`V3`/
/// `V14` precisam só do 1º segmento; a **precisão de sub-caminho** (caminho
/// completo e, logo, o subdir do V9 a partir de atributos) é o **residual nomeado**
/// da parte B — caminhos em corpos de macro não-estruturados também ficam aqui.
fn scan_token_tree<'a>(
    node: Node,
    source: &'a [u8],
    config: &CrystallineConfig,
    registry: &CrateRegistry,
    owner: Option<&MemberCrate>,
    cfg_test: bool,
    seen: &mut HashSet<String>,
    imports: &mut Vec<Import<'a>>,
) {
    let n = node.child_count();
    for i in 0..n {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if child.kind() != "identifier" {
            continue;
        }
        let next_is_colon = node.child(i + 1).map(|c| c.kind() == "::").unwrap_or(false);
        let prev_is_colon = i > 0 && node.child(i - 1).map(|c| c.kind() == "::").unwrap_or(false);
        if next_is_colon && !prev_is_colon {
            let line = child.start_position().row + 1;
            try_emit_path_ref(
                node_text(child, source),
                line,
                config,
                registry,
                owner,
                cfg_test,
                seen,
                imports,
            );
        }
    }
}

fn collect_imports<'a>(
    node: Node,
    source: &'a [u8],
    config: &CrystallineConfig,
    registry: &CrateRegistry,
    owner: Option<&MemberCrate>,
    cfg_test: bool,
    imports: &mut Vec<Import<'a>>,
) {
    match node.kind() {
        "use_declaration" => {
            let line = node.start_position().row + 1;
            let path = use_declaration_path(node, source);
            let kind = if path.ends_with("::*") {
                ImportKind::Glob
            } else if path.contains(" as ") {
                ImportKind::Alias
            } else if path.contains('{') && path.contains('}') {
                ImportKind::Named
            } else {
                ImportKind::Direct
            };
            // Item local (ex.: `use EnumLocal::*`) não é import inter-crate/externo
            // — não emitir Import. O falso positivo do V14 (`Kind`) some sem tocar a regra.
            if let ImportClass::Resolved(target_layer) =
                classify_import(path, config, registry, owner)
            {
                let target_subdir = resolve_subdir(path, config, &target_layer);
                imports.push(Import {
                    path,
                    line,
                    kind,
                    target_layer,
                    target_subdir,
                    is_test_origin: cfg_test,
                });
            }
        }
        "extern_crate_declaration" => {
            let line = node.start_position().row + 1;
            let text = node_text(node, source);
            let path = text
                .trim_start_matches("extern crate ")
                .trim_end_matches(';')
                .trim();
            if let ImportClass::Resolved(target_layer) =
                classify_import(path, config, registry, owner)
            {
                let target_subdir = resolve_subdir(path, config, &target_layer);
                imports.push(Import {
                    path,
                    line,
                    kind: ImportKind::Direct,
                    target_layer,
                    target_subdir,
                    is_test_origin: cfg_test,
                });
            }
        }
        _ => {}
    }

    // Escopo de teste por filho decidido pelos atributos `#[cfg(test)]` irmãos (0061).
    for_each_child_in_test_scope(node, source, cfg_test, |child, child_cfg| {
        collect_imports(child, source, config, registry, owner, child_cfg, imports);
    });
}

/// Extract the path string from a `use_declaration` node.
fn use_declaration_path<'a>(node: Node, source: &'a [u8]) -> &'a str {
    // The argument is typically the second child after "use" keyword
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            let kind = child.kind();
            if kind != "use" && kind != ";" && kind != "pub" && kind != "visibility_modifier" {
                return node_text(child, source);
            }
        }
    }
    node_text(node, source)
        .trim_start_matches("use ")
        .trim_end_matches(';')
        .trim()
}

/// Resultado da classificação de um `use`/`extern crate`.
enum ImportClass {
    /// Emitir `Import` com esta camada alvo.
    Resolved(Layer),
    /// Item local (ex.: `use EnumLocal::*`) — não é import inter-crate/externo;
    /// não emitir `Import` (evita o falso positivo do V14 sem tocar a regra).
    LocalItem,
}

/// Primeiro segmento de um path de `use`, normalizado `-`→`_`.
/// Corta no primeiro de `::` ou ` as ` — o sufixo de alias de crate
/// (`use lente_catalogo as cat;`) não pode contaminar o nome do crate (cego #1, 0059).
fn first_segment(path: &str) -> String {
    let p = path.trim_start_matches('{').trim();
    let end = [p.find("::"), p.find(" as ")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(p.len());
    p[..end].trim().replace('-', "_")
}

/// Camada do segundo segmento de um path qualificado por crate.
/// Funciona para `crate::M::…`, `super::M::…` e `nome_do_crate::M::…`
/// (todos têm o módulo em `segments[1]`).
fn module_layer(path: &str, config: &CrystallineConfig) -> Layer {
    let segments: Vec<&str> = path.splitn(4, "::").collect();
    segments
        .get(1)
        .map(|m| config.layer_for_module(m))
        .unwrap_or(Layer::Unknown)
}

/// Classifica um import com conhecimento das dependências reais (ADR-0009 / 0052).
/// `owner` é o crate-membro dono do ficheiro que faz o import (contexto per-crate).
///
/// Ordem: intra-crate (`crate::`/`super::`) → stdlib → self-import por nome →
/// outro membro first-party → dep externa declarada → (sem owner) legado →
/// item local (não emitir).
fn classify_import(
    path: &str,
    config: &CrystallineConfig,
    registry: &CrateRegistry,
    owner: Option<&MemberCrate>,
) -> ImportClass {
    let p = path.trim_start_matches('{').trim();

    // 1. Intra-crate explícito — inalterado.
    if p.starts_with("crate::") || p.starts_with("super::") {
        return ImportClass::Resolved(module_layer(p, config));
    }

    let seg = first_segment(p);

    // 2. Stdlib — preservado (V14 isenta std/core/alloc; I/O fica com o V4).
    if matches!(seg.as_str(), "std" | "core" | "alloc") {
        return ImportClass::Resolved(Layer::Unknown);
    }

    // 3. Self-import pelo nome do próprio crate ≡ intra-crate (ex.: `crystalline_lint::…`,
    //    ou `use lente_filtro::…` num teste de integração do próprio pacote).
    //    `module_layer` resolve o sub-módulo (caso multi-camada como o próprio linter);
    //    se o sub-módulo não está mapeado em `[module_layers]`, cai na camada do
    //    próprio crate (`owner.layer`) — um self-import NUNCA é externo, então jamais
    //    pode virar `Unknown` e disparar V14 (resíduo corrigido após o 0053).
    if let Some(o) = owner {
        if o.name == seg {
            let by_module = module_layer(p, config);
            let layer = if by_module == Layer::Unknown {
                o.layer.clone()
            } else {
                by_module
            };
            return ImportClass::Resolved(layer);
        }
    }

    // 4. Outro membro first-party → camada do membro (V3 enxerga direção entre crates).
    //    Resolve a renomeação por-membro do owner (chave → pacote real) antes de
    //    testar a filiação — `use y::…` com `y = { package = "x" }` vê o crate `x`
    //    (cego #3, 0059). Sem rename, `real == seg`.
    let real = owner
        .and_then(|o| o.renames.get(&seg))
        .map(String::as_str)
        .unwrap_or(seg.as_str());
    if let Some(layer) = registry.member_layer(real) {
        return ImportClass::Resolved(layer);
    }

    // 5. Dep externa declarada pelo owner → externo (Unknown; V14 aplica no L1).
    if let Some(o) = owner {
        if o.deps.contains(&seg) {
            return ImportClass::Resolved(Layer::Unknown);
        }
        // 7. owner presente e o segmento não é membro/dep/stdlib → item local.
        return ImportClass::LocalItem;
    }

    // 6. Sem owner (ficheiro fora de qualquer membro) → comportamento legado.
    ImportClass::Resolved(Layer::Unknown)
}

/// Resolve o subdiretório de destino de um import para V9.
/// Retorna Some("entities") se import aponta para crate::entities::...
/// Inspeciona o segundo segmento do path — o nome do módulo de L1.
///
/// `target_layer` é a camada já classificada do import. Para `crate::`/`super::`
/// o comportamento legado é preservado bit-a-bit. Para membros first-party
/// resolvidos a L1 (cross-crate), o subdir é `segments[1]` — mas só quando o
/// import alcança um sub-módulo (≥3 segmentos: `crate::sub::Item`); um import de
/// 2 segmentos (`crate::Item`) usa a API re-exportada na raiz, não uma porta.
fn resolve_subdir<'a>(
    path: &'a str,
    config: &CrystallineConfig,
    target_layer: &Layer,
) -> Option<&'a str> {
    let path = path.trim_start_matches('{').trim();
    let segments: Vec<&'a str> = path.splitn(4, "::").collect();

    if path.starts_with("crate::") || path.starts_with("super::") {
        // Legado — segments[0] = "crate"|"super", segments[1] = nome do módulo.
        let module_name = segments.get(1).copied()?;
        if config.layer_for_module(module_name) == Layer::L1 {
            return Some(module_name);
        }
        return None;
    }

    // Cross-crate: membro first-party resolvido a L1 → subdir = segments[1],
    // apenas quando alcança um sub-módulo (≥3 segmentos).
    if *target_layer == Layer::L1 && segments.len() >= 3 {
        return segments.get(1).copied();
    }

    None
}

// ── PublicInterface extraction ────────────────────────────────────────────────

/// Extract the public interface from the top-level items of the source file.
fn extract_public_interface<'a>(root: Node, source: &'a [u8]) -> PublicInterface<'a> {
    let mut functions = Vec::new();
    let mut types = Vec::new();
    let mut reexports = Vec::new();

    for i in 0..root.child_count() {
        if let Some(child) = root.child(i) {
            if !is_pub_item(child, source) {
                continue;
            }
            match child.kind() {
                "function_item" => {
                    if let Some(sig) = extract_fn_sig(child, source) {
                        functions.push(sig);
                    }
                }
                "struct_item" => {
                    if let Some(sig) = extract_type_sig(child, source, TypeKind::Struct) {
                        types.push(sig);
                    }
                }
                "enum_item" => {
                    if let Some(sig) = extract_type_sig(child, source, TypeKind::Enum) {
                        types.push(sig);
                    }
                }
                "trait_item" => {
                    if let Some(sig) = extract_type_sig(child, source, TypeKind::Trait) {
                        types.push(sig);
                    }
                }
                "use_declaration" => {
                    reexports.push(use_declaration_path(child, source));
                }
                _ => {}
            }
        }
    }

    PublicInterface {
        functions,
        types,
        reexports,
    }
}

fn is_pub_item(node: Node, source: &[u8]) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "visibility_modifier" {
                let text = node_text(child, source);
                if text.starts_with("pub") {
                    return true;
                }
            }
        }
    }
    false
}

fn extract_fn_sig<'a>(node: Node, source: &'a [u8]) -> Option<FunctionSignature<'a>> {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(n, source))?;

    let params = node
        .child_by_field_name("parameters")
        .map(|p| extract_param_types(p, source))
        .unwrap_or_default();

    let return_type = node
        .child_by_field_name("return_type")
        .map(|rt| node_text(rt, source).trim_start_matches("->").trim());

    Some(FunctionSignature {
        name,
        params,
        return_type,
    })
}

fn extract_param_types<'a>(params_node: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut result = Vec::new();
    for i in 0..params_node.child_count() {
        if let Some(child) = params_node.child(i) {
            if child.kind() == "parameter" {
                if let Some(ty) = child.child_by_field_name("type") {
                    result.push(node_text(ty, source));
                }
            }
        }
    }
    result
}

fn extract_type_sig<'a>(node: Node, source: &'a [u8], kind: TypeKind) -> Option<TypeSignature<'a>> {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(n, source))?;

    let members = match &kind {
        TypeKind::Struct => node
            .child_by_field_name("body")
            .map(|b| extract_named_children(b, source, "field_declaration"))
            .unwrap_or_default(),
        TypeKind::Enum => node
            .child_by_field_name("body")
            .map(|b| extract_named_children(b, source, "enum_variant"))
            .unwrap_or_default(),
        TypeKind::Trait => node
            .child_by_field_name("body")
            .map(|b| extract_trait_method_names(b, source))
            .unwrap_or_default(),
        // OO types (Class/Interface/TypeAlias) are never produced by RustParser;
        // this arm is required for exhaustiveness when TsParser uses the same enum.
        TypeKind::Class | TypeKind::Interface | TypeKind::TypeAlias => vec![],
    };

    Some(TypeSignature {
        name,
        kind,
        members,
    })
}

fn extract_named_children<'a>(body: Node, source: &'a [u8], item_kind: &str) -> Vec<&'a str> {
    let mut result = Vec::new();
    for i in 0..body.child_count() {
        if let Some(child) = body.child(i) {
            if child.kind() == item_kind {
                if let Some(name_node) = child.child_by_field_name("name") {
                    result.push(node_text(name_node, source));
                }
            }
        }
    }
    result
}

fn extract_trait_method_names<'a>(body: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut result = Vec::new();
    for i in 0..body.child_count() {
        if let Some(child) = body.child(i) {
            if matches!(child.kind(), "function_signature_item" | "function_item") {
                if let Some(name_node) = child.child_by_field_name("name") {
                    result.push(node_text(name_node, source));
                }
            }
        }
    }
    result
}

// ── Token extraction ──────────────────────────────────────────────────────────

fn extract_tokens<'a>(root: Node, source: &'a [u8]) -> Vec<Token<'a>> {
    let mut tokens = Vec::new();
    collect_tokens(root, source, &mut tokens);
    tokens
}

fn collect_tokens<'a>(node: Node, source: &'a [u8], tokens: &mut Vec<Token<'a>>) {
    match node.kind() {
        "call_expression" => {
            if let Some(func_node) = node.child(0) {
                let symbol = Cow::Borrowed(node_text(func_node, source));
                let pos = node.start_position();
                tokens.push(Token {
                    symbol,
                    line: pos.row + 1,
                    column: pos.column,
                    kind: TokenKind::CallExpression,
                });
            }
        }
        "macro_invocation" => {
            // First child is the macro path/name
            if let Some(name_node) = node.child(0) {
                let symbol = Cow::Borrowed(node_text(name_node, source));
                let pos = node.start_position();
                tokens.push(Token {
                    symbol,
                    line: pos.row + 1,
                    column: pos.column,
                    kind: TokenKind::MacroInvocation,
                });
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_tokens(child, source, tokens);
        }
    }
}

// ── Test coverage helpers ─────────────────────────────────────────────────────

fn has_test_attribute(root: Node, source: &[u8]) -> bool {
    check_cfg_test(root, source)
}

fn check_cfg_test(node: Node, source: &[u8]) -> bool {
    if node.kind() == "attribute_item" || node.kind() == "inner_attribute_item" {
        let text = node_text(node, source);
        if text.contains("cfg(test)") {
            return true;
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if check_cfg_test(child, source) {
                return true;
            }
        }
    }
    false
}

/// Returns true if the file only declares traits/structs/enums without impl bodies.
/// Such files are exempt from V2.
fn is_declaration_only(root: Node, source: &[u8]) -> bool {
    !has_impl_with_functions(root, source)
}

fn has_impl_with_functions(node: Node, _source: &[u8]) -> bool {
    if node.kind() == "impl_item" {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "declaration_list" {
                    for j in 0..child.child_count() {
                        if let Some(item) = child.child(j) {
                            if item.kind() == "function_item" && node_has_child_kind(item, "block")
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if has_impl_with_functions(child, _source) {
                return true;
            }
        }
    }
    false
}

// ── Declared / Implemented traits / Declarations (ADR-0007) ──────────────────

/// Returns true if any component of `path` equals `segment` exactly.
fn path_contains_segment(path: &std::path::Path, segment: &str) -> bool {
    path.components()
        .any(|c| c.as_os_str().to_str().unwrap_or("") == segment)
}

/// Returns the last `::` segment of a trait path, stripping generic params.
/// `crate::contracts::FileProvider<'a>` → `"FileProvider"`
/// `LanguageParser` → `"LanguageParser"`
fn trait_last_segment(path_str: &str) -> &str {
    let base = path_str.rsplit("::").next().unwrap_or(path_str);
    base.split('<').next().unwrap_or(base).trim()
}

/// Extract names of public `trait` items at the top level of the AST.
/// Caller must gate on `L1/contracts` — this function does no filtering.
fn extract_declared_traits<'a>(root: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut traits = Vec::new();
    for i in 0..root.child_count() {
        if let Some(node) = root.child(i) {
            if node.kind() == "trait_item" && is_pub_item(node, source) {
                if let Some(name_node) = node.child_by_field_name("name") {
                    traits.push(node_text(name_node, source));
                }
            }
        }
    }
    traits
}

/// Extract trait names from top-level `impl Trait for Type` items.
/// Only items where the `trait` field is present are captured.
/// Caller must gate on `L2 | L3` — this function does no filtering.
fn extract_implemented_traits<'a>(root: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut traits = Vec::new();
    for i in 0..root.child_count() {
        if let Some(node) = root.child(i) {
            if node.kind() == "impl_item" {
                if let Some(trait_node) = node.child_by_field_name("trait") {
                    let trait_str = node_text(trait_node, source);
                    traits.push(trait_last_segment(trait_str));
                }
            }
        }
    }
    traits
}

/// Extract trait names satisfied by blanket impls — ADR-0015.
///
/// Detects three canonical patterns:
///   `impl<T: B> Trait for T`           (single bound)
///   `impl<T: B1 + B2> Trait for T`    (multi-bound)
///   `impl<T> Trait for T where T: B`  (where clause)
///
/// Pattern 4 (`impl<T: B> Trait for &T` / `Box<T>`) is intentionally
/// excluded — available via `[v11_blanket_exceptions]` in crystalline.toml.
///
/// Caller must gate on `L2 | L3` — this function does no filtering.
fn extract_blanket_impls<'a>(root: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut result = Vec::new();
    for i in 0..root.child_count() {
        if let Some(node) = root.child(i) {
            if node.kind() != "impl_item" {
                continue;
            }
            // Passo 1: recolher parâmetros genéricos do impl
            let type_params = node
                .child_by_field_name("type_parameters")
                .map(|n| collect_type_param_names(n, source))
                .unwrap_or_default();
            if type_params.is_empty() {
                continue; // impl concreto, já tratado por extract_implemented_traits
            }
            // Passo 2: verificar se o tipo em `for` é parâmetro genérico simples
            let for_type = node
                .child_by_field_name("type")
                .map(|n| node_text(n, source));
            let is_blanket = for_type
                .map(|t| type_params.iter().any(|p| *p == t))
                .unwrap_or(false);
            if !is_blanket {
                continue; // impl<T> Trait for ConcreteType — não é blanket
            }
            // Passo 3: extrair nome da trait
            if let Some(trait_node) = node.child_by_field_name("trait") {
                let trait_str = node_text(trait_node, source);
                result.push(trait_last_segment(trait_str));
            }
        }
    }
    result
}

/// Coleta os nomes dos parâmetros de tipo de um nó `type_parameters`.
/// Exemplo: `<T: World, U>` → `["T", "U"]`
/// tree-sitter resolve where clauses e multi-bounds no mesmo nó,
/// portanto os três padrões da ADR-0015 usam o mesmo algoritmo.
///
/// tree-sitter-rust 0.23 embrulha cada parâmetro num nó `type_parameter`
/// com field `name` (tanto `<T>` como `<T: Bound>`). As variantes
/// `type_identifier` solto e `constrained_type_parameter`/`left` são mantidas
/// por compatibilidade com grammars anteriores.
fn collect_type_param_names<'a>(node: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut names = Vec::new();
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                // grammar 0.23: `<T>` e `<T: Bound>` → type_parameter{ name }
                "type_parameter" => {
                    if let Some(name) = child.child_by_field_name("name") {
                        names.push(node_text(name, source));
                    }
                }
                // type_identifier solto — grammars anteriores
                "type_identifier" => names.push(node_text(child, source)),
                // constrained_type_parameter{ left } — grammars anteriores
                "constrained_type_parameter" => {
                    if let Some(left) = child.child_by_field_name("left") {
                        names.push(node_text(left, source));
                    }
                }
                // lifetimes ('a) não são parâmetros de tipo — ignorar
                _ => {}
            }
        }
    }
    names
}

/// Extract top-level struct/enum/impl-without-trait declarations for V12.
/// All files are processed — V12 filters by `layer == L4` internally.
fn extract_declarations<'a>(root: Node, source: &'a [u8]) -> Vec<Declaration<'a>> {
    let mut decls = Vec::new();
    for i in 0..root.child_count() {
        if let Some(node) = root.child(i) {
            match node.kind() {
                "struct_item" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        decls.push(Declaration {
                            kind: DeclarationKind::Struct,
                            name: node_text(name_node, source),
                            line: node.start_position().row + 1,
                        });
                    }
                }
                "enum_item" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        decls.push(Declaration {
                            kind: DeclarationKind::Enum,
                            name: node_text(name_node, source),
                            line: node.start_position().row + 1,
                        });
                    }
                }
                "impl_item" => {
                    // Only capture impl without trait: `impl Type { ... }`
                    if node.child_by_field_name("trait").is_none() {
                        if let Some(type_node) = node.child_by_field_name("type") {
                            decls.push(Declaration {
                                kind: DeclarationKind::Impl,
                                name: node_text(type_node, source),
                                line: node.start_position().row + 1,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }
    decls
}

/// Extract top-level static_item declarations for V13.
/// All files are processed — V13 filters by `layer == L1` internally.
fn extract_static_declarations<'a>(root: Node, source: &'a [u8]) -> Vec<StaticDeclaration<'a>> {
    let mut decls = Vec::new();
    for i in 0..root.child_count() {
        if let Some(node) = root.child(i) {
            if node.kind() == "static_item" {
                let is_mut = node_has_child_kind(node, "mutable_specifier");
                let name = node
                    .child_by_field_name("name")
                    .map(|n| node_text(n, source))
                    .unwrap_or("");
                let type_text = node
                    .child_by_field_name("type")
                    .map(|n| node_text(n, source))
                    .unwrap_or("");
                let line = node.start_position().row + 1;
                if !name.is_empty() {
                    decls.push(StaticDeclaration {
                        name,
                        type_text,
                        is_mut,
                        line,
                    });
                }
            }
        }
    }
    decls
}

/// Extract bare `mod foo;` declarations (no inline block body) for ADR-0013.
/// Inline `mod foo { }` blocks are skipped — they are not external module declarations.
/// The `target_layer` is the layer of the declaring file (same layer, different module).
fn extract_module_decls<'a>(
    root: Node,
    source: &'a [u8],
    file_layer: &Layer,
) -> Vec<ModuleDecl<'a>> {
    let mut decls = Vec::new();
    collect_module_decls(root, source, file_layer, &mut decls);
    decls
}

fn collect_module_decls<'a>(
    node: Node,
    source: &'a [u8],
    file_layer: &Layer,
    decls: &mut Vec<ModuleDecl<'a>>,
) {
    if node.kind() == "mod_item" && !node_has_child_kind(node, "declaration_list") {
        let line = node.start_position().row + 1;
        let text = node_text(node, source);
        let name = text
            .trim_start_matches("pub ")
            .trim_start_matches("mod ")
            .trim_end_matches(';')
            .trim();
        if !name.is_empty() {
            decls.push(ModuleDecl {
                name,
                target_layer: file_layer.clone(),
                line,
            });
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_module_decls(child, source, file_layer, decls);
        }
    }
}

// ── AST utilities ─────────────────────────────────────────────────────────────

fn node_text<'a>(node: Node, source: &'a [u8]) -> &'a str {
    std::str::from_utf8(&source[node.start_byte()..node.end_byte()]).unwrap_or("")
}

fn node_has_child_kind(node: Node, kind: &str) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == kind {
                return true;
            }
        }
    }
    false
}

fn find_first_error_pos(node: Node) -> (usize, usize) {
    if node.is_error() || node.is_missing() {
        let pos = node.start_position();
        return (pos.row + 1, pos.column);
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.has_error() || child.is_error() || child.is_missing() {
                let result = find_first_error_pos(child);
                if result.0 > 0 {
                    return result;
                }
            }
        }
    }
    // Fallback: usar a posição do próprio nó se tem erro mas sem filhos com erro
    if node.has_error() {
        let pos = node.start_position();
        if pos.row > 0 || pos.column > 0 {
            return (pos.row + 1, pos.column);
        }
    }
    (1, 0) // linha 1 como fallback mínimo — nunca reportar linha 0
}

// ── Decision Expressions Extraction (ADR-0016) ────────────────────────────────

fn extract_decision_exprs<'a>(root: Node, source: &'a [u8]) -> Vec<DecisionExpr<'a>> {
    let mut exprs = Vec::new();
    find_match_expressions(root, source, &mut exprs);
    exprs
}

fn find_match_expressions<'a>(node: Node, source: &'a [u8], acc: &mut Vec<DecisionExpr<'a>>) {
    if node.kind() == "match_expression" {
        if let Some(expr) = parse_match_expression(node, source) {
            acc.push(expr);
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_match_expressions(child, source, acc);
    }
}

fn parse_match_expression<'a>(node: Node, source: &'a [u8]) -> Option<DecisionExpr<'a>> {
    // Find scrutinee and match_block
    let mut scrutinee_node = None;
    let mut block_node = None;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "match_block" {
            block_node = Some(child);
        } else if child.kind() != "match"
            && child.is_named()
            && scrutinee_node.is_none()
            && block_node.is_none()
        {
            scrutinee_node = Some(child);
        }
    }

    let scrutinee = scrutinee_node?;
    let block = block_node?;

    let snippet_scrutinee =
        std::str::from_utf8(&source[scrutinee.start_byte()..scrutinee.end_byte()])
            .ok()?
            .trim();
    let scrutinee_form = classify_scrutinee(scrutinee, source);
    let span_pos = node.start_position();

    let mut arms = Vec::new();
    let mut block_cursor = block.walk();
    for child in block.children(&mut block_cursor) {
        if child.kind() == "match_arm" {
            if let Some(arm) = parse_match_arm(child, source) {
                arms.push(arm);
            }
        }
    }

    Some(DecisionExpr {
        snippet_scrutinee,
        scrutinee_form,
        arms,
        line: span_pos.row + 1,
        column: span_pos.column,
    })
}

fn classify_scrutinee(node: Node, _source: &[u8]) -> ScrutineeForm {
    match node.kind() {
        "call_expression" => {
            // Check if it is a method call like foo.bar() or self.kind()
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "field_expression" {
                    return ScrutineeForm::MethodCall;
                }
            }
            ScrutineeForm::MethodCall
        }
        "field_expression" => ScrutineeForm::FieldAccess,
        "index_expression" => ScrutineeForm::Index,
        "tuple_expression" => ScrutineeForm::Tuple,
        "identifier" | "scoped_identifier" => ScrutineeForm::Path,
        "integer_literal" | "float_literal" | "string_literal" | "char_literal"
        | "boolean_literal" => ScrutineeForm::Literal,
        _ => ScrutineeForm::Other,
    }
}

fn parse_match_arm<'a>(node: Node, source: &'a [u8]) -> Option<DecisionArm<'a>> {
    let mut pattern_node = None;
    let mut body_node = None;
    let mut passed_arrow = false;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "=>" {
            passed_arrow = true;
        } else if !passed_arrow && child.kind() == "match_pattern" {
            pattern_node = Some(child);
        } else if passed_arrow && child.is_named() && child.kind() != "," && body_node.is_none() {
            body_node = Some(child);
        }
    }

    let match_pattern = pattern_node?;
    let body = body_node?;

    // Inside match_pattern: pattern nodes before optional `if` node
    let mut pattern_end_byte = match_pattern.end_byte();
    let mut guard_node = None;
    let mut has_if = false;

    let mut pat_cursor = match_pattern.walk();
    for child in match_pattern.children(&mut pat_cursor) {
        if child.kind() == "if" {
            has_if = true;
            pattern_end_byte = child.start_byte();
        } else if has_if && guard_node.is_none() && child.is_named() {
            guard_node = Some(child);
        }
    }

    let pattern_bytes = &source[match_pattern.start_byte()..pattern_end_byte];
    let pattern_snippet = std::str::from_utf8(pattern_bytes).ok()?.trim();

    let is_catchall = check_is_catchall(match_pattern, pattern_end_byte, source);
    let bound_ident = get_catchall_ident(match_pattern, pattern_end_byte, source);

    let bound_ident_used_in_body = if let Some(ident) = bound_ident {
        if ident != "_" {
            has_ident_in_node(body, ident, source)
        } else {
            false
        }
    } else {
        false
    };

    let mut qualified_prefixes = Vec::new();
    extract_prefixes_from_pattern(
        match_pattern,
        pattern_end_byte,
        source,
        &mut qualified_prefixes,
    );

    let has_guard = has_if;
    let guard_is_compound = if let Some(g) = guard_node {
        check_compound_guard(g, source)
    } else {
        false
    };

    let pattern_is_range = check_pattern_is_range(match_pattern, pattern_end_byte, source);
    let pattern_depth = measure_pattern_depth(match_pattern, pattern_end_byte);
    let or_alternatives = count_or_alternatives(match_pattern, pattern_end_byte);

    let body_form = classify_body(body, source);

    let body_raw = std::str::from_utf8(&source[body.start_byte()..body.end_byte()]).unwrap_or("");
    let body_snippet = truncate_str_safe(body_raw.trim(), 80);

    let span_pos = node.start_position();

    Some(DecisionArm {
        pattern_snippet,
        is_catchall,
        bound_ident_used_in_body,
        qualified_prefixes,
        has_guard,
        guard_is_compound,
        pattern_is_range,
        pattern_depth,
        or_alternatives,
        body_form,
        body_snippet,
        line: span_pos.row + 1,
        column: span_pos.column,
    })
}

fn check_is_catchall(match_pattern: Node, limit_byte: usize, source: &[u8]) -> bool {
    let mut cursor = match_pattern.walk();
    for child in match_pattern.children(&mut cursor) {
        if child.end_byte() > limit_byte {
            continue;
        }
        match child.kind() {
            "_" => return true,
            "identifier" => {
                let name = std::str::from_utf8(&source[child.start_byte()..child.end_byte()])
                    .unwrap_or("");
                // In Rust, uppercase identifiers are likely unit struct patterns; lowercase/single ident are catchalls
                if !name.is_empty() {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn get_catchall_ident<'a>(
    match_pattern: Node,
    limit_byte: usize,
    source: &'a [u8],
) -> Option<&'a str> {
    let mut cursor = match_pattern.walk();
    for child in match_pattern.children(&mut cursor) {
        if child.end_byte() > limit_byte {
            continue;
        }
        if child.kind() == "_" {
            return Some("_");
        }
        if child.kind() == "identifier" {
            let name = std::str::from_utf8(&source[child.start_byte()..child.end_byte()]).ok()?;
            return Some(name);
        }
    }
    None
}

fn has_ident_in_node(node: Node, ident: &str, source: &[u8]) -> bool {
    if node.kind() == "identifier" {
        let name = std::str::from_utf8(&source[node.start_byte()..node.end_byte()]).unwrap_or("");
        if name == ident {
            return true;
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if has_ident_in_node(child, ident, source) {
            return true;
        }
    }
    false
}

fn extract_prefixes_from_pattern<'a>(
    node: Node,
    limit_byte: usize,
    source: &'a [u8],
    prefixes: &mut Vec<&'a str>,
) {
    if node.start_byte() >= limit_byte {
        return;
    }
    if node.kind() == "scoped_identifier" || node.kind() == "scoped_type_identifier" {
        // e.g. Unit::Pt -> prefix is "Unit"
        let full = std::str::from_utf8(&source[node.start_byte()..node.end_byte()]).unwrap_or("");
        if let Some(idx) = full.rfind("::") {
            let prefix = full[..idx].trim();
            // If prefix has multiple segments like a::b::Unit, take last segment or whole
            let base_prefix = if let Some(last_col) = prefix.rfind("::") {
                &prefix[last_col + 2..]
            } else {
                prefix
            };
            if !base_prefix.is_empty() && !prefixes.contains(&base_prefix) {
                prefixes.push(base_prefix);
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_prefixes_from_pattern(child, limit_byte, source, prefixes);
    }
}

fn check_compound_guard(node: Node, source: &[u8]) -> bool {
    if node.kind() == "binary_expression" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "&&" || child.kind() == "||" {
                return true;
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if check_compound_guard(child, source) {
            return true;
        }
    }
    false
}

fn check_pattern_is_range(node: Node, limit_byte: usize, source: &[u8]) -> bool {
    if node.start_byte() >= limit_byte {
        return false;
    }
    if node.kind() == "range_pattern" || node.kind() == "range_inclusive_pattern" {
        return true;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if check_pattern_is_range(child, limit_byte, source) {
            return true;
        }
    }
    false
}

fn measure_pattern_depth(node: Node, limit_byte: usize) -> u8 {
    if node.start_byte() >= limit_byte {
        return 0;
    }
    match node.kind() {
        "tuple_struct_pattern"
        | "struct_pattern"
        | "tuple_pattern"
        | "slice_pattern"
        | "reference_pattern" => {
            let mut max_child_depth = 0;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.is_named()
                    && child.kind() != "type_identifier"
                    && child.kind() != "identifier"
                    && child.kind() != "scoped_identifier"
                {
                    let d = measure_pattern_depth(child, limit_byte);
                    if d > max_child_depth {
                        max_child_depth = d;
                    }
                }
            }
            1 + max_child_depth.max(1)
        }
        _ => {
            let mut max_child = 0;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.is_named() {
                    let d = measure_pattern_depth(child, limit_byte);
                    if d > max_child {
                        max_child = d;
                    }
                }
            }
            if max_child > 0 {
                max_child
            } else {
                1
            }
        }
    }
}

fn count_or_alternatives(node: Node, limit_byte: usize) -> u16 {
    if node.start_byte() >= limit_byte {
        return 0;
    }
    if node.kind() == "or_pattern" {
        let mut count = 0;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                if child.kind() == "or_pattern" {
                    count += count_or_alternatives(child, limit_byte);
                } else {
                    count += 1;
                }
            }
        }
        return count.max(2);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let c = count_or_alternatives(child, limit_byte);
        if c > 1 {
            return c;
        }
    }
    1
}

fn classify_body(node: Node, source: &[u8]) -> BodyForm {
    let text = std::str::from_utf8(&source[node.start_byte()..node.end_byte()])
        .unwrap_or("")
        .trim();

    // Check for error barriers across expressions
    if text.starts_with("return Err")
        || text.starts_with("Err(")
        || text.starts_with("Err (")
        || text.contains("SourceDiagnostic::error")
        || text.contains("bail!(")
        || text.contains("panic!(")
        || text.contains("unreachable!(")
        || text.contains("todo!(")
        || text.contains("unimplemented!(")
        || text.contains("compile_error!(")
    {
        return BodyForm::ErrorBarrier;
    }

    match node.kind() {
        "macro_invocation" => {
            let macro_name = get_macro_name(node, source);
            match macro_name.as_str() {
                "panic" | "unreachable" | "bail" | "todo" | "unimplemented" | "compile_error" => {
                    BodyForm::ErrorBarrier
                }
                "format" | "format_args" | "write" | "writeln" => BodyForm::MessageProducer,
                "vec" => {
                    let clean = text.replace(' ', "");
                    if clean == "vec![]" || clean == "vec!()" {
                        BodyForm::LiteralNeutral
                    } else {
                        BodyForm::LiteralOther
                    }
                }
                "hash_map" | "hash_set" | "vec_deque" | "btree_map" | "btree_set" => {
                    BodyForm::LiteralOther
                }
                _ => BodyForm::LiteralOther,
            }
        }
        "call_expression" => {
            if text == "Default::default()"
                || text == "Default::default ()"
                || text == "String::new()"
                || text == "String::new ()"
                || text == "Vec::new()"
                || text == "Vec::new ()"
            {
                BodyForm::LiteralNeutral
            } else if is_error_message_fn_call(text) {
                BodyForm::MessageProducer
            } else {
                BodyForm::Call
            }
        }
        "scoped_identifier" => {
            if text.ends_with("::None") || text == "None" {
                BodyForm::LiteralNeutral
            } else {
                BodyForm::EnumPath
            }
        }
        "identifier" => {
            if text == "None" {
                BodyForm::LiteralNeutral
            } else if text
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
            {
                BodyForm::EnumPath
            } else {
                BodyForm::Other
            }
        }
        "boolean_literal" => BodyForm::LiteralNeutral,
        "integer_literal" => {
            if text == "0" {
                BodyForm::LiteralNeutral
            } else {
                BodyForm::LiteralOther
            }
        }
        "float_literal" => {
            if text == "0.0" || text == "0." {
                BodyForm::LiteralNeutral
            } else {
                BodyForm::LiteralOther
            }
        }
        "string_literal" => {
            if text == "\"\"" {
                BodyForm::LiteralNeutral
            } else {
                BodyForm::LiteralOther
            }
        }
        "unit_expression" => BodyForm::LiteralNeutral,
        "tuple_expression" => {
            let clean = text.replace(' ', "");
            if clean == "(0,0)"
                || clean == "(0,0,0)"
                || clean == "(0.0,0.0)"
                || clean == "()"
                || clean == "(None,None)"
                || clean == "(None,None,None)"
                || clean == "(None,None,None,None,None)"
            {
                BodyForm::LiteralNeutral
            } else {
                BodyForm::LiteralOther
            }
        }
        "return_expression" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.is_named() {
                    return classify_body(child, source);
                }
            }
            BodyForm::LiteralNeutral
        }
        "block" => {
            let mut named_children = Vec::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.is_named() {
                    named_children.push(child);
                }
            }
            if named_children.is_empty() {
                BodyForm::EmptyBlock
            } else if named_children.len() == 1 {
                classify_body(named_children[0], source)
            } else {
                let last = named_children.last().unwrap();
                let last_form = classify_body(*last, source);
                if matches!(
                    last_form,
                    BodyForm::ErrorBarrier | BodyForm::MessageProducer
                ) {
                    last_form
                } else {
                    BodyForm::Other
                }
            }
        }
        "continue_expression" | "break_expression" => BodyForm::Continue,
        _ => BodyForm::Other,
    }
}

fn is_error_message_fn_call(text: &str) -> bool {
    let t = text.to_lowercase();
    t.starts_with("error")
        || t.starts_with("err_")
        || t.starts_with("cannot_")
        || t.starts_with("expected_")
        || t.contains("::error(")
        || t.contains("::err_")
        || t.contains("::cannot_")
        || t.contains("::expected_")
}

fn truncate_str_safe(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

fn get_macro_name(node: Node, source: &[u8]) -> String {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" || child.kind() == "scoped_identifier" {
            let raw =
                std::str::from_utf8(&source[child.start_byte()..child.end_byte()]).unwrap_or("");
            if let Some(idx) = raw.rfind("::") {
                return raw[idx + 2..].to_string();
            }
            return raw.to_string();
        }
    }
    String::new()
}

// ── Semantic preservation observations (V23–V25 — ADR-0018) ─────────────────

fn extract_semantic_observations(
    root: Node,
    source: &[u8],
    path: &std::path::Path,
    contracts: &SemanticContractsConfig,
) -> Vec<SemanticObservation> {
    if contracts.context.is_empty()
        && contracts.projection.is_empty()
        && contracts.decision.is_empty()
    {
        return Vec::new();
    }
    let mut out = Vec::new();
    collect_semantic_functions(root, source, path, contracts, &mut out);
    out
}

fn collect_semantic_functions(
    node: Node,
    source: &[u8],
    path: &std::path::Path,
    contracts: &SemanticContractsConfig,
    out: &mut Vec<SemanticObservation>,
) {
    if node.kind() == "function_item" {
        analyze_semantic_function(node, source, path, contracts, out);
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_semantic_functions(child, source, path, contracts, out);
    }
}

fn analyze_semantic_function(
    function: Node,
    source: &[u8],
    path: &std::path::Path,
    contracts: &SemanticContractsConfig,
    out: &mut Vec<SemanticObservation>,
) {
    let Some(name_node) = function.child_by_field_name("name") else {
        return;
    };
    let Ok(name) = name_node.utf8_text(source) else {
        return;
    };
    let path_text = path.to_string_lossy().replace('\\', "/");
    let scope = format!("{path_text}::{name}");
    let Some(body) = function.child_by_field_name("body") else {
        return;
    };
    let body_text = body.utf8_text(source).unwrap_or("");

    for contract in &contracts.context {
        if !rust_contract(&contract.language)
            || !contract
                .scopes
                .iter()
                .any(|candidate| scope_matches(candidate, &scope))
        {
            continue;
        }
        let sink_present = contract
            .sinks
            .iter()
            .any(|sink| sink == name || body_text.contains(sink) || sink == "$return");
        if sink_present {
            collect_context_observations(body, source, contract, out);
        }
    }

    for contract in &contracts.projection {
        if !rust_contract(&contract.language)
            || !scope_matches(&contract.scope, &scope)
            || contract.normalization != "preserve"
        {
            continue;
        }
        if let Some(slot) = contract
            .destination
            .strip_prefix("return.")
            .and_then(|value| value.parse::<usize>().ok())
        {
            collect_projection_observations(body, source, contract, slot, out);
        }
    }

    for contract in &contracts.decision {
        if !rust_contract(&contract.language) {
            continue;
        }
        if contract
            .duplicate_owners
            .iter()
            .any(|candidate| scope_matches(candidate, &scope))
        {
            out.push(SemanticObservation {
                contract_id: contract.id.clone(),
                kind: SemanticObservationKind::DuplicateDecisionOwner,
                detail: format!(
                    "segundo owner `{scope}`; owner canônico `{}`",
                    contract.owner
                ),
                line: function.start_position().row + 1,
                column: function.start_position().column,
            });
        }
        if contract
            .consumers
            .iter()
            .any(|candidate| scope_matches(candidate, &scope))
        {
            collect_proxy_observations(body, source, contract, out);
        }
        if contract
            .resolved_after
            .iter()
            .any(|candidate| scope_matches(candidate, &scope))
        {
            collect_canonicalizer_observations(body, source, contract, out);
        }
    }
}

fn rust_contract(language: &str) -> bool {
    language.is_empty() || language.eq_ignore_ascii_case("rust")
}

fn scope_matches(pattern: &str, scope: &str) -> bool {
    let pattern = pattern.replace('\\', "/");
    if let Some(prefix) = pattern.strip_suffix("::*") {
        return scope.contains(prefix);
    }
    scope == pattern || scope.ends_with(&pattern)
}

fn selector_matches(pattern: &str, value: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix(".*") {
        value == prefix || value.starts_with(&format!("{prefix}."))
    } else {
        value == pattern || value.starts_with(&format!("{pattern}."))
    }
}

fn is_zero_context(text: &str) -> bool {
    let normalized = text
        .trim()
        .trim_end_matches(|c: char| c.is_ascii_alphabetic() || c == '_');
    normalized
        .parse::<f64>()
        .map(|value| value == 0.0)
        .unwrap_or(false)
}

fn collect_context_observations(
    node: Node,
    source: &[u8],
    contract: &crate::infra::config::ContextSemanticContract,
    out: &mut Vec<SemanticObservation>,
) {
    if node.kind() == "call_expression" {
        if let (Some(function), Some(arguments)) = (
            node.child_by_field_name("function"),
            node.child_by_field_name("arguments"),
        ) {
            let function_text = function.utf8_text(source).unwrap_or("");
            let receiver = function
                .child_by_field_name("value")
                .and_then(|value| value.utf8_text(source).ok())
                .unwrap_or("");
            for resolver in &contract.resolvers {
                let symbol_match = function_text == resolver.symbol
                    || function_text.ends_with(&format!(".{}", resolver.symbol))
                    || function_text.ends_with(&format!("::{}", resolver.symbol));
                let source_match = contract
                    .sources
                    .iter()
                    .any(|source_pattern| selector_matches(source_pattern, receiver));
                let absolute = contract
                    .absolute_sources
                    .iter()
                    .any(|source_pattern| selector_matches(source_pattern, receiver));
                if symbol_match && source_match && !absolute {
                    if let Some(argument) = arguments.named_child(resolver.context_arg) {
                        let arg_text = argument.utf8_text(source).unwrap_or("");
                        if is_zero_context(arg_text) {
                            out.push(SemanticObservation {
                                contract_id: contract.id.clone(),
                                kind: SemanticObservationKind::ContextNeutralArgument,
                                detail: format!(
                                    "`{receiver}` resolvido por `{}` com contexto `{arg_text}`",
                                    resolver.symbol
                                ),
                                line: node.start_position().row + 1,
                                column: node.start_position().column,
                            });
                        }
                    }
                }
            }
        }
    } else if node.kind() == "field_expression" {
        if let (Some(value), Some(field)) = (
            node.child_by_field_name("value"),
            node.child_by_field_name("field"),
        ) {
            let receiver = value.utf8_text(source).unwrap_or("");
            let field_text = field.utf8_text(source).unwrap_or("");
            let source_match = contract
                .sources
                .iter()
                .any(|source_pattern| selector_matches(source_pattern, receiver));
            if source_match
                && contract
                    .erasing_projections
                    .iter()
                    .any(|projection| projection == field_text)
            {
                out.push(SemanticObservation {
                    contract_id: contract.id.clone(),
                    kind: SemanticObservationKind::ContextErasingProjection,
                    detail: format!("projeção `{field_text}` apaga contexto de `{receiver}`"),
                    line: node.start_position().row + 1,
                    column: node.start_position().column,
                });
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_context_observations(child, source, contract, out);
    }
}

fn collect_projection_observations(
    node: Node,
    source: &[u8],
    contract: &crate::infra::config::ProjectionSemanticContract,
    slot: usize,
    out: &mut Vec<SemanticObservation>,
) {
    if node.kind() == "tuple_expression" {
        if let Some(destination) = node.named_child(slot) {
            let text = destination.utf8_text(source).unwrap_or("");
            let depends_on_source = text.contains(&contract.source);
            let neutral = contract
                .neutral_forms
                .iter()
                .any(|form| match form.as_str() {
                    "default" => text.contains("default()"),
                    "none" => text.trim() == "None",
                    "zero" => is_zero_context(text),
                    other => text.trim() == other,
                });
            if neutral && !depends_on_source {
                out.push(SemanticObservation {
                    contract_id: contract.id.clone(),
                    kind: SemanticObservationKind::NeutralProjectionDestination,
                    detail: format!(
                        "`{}` não alcança `{}`; destino neutro `{}`",
                        contract.source,
                        contract.destination,
                        text.trim()
                    ),
                    line: destination.start_position().row + 1,
                    column: destination.start_position().column,
                });
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_projection_observations(child, source, contract, slot, out);
    }
}

fn collect_proxy_observations(
    node: Node,
    source: &[u8],
    contract: &crate::infra::config::DecisionSemanticContract,
    out: &mut Vec<SemanticObservation>,
) {
    if node.kind() == "binary_expression" {
        let text = node.utf8_text(source).unwrap_or("");
        let explicit = contract
            .explicit_sources
            .iter()
            .any(|value| text.contains(value));
        let proxy = contract
            .proxies
            .iter()
            .find(|value| text.contains(value.as_str()));
        if text.contains("||") && explicit {
            if let Some(proxy) = proxy {
                out.push(SemanticObservation {
                    contract_id: contract.id.clone(),
                    kind: SemanticObservationKind::DecisionProxyReentry,
                    detail: format!("consumidor recombina decisão explícita com proxy `{proxy}`"),
                    line: node.start_position().row + 1,
                    column: node.start_position().column,
                });
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_proxy_observations(child, source, contract, out);
    }
}

fn collect_canonicalizer_observations(
    node: Node,
    source: &[u8],
    contract: &crate::infra::config::DecisionSemanticContract,
    out: &mut Vec<SemanticObservation>,
) {
    if node.kind() == "call_expression" {
        if let Some(function) = node.child_by_field_name("function") {
            let text = function.utf8_text(source).unwrap_or("");
            if let Some(symbol) = contract
                .canonicalizers
                .iter()
                .find(|symbol| text == symbol.as_str() || text.ends_with(&format!("::{}", symbol)))
            {
                out.push(SemanticObservation {
                    contract_id: contract.id.clone(),
                    kind: SemanticObservationKind::CanonicalizerReentry,
                    detail: format!("canonicalizador `{symbol}` reexecutado após marco resolvido"),
                    line: node.start_position().row + 1,
                    column: node.start_position().column,
                });
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_canonicalizer_observations(child, source, contract, out);
    }
}

// ── Source Constants Extraction (V21/V22 — Passo 0066) ─────────────────────────

fn extract_constants<'a>(root: Node, source: &'a [u8]) -> Vec<SourceConstant<'a>> {
    let citations = extract_citations(source);
    let mut constants = Vec::new();
    collect_constants(
        root,
        source,
        &citations,
        false,
        None,
        false,
        false,
        false,
        None,
        None,
        &mut constants,
    );
    constants
}

fn extract_citations<'a>(source: &'a [u8]) -> std::collections::HashMap<usize, Citation<'a>> {
    let mut citations = std::collections::HashMap::new();
    let text = match std::str::from_utf8(source) {
        Ok(t) => t,
        Err(_) => return citations,
    };

    for (idx, line) in text.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();
        if let Some(comment_content) = trimmed.strip_prefix("//") {
            let c_trimmed = comment_content.trim_start_matches('/').trim();

            // 1. Prefixos explícitos
            if let Some(idx_ref) = c_trimmed.find("ref:") {
                let rest = c_trimmed[idx_ref + 4..].trim();
                if let Some((path, line_part)) = rest.split_once(':') {
                    let line_digits: String = line_part
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if let Ok(l) = line_digits.parse::<usize>() {
                        citations.insert(
                            line_num,
                            Citation {
                                kind: CitationKind::Ref {
                                    path: path.trim(),
                                    line: l,
                                },
                                raw: trimmed,
                                line: line_num,
                            },
                        );
                        continue;
                    }
                }
            }
            if let Some(idx_spec) = c_trimmed.find("spec:") {
                let rest = c_trimmed[idx_spec + 5..].trim();
                citations.insert(
                    line_num,
                    Citation {
                        kind: CitationKind::Spec(rest),
                        raw: trimmed,
                        line: line_num,
                    },
                );
                continue;
            }
            if let Some(idx_rat) = c_trimmed.find("rationale:") {
                let rest = c_trimmed[idx_rat + 10..].trim();
                citations.insert(
                    line_num,
                    Citation {
                        kind: CitationKind::Rationale(rest),
                        raw: trimmed,
                        line: line_num,
                    },
                );
                continue;
            }

            // 2. Detecção de file:line em qualquer ponto do comentário
            // Tokens separados por espaços, parênteses, colchetes, vírgulas, travessão
            let mut found_ref = false;
            for token in c_trimmed.split(|c: char| {
                c.is_whitespace()
                    || c == '('
                    || c == ')'
                    || c == '['
                    || c == ']'
                    || c == '—'
                    || c == ','
                    || c == '|'
                    || c == ';'
            }) {
                if token.is_empty() {
                    continue;
                }
                if let Some((path, line_part)) = token.split_once(':') {
                    let line_digits: String = line_part
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if let Ok(l) = line_digits.parse::<usize>() {
                        let exts = [
                            ".rs", ".md", ".typ", ".toml", ".c", ".h", ".cpp", ".py", ".ts",
                            ".txt", ".js",
                        ];
                        let has_ext = exts.iter().any(|ext| path.ends_with(ext));
                        let has_slash = path.contains('/');
                        let is_known_stem = path.contains("resolve")
                            || path.contains("container")
                            || path.contains("layout")
                            || path.contains("math");
                        if has_ext || has_slash || is_known_stem {
                            citations.insert(
                                line_num,
                                Citation {
                                    kind: CitationKind::Ref {
                                        path: path.trim(),
                                        line: l,
                                    },
                                    raw: trimmed,
                                    line: line_num,
                                },
                            );
                            found_ref = true;
                            break;
                        }
                    }
                }
            }
            if found_ref {
                continue;
            }

            // 3. Menção a passo (ex: P813, P1042) ou especificação
            let contains_step = c_trimmed.split_whitespace().any(|word| {
                let w = word.trim_matches(|c: char| !c.is_alphanumeric());
                w.starts_with('P') && w.len() >= 2 && w[1..].chars().all(|c| c.is_ascii_digit())
            });
            if contains_step {
                citations.insert(
                    line_num,
                    Citation {
                        kind: CitationKind::Spec(c_trimmed),
                        raw: trimmed,
                        line: line_num,
                    },
                );
            }
        }
    }

    citations
}

fn is_numeric_format_specifier(text: &str) -> bool {
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut spec = String::new();
            while let Some(&next) = chars.peek() {
                chars.next();
                if next == '}' {
                    break;
                }
                spec.push(next);
            }
            if let Some((_, fmt)) = spec.split_once(':') {
                if fmt.contains('.') || fmt.chars().any(|ch| ch.is_ascii_digit()) {
                    return true;
                }
            }
        }
    }
    false
}

/// Detecta se um match é uma tabela de dados (>= 5 braços com corpo literal).
fn is_data_table_match(node: Node, _source: &[u8]) -> bool {
    let mut arm_count = 0;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "match_block" {
            let mut block_cursor = child.walk();
            for arm in child.children(&mut block_cursor) {
                if arm.kind() == "match_arm" {
                    arm_count += 1;
                }
            }
        }
    }
    arm_count >= 5
}

fn extract_sink_from_node(node: Node, source: &[u8]) -> Option<String> {
    match node.kind() {
        "let_declaration" => {
            if let Some(pat) = node.child_by_field_name("pattern") {
                return Some(node_text(pat, source).trim().to_string());
            }
        }
        "assignment_expression" | "compound_assignment_expr" => {
            if let Some(left) = node.child_by_field_name("left") {
                return Some(node_text(left, source).trim().to_string());
            }
        }
        "field_initializer" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "field_identifier" {
                    return Some(node_text(child, source).trim().to_string());
                }
            }
        }
        "call_expression" => {
            if let Some(function) = node.child_by_field_name("function") {
                return Some(node_text(function, source).trim().to_string());
            }
        }
        _ => {}
    }
    None
}

fn collect_constants<'a>(
    node: Node,
    source: &'a [u8],
    citations: &std::collections::HashMap<usize, Citation<'a>>,
    cfg_test: bool,
    fn_return_type: Option<&'a str>,
    in_fn_body: bool,
    in_pattern: bool,
    in_data_table: bool,
    active_sink: Option<String>,
    scaling_context_var: Option<String>,
    acc: &mut Vec<SourceConstant<'a>>,
) {
    let kind = node.kind();
    let pos = node.start_position();
    let line_num = pos.row + 1;
    let col = pos.column;

    let get_citation = || -> Option<Citation<'a>> {
        for offset in 0..=3 {
            if line_num >= offset + 1 {
                if let Some(c) = citations.get(&(line_num - offset)) {
                    return Some(c.clone());
                }
            }
        }
        None
    };

    // Atualiza o sink ativo se o nó corrente for uma declaração/atribuição/chamada
    let current_sink = extract_sink_from_node(node, source).or(active_sink.clone());

    match kind {
        "const_item" | "static_item" => {
            let snippet = node_text(node, source).trim();
            acc.push(SourceConstant {
                kind: ConstantKind::ItemDefinition,
                snippet,
                line: line_num,
                column: col,
                citation: get_citation(),
                is_test_origin: cfg_test,
                function_return_type: fn_return_type,
                is_in_binary_scaling: false,
                context_var: None,
                geometric_sink: current_sink.clone(),
                is_in_data_table: in_data_table,
            });
            return;
        }
        "binary_expression" => {
            let mut op = None;
            let mut left = None;
            let mut right = None;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "*" || child.kind() == "/" {
                    op = Some(child.kind());
                } else if left.is_none() {
                    left = Some(child);
                } else if right.is_none() && op.is_some() {
                    right = Some(child);
                }
            }

            if let (Some(_operator), Some(l_node), Some(r_node)) = (op, left, right) {
                // Checa se um dos lados é literal numérico e o outro é identificador/campo
                let l_is_lit =
                    l_node.kind() == "integer_literal" || l_node.kind() == "float_literal";
                let r_is_lit =
                    r_node.kind() == "integer_literal" || r_node.kind() == "float_literal";

                if l_is_lit && !r_is_lit {
                    let var_name = node_text(r_node, source).trim().to_string();
                    collect_constants(
                        l_node,
                        source,
                        citations,
                        cfg_test,
                        fn_return_type,
                        in_fn_body,
                        in_pattern,
                        in_data_table,
                        current_sink.clone(),
                        Some(var_name),
                        acc,
                    );
                    collect_constants(
                        r_node,
                        source,
                        citations,
                        cfg_test,
                        fn_return_type,
                        in_fn_body,
                        in_pattern,
                        in_data_table,
                        current_sink.clone(),
                        None,
                        acc,
                    );
                    return;
                } else if r_is_lit && !l_is_lit {
                    let var_name = node_text(l_node, source).trim().to_string();
                    collect_constants(
                        l_node,
                        source,
                        citations,
                        cfg_test,
                        fn_return_type,
                        in_fn_body,
                        in_pattern,
                        in_data_table,
                        current_sink.clone(),
                        None,
                        acc,
                    );
                    collect_constants(
                        r_node,
                        source,
                        citations,
                        cfg_test,
                        fn_return_type,
                        in_fn_body,
                        in_pattern,
                        in_data_table,
                        current_sink.clone(),
                        Some(var_name),
                        acc,
                    );
                    return;
                }
            }
        }
        "unary_expression" => {
            let mut cursor = node.walk();
            let mut has_minus = false;
            let mut literal_child = None;
            for child in node.children(&mut cursor) {
                if child.kind() == "-" {
                    has_minus = true;
                } else if has_minus
                    && (child.kind() == "integer_literal" || child.kind() == "float_literal")
                {
                    literal_child = Some(child);
                }
            }
            if let Some(_lit) = literal_child {
                let snippet = node_text(node, source).trim();
                acc.push(SourceConstant {
                    kind: ConstantKind::NegativeLiteral,
                    snippet,
                    line: line_num,
                    column: col,
                    citation: get_citation(),
                    is_test_origin: cfg_test,
                    function_return_type: fn_return_type,
                    is_in_binary_scaling: scaling_context_var.is_some(),
                    context_var: scaling_context_var.clone(),
                    geometric_sink: current_sink.clone(),
                    is_in_data_table: in_data_table,
                });
                return;
            }
        }
        "match_pattern" | "range_pattern" => {
            if kind == "range_pattern" {
                let snippet = node_text(node, source).trim();
                acc.push(SourceConstant {
                    kind: ConstantKind::MatchPattern,
                    snippet,
                    line: line_num,
                    column: col,
                    citation: get_citation(),
                    is_test_origin: cfg_test,
                    function_return_type: fn_return_type,
                    is_in_binary_scaling: false,
                    context_var: None,
                    geometric_sink: current_sink.clone(),
                    is_in_data_table: in_data_table,
                });
                return;
            }
        }
        "integer_literal" | "float_literal" => {
            let snippet = node_text(node, source).trim();
            let c_kind = if in_pattern {
                ConstantKind::MatchPattern
            } else {
                ConstantKind::FunctionNumberLiteral
            };
            acc.push(SourceConstant {
                kind: c_kind,
                snippet,
                line: line_num,
                column: col,
                citation: get_citation(),
                is_test_origin: cfg_test,
                function_return_type: fn_return_type,
                is_in_binary_scaling: scaling_context_var.is_some(),
                context_var: scaling_context_var.clone(),
                geometric_sink: current_sink.clone(),
                is_in_data_table: in_data_table,
            });
            return;
        }
        "string_literal" | "raw_string_literal" => {
            let snippet = node_text(node, source).trim();
            let is_fmt = is_numeric_format_specifier(snippet);
            let c_kind = if is_fmt {
                ConstantKind::FormatString
            } else if in_pattern {
                ConstantKind::MatchPattern
            } else {
                ConstantKind::FunctionStringLiteral
            };
            acc.push(SourceConstant {
                kind: c_kind,
                snippet,
                line: line_num,
                column: col,
                citation: get_citation(),
                is_test_origin: cfg_test,
                function_return_type: fn_return_type,
                is_in_binary_scaling: false,
                context_var: None,
                geometric_sink: current_sink.clone(),
                is_in_data_table: in_data_table,
            });
            return;
        }
        "match_expression" => {
            let is_table = is_data_table_match(node, source);
            for_each_child_in_test_scope(node, source, cfg_test, |child, child_cfg| {
                collect_constants(
                    child,
                    source,
                    citations,
                    child_cfg,
                    fn_return_type,
                    in_fn_body,
                    in_pattern,
                    in_data_table || is_table,
                    current_sink.clone(),
                    scaling_context_var.clone(),
                    acc,
                );
            });
            return;
        }
        "function_item" => {
            let mut ret_type = None;
            if let Some(ret_node) = node.child_by_field_name("return_type") {
                ret_type = Some(node_text(ret_node, source).trim());
            }

            for_each_child_in_test_scope(node, source, cfg_test, |child, child_cfg| {
                let is_body = child.kind() == "block";
                collect_constants(
                    child,
                    source,
                    citations,
                    child_cfg,
                    ret_type,
                    in_fn_body || is_body,
                    in_pattern,
                    in_data_table,
                    current_sink.clone(),
                    scaling_context_var.clone(),
                    acc,
                );
            });
            return;
        }
        "closure_expression" => {
            for_each_child_in_test_scope(node, source, cfg_test, |child, child_cfg| {
                collect_constants(
                    child,
                    source,
                    citations,
                    child_cfg,
                    fn_return_type,
                    true,
                    in_pattern,
                    in_data_table,
                    current_sink.clone(),
                    scaling_context_var.clone(),
                    acc,
                );
            });
            return;
        }
        "match_arm" => {
            let mut cursor = node.walk();
            let mut seen_arrow = false;
            for child in node.children(&mut cursor) {
                if child.kind() == "=>" {
                    seen_arrow = true;
                } else if !seen_arrow && child.kind() != "match" && child.kind() != "if" {
                    collect_constants(
                        child,
                        source,
                        citations,
                        cfg_test,
                        fn_return_type,
                        in_fn_body,
                        true,
                        in_data_table,
                        current_sink.clone(),
                        scaling_context_var.clone(),
                        acc,
                    );
                } else if seen_arrow {
                    collect_constants(
                        child,
                        source,
                        citations,
                        cfg_test,
                        fn_return_type,
                        true,
                        false,
                        in_data_table,
                        current_sink.clone(),
                        scaling_context_var.clone(),
                        acc,
                    );
                }
            }
            return;
        }
        _ => {}
    }

    for_each_child_in_test_scope(node, source, cfg_test, |child, child_cfg| {
        collect_constants(
            child,
            source,
            citations,
            child_cfg,
            fn_return_type,
            in_fn_body,
            in_pattern,
            in_data_table,
            current_sink.clone(),
            scaling_context_var.clone(),
            acc,
        );
    });
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::prompt_reader::PromptReader;
    use crate::contracts::prompt_snapshot_reader::PromptSnapshotReader;
    use crate::entities::parsed_file::PublicInterface;
    use std::path::{Path, PathBuf};

    struct NullPromptReader;
    impl PromptReader for NullPromptReader {
        fn read_hash(&self, _: &str) -> Option<String> {
            None
        }
        fn exists(&self, _: &str) -> bool {
            false
        }
    }

    struct NullSnapshotReader;
    impl PromptSnapshotReader for NullSnapshotReader {
        fn read_snapshot(&self, _: &str) -> Option<PublicInterface<'static>> {
            None
        }
        fn serialize_snapshot(&self, _: &PublicInterface<'_>) -> String {
            String::new()
        }
    }

    fn make_parser() -> RustParser<NullPromptReader, NullSnapshotReader> {
        RustParser::new(
            NullPromptReader,
            NullSnapshotReader,
            CrystallineConfig::default(),
            CrateRegistry::empty(),
        )
    }

    fn make_semantic_parser(
        semantic: crate::infra::config::SemanticContractsConfig,
    ) -> RustParser<NullPromptReader, NullSnapshotReader> {
        let mut config = CrystallineConfig::default();
        config.semantic = semantic;
        RustParser::new(
            NullPromptReader,
            NullSnapshotReader,
            config,
            CrateRegistry::empty(),
        )
    }

    /// Parser com um registro de membros — exercita a classificação cross-crate.
    fn make_parser_with_registry(
        registry: CrateRegistry,
    ) -> RustParser<NullPromptReader, NullSnapshotReader> {
        RustParser::new(
            NullPromptReader,
            NullSnapshotReader,
            CrystallineConfig::default(),
            registry,
        )
    }

    /// SourceFile num path/camada específicos (para casar com o `owner` do registro).
    fn source_file_at(path: &str, layer: Layer, content: &str) -> SourceFile {
        SourceFile {
            path: PathBuf::from(path),
            content: content.to_string(),
            language: Language::Rust,
            layer,
            has_adjacent_test: false,
        }
    }

    /// Camada classificada de um import isolado, dado um registro e owner.
    fn classify_layer(
        path: &str,
        config: &CrystallineConfig,
        registry: &CrateRegistry,
        owner: Option<&MemberCrate>,
    ) -> Option<Layer> {
        match classify_import(path, config, registry, owner) {
            ImportClass::Resolved(l) => Some(l),
            ImportClass::LocalItem => None,
        }
    }

    fn source_file(content: &str) -> SourceFile {
        SourceFile {
            path: PathBuf::from("01_core/foo.rs"),
            content: content.to_string(),
            language: Language::Rust,
            layer: Layer::L1,
            has_adjacent_test: false,
        }
    }

    #[test]
    fn parses_valid_rust_source() {
        let parser = make_parser();
        let file = source_file("fn main() {}");
        assert!(parser.parse(&file).is_ok());
    }

    #[test]
    fn v23_extracts_context_erasure_but_not_legitimate_resolution() {
        use crate::infra::config::{
            ContextSemanticContract, SemanticContractsConfig, SemanticResolverConfig,
        };
        let semantic = SemanticContractsConfig {
            context: vec![ContextSemanticContract {
                id: "radius".into(),
                language: "rust".into(),
                scopes: vec!["01_core/foo.rs::draw".into()],
                sources: vec!["contextual_radius".into(), "absolute_radius".into()],
                resolvers: vec![SemanticResolverConfig {
                    symbol: "resolve_pt".into(),
                    context_arg: 0,
                }],
                erasing_projections: vec!["abs".into()],
                sinks: vec!["rounded_rect".into()],
                absolute_sources: vec!["absolute_radius".into()],
            }],
            ..Default::default()
        };
        let parser = make_semantic_parser(semantic);
        let file = source_file(
            "fn draw(contextual_radius: Length, absolute_radius: Length, style: Style) {\n\
            let a = contextual_radius.resolve_pt(0.0); rounded_rect(a);\n\
            let b = contextual_radius.abs.0; rounded_rect(b);\n\
            let c = absolute_radius.resolve_pt(0.0); rounded_rect(c);\n\
            let d = contextual_radius.resolve_pt(style.size.val()); rounded_rect(d);\n\
            let zero = 0.0; consume(zero);\n\
        }",
        );
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.semantic_observations.len(), 2);
        assert!(parsed
            .semantic_observations
            .iter()
            .any(|o| o.kind == SemanticObservationKind::ContextNeutralArgument));
        assert!(parsed
            .semantic_observations
            .iter()
            .any(|o| o.kind == SemanticObservationKind::ContextErasingProjection));
    }

    #[test]
    fn v24_distinguishes_lost_preserved_and_authorized_fields() {
        use crate::infra::config::{ProjectionSemanticContract, SemanticContractsConfig};
        let semantic = SemanticContractsConfig {
            projection: vec![
                ProjectionSemanticContract {
                    id: "font-id".into(),
                    language: "rust".into(),
                    scope: "01_core/foo.rs::lost".into(),
                    source: "style.variations".into(),
                    destination: "return.2".into(),
                    neutral_forms: vec!["default".into()],
                    normalization: "preserve".into(),
                },
                ProjectionSemanticContract {
                    id: "font-id-preserved".into(),
                    language: "rust".into(),
                    scope: "01_core/foo.rs::kept".into(),
                    source: "style.variations".into(),
                    destination: "return.2".into(),
                    neutral_forms: vec!["default".into()],
                    normalization: "preserve".into(),
                },
                ProjectionSemanticContract {
                    id: "normalized".into(),
                    language: "rust".into(),
                    scope: "01_core/foo.rs::normalized".into(),
                    source: "style.variations".into(),
                    destination: "return.2".into(),
                    neutral_forms: vec!["default".into()],
                    normalization: "drop-to-default".into(),
                },
            ],
            ..Default::default()
        };
        let parser = make_semantic_parser(semantic);
        let file = source_file("fn lost(style: &Style) -> Option<(A,B,V)> { Some((a(), b(), V::default())) }\n\
            fn kept(style: &Style) -> Option<(A,B,V)> { Some((a(), b(), style.variations.clone().unwrap_or_default())) }\n\
            fn normalized(style: &Style) -> Option<(A,B,V)> { Some((a(), b(), V::default())) }");
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.semantic_observations.len(), 1);
        assert_eq!(parsed.semantic_observations[0].contract_id, "font-id");
    }

    #[test]
    fn v25_extracts_duplicate_proxy_and_canonicalizer_reentry() {
        use crate::infra::config::{DecisionSemanticContract, SemanticContractsConfig};
        let semantic = SemanticContractsConfig {
            decision: vec![DecisionSemanticContract {
                id: "math".into(),
                language: "rust".into(),
                owner: "01_core/foo.rs::owner".into(),
                consumers: vec!["01_core/foo.rs::consumer".into()],
                explicit_sources: vec!["style.math".into()],
                proxies: vec!["contains(\"math\")".into()],
                duplicate_owners: vec!["01_core/foo.rs::duplicate".into()],
                canonicalizers: vec!["map_glyph".into()],
                resolved_after: vec!["01_core/foo.rs::downstream".into()],
            }],
            ..Default::default()
        };
        let parser = make_semantic_parser(semantic);
        let file = source_file("fn owner(style: &Style) -> bool { style.math }\n\
            fn duplicate(text: &str) -> bool { text.len() == 1 }\n\
            fn consumer(style: &Style, name: &str) -> bool { style.math || name.contains(\"math\") }\n\
            fn downstream(g: Glyph) -> Glyph { map_glyph(g) }\n\
            fn legitimate(style: &Style) -> bool { owner(style) }");
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.semantic_observations.len(), 3);
    }

    #[test]
    fn returns_empty_source_error() {
        let parser = make_parser();
        let file = source_file("");
        assert!(matches!(
            parser.parse(&file),
            Err(ParseError::EmptySource { .. })
        ));
    }

    #[test]
    fn returns_unsupported_language_error() {
        let parser = make_parser();
        let mut file = source_file("fn main() {}");
        file.language = Language::TypeScript;
        assert!(matches!(
            parser.parse(&file),
            Err(ParseError::UnsupportedLanguage { .. })
        ));
    }

    #[test]
    fn extracts_prompt_header() {
        let parser = make_parser();
        let file = source_file(
            "//! Crystalline Lineage\n\
//! @prompt 00_nucleo/prompts/linter-core.md\n\
//! @prompt-hash c0d309ae\n\
//! @layer L1\n\
//! @updated 2026-06-09
fn main() {}",
        );
        let parsed = parser.parse(&file).unwrap();
        let header = parsed.prompt_header.unwrap();
        assert_eq!(header.prompt_path, "00_nucleo/prompts/linter-core.md");
        assert_eq!(header.prompt_hash, Some("c0d309ae"));
        assert_eq!(header.layer, Layer::L1);
    }

    // ── prompt_refs (V15) ───────────────────────────────────────────────────

    #[test]
    fn prompt_refs_single_for_normal_header() {
        let parser = make_parser();
        let file = source_file(
            "//! @prompt 00_nucleo/prompts/linter-core.md\n\
//! @layer L1
fn main() {}",
        );
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.prompt_refs, vec!["00_nucleo/prompts/linter-core.md"]);
    }

    #[test]
    fn prompt_refs_empty_without_header() {
        let parser = make_parser();
        let file = source_file("fn main() {}");
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.prompt_refs.is_empty());
    }

    #[test]
    fn prompt_refs_collects_all_prompt_lines_in_order() {
        let parser = make_parser();
        let file = source_file(
            "//! @prompt 00_nucleo/prompts/a.md\n\
//! @prompt-hash c0d309ae\n\
//! @prompt 00_nucleo/prompts/b.md\n\
//! @layer L1
fn main() {}",
        );
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(
            parsed.prompt_refs,
            vec!["00_nucleo/prompts/a.md", "00_nucleo/prompts/b.md"]
        );
        // Último valor preservado para V1/V5 — V15 bloqueia o caso.
        assert_eq!(
            parsed.prompt_header.unwrap().prompt_path,
            "00_nucleo/prompts/b.md"
        );
    }

    #[test]
    fn prompt_refs_ignores_prompt_outside_doc_header() {
        let parser = make_parser();
        let file = source_file(
            "//! @prompt 00_nucleo/prompts/a.md\n\
//! @layer L1
fn main() {}
// @prompt 00_nucleo/prompts/nao-conta.md",
        );
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.prompt_refs, vec!["00_nucleo/prompts/a.md"]);
    }

    #[test]
    fn detects_cfg_test_as_coverage() {
        let parser = make_parser();
        let file = source_file(
            "fn foo() {}\n\
             #[cfg(test)]\n\
             mod tests { #[test] fn t() { assert!(true); } }",
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.has_test_coverage);
    }

    #[test]
    fn trait_only_file_is_declaration_only() {
        let parser = make_parser();
        let file = source_file("pub trait Foo { fn bar(&self); }");
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.has_test_coverage); // exempt via is_declaration_only
    }

    #[test]
    fn resolves_crate_import_layer() {
        let config = CrystallineConfig::default();
        let reg = CrateRegistry::empty();
        assert_eq!(
            classify_layer("crate::entities::layer::Layer", &config, &reg, None),
            Some(Layer::L1)
        );
        assert_eq!(
            classify_layer("crate::shell::cli::Cli", &config, &reg, None),
            Some(Layer::L2)
        );
        assert_eq!(
            classify_layer("crate::infra::walker::FileWalker", &config, &reg, None),
            Some(Layer::L3)
        );
    }

    #[test]
    fn external_crate_resolves_to_unknown() {
        // Sem owner (registro vazio) → comportamento legado: tudo não-crate vira Unknown.
        let config = CrystallineConfig::default();
        let reg = CrateRegistry::empty();
        assert_eq!(
            classify_layer("reqwest::Client", &config, &reg, None),
            Some(Layer::Unknown)
        );
        assert_eq!(
            classify_layer("std::fs::read", &config, &reg, None),
            Some(Layer::Unknown)
        );
    }

    // ── Classificação ciente de dependências (0052) ──────────────────────────

    fn member(name: &str, dir: &str, layer: Layer, deps: &[&str]) -> MemberCrate {
        MemberCrate {
            name: name.to_string(),
            dir: PathBuf::from(dir),
            layer,
            deps: deps.iter().map(|s| s.to_string()).collect(),
            renames: Default::default(),
        }
    }

    /// Workspace de teste: L1 (core, shared), L2 (shell), L3 (infra), L4 (wiring).
    fn case_registry() -> CrateRegistry {
        CrateRegistry::from_members(vec![
            member("proj_core", "/proj/core", Layer::L1, &["serde"]),
            member("proj_shared", "/proj/shared", Layer::L1, &[]),
            member("proj_shell", "/proj/shell", Layer::L2, &[]),
            member("proj_infra", "/proj/infra", Layer::L3, &["proj_wiring"]),
            member("proj_wiring", "/proj/wiring", Layer::L4, &[]),
        ])
        .unwrap()
    }

    #[test]
    fn case1_first_party_cross_crate_forbidden_resolves_target_layer() {
        // L3 importando membro L4 → L4 (V3 dispararia: L3→L4). Hoje era o buraco.
        let config = CrystallineConfig::default();
        let reg = case_registry();
        let owner = reg.owner_of(Path::new("/proj/infra/src/x.rs"));
        assert_eq!(
            classify_layer("proj_wiring::Algo", &config, &reg, owner),
            Some(Layer::L4)
        );
    }

    #[test]
    fn case2_first_party_cross_crate_allowed_resolves_l1() {
        // L1 importando outro membro L1 → L1 (V3 calado L1→L1, e V14 calado: não-Unknown).
        let config = CrystallineConfig::default();
        let reg = case_registry();
        let owner = reg.owner_of(Path::new("/proj/core/src/x.rs"));
        assert_eq!(
            classify_layer("proj_shared::X", &config, &reg, owner),
            Some(Layer::L1)
        );
    }

    #[test]
    fn case3_declared_external_resolves_unknown() {
        // L1 com dep externa declarada → Unknown (V14 dispara).
        let config = CrystallineConfig::default();
        let reg = case_registry();
        let owner = reg.owner_of(Path::new("/proj/core/src/x.rs"));
        assert_eq!(
            classify_layer("serde::Serialize", &config, &reg, owner),
            Some(Layer::Unknown)
        );
    }

    #[test]
    fn case4_local_type_is_not_classified() {
        // L1 com `use EnumLocal::*` (não é dependência) → não classificar (V14 calado).
        let config = CrystallineConfig::default();
        let reg = case_registry();
        let owner = reg.owner_of(Path::new("/proj/core/src/x.rs"));
        assert_eq!(classify_layer("EnumLocal::*", &config, &reg, owner), None);
    }

    #[test]
    fn case5_std_preserved_as_unknown() {
        let config = CrystallineConfig::default();
        let reg = case_registry();
        let owner = reg.owner_of(Path::new("/proj/core/src/x.rs"));
        assert_eq!(
            classify_layer("std::collections::HashMap", &config, &reg, owner),
            Some(Layer::Unknown)
        );
    }

    #[test]
    fn case6_intra_crate_preserved_via_module_layer() {
        // `use crate::shell::X` num L1 → L2 (o controle do 0051; V3 dispara).
        let config = CrystallineConfig::default();
        let reg = case_registry();
        let owner = reg.owner_of(Path::new("/proj/core/src/x.rs"));
        assert_eq!(
            classify_layer("crate::shell::cli::X", &config, &reg, owner),
            Some(Layer::L2)
        );
    }

    #[test]
    fn self_import_by_crate_name_is_intra_crate() {
        // `proj_core::shell::X` do próprio proj_core → module_layer(shell)=L2, não a camada do membro (L1).
        let config = CrystallineConfig::default();
        let reg = case_registry();
        let owner = reg.owner_of(Path::new("/proj/core/src/x.rs"));
        assert_eq!(
            classify_layer("proj_core::shell::X", &config, &reg, owner),
            Some(Layer::L2)
        );
    }

    #[test]
    fn self_import_unmapped_submodule_falls_back_to_owner_layer() {
        // `use lente_filtro::filtrar_stdlib` — self-import de função re-exportada na raiz,
        // submódulo NÃO mapeado em [module_layers]. Deve cair na camada do próprio crate
        // (L1), não Unknown — senão V14 dispara falso (o resíduo do laudo 0053).
        let config = CrystallineConfig::default();
        let reg = case_registry();
        let owner = reg.owner_of(Path::new("/proj/core/src/x.rs"));
        assert_eq!(
            classify_layer("proj_core::filtrar_stdlib", &config, &reg, owner),
            Some(Layer::L1)
        );
    }

    #[test]
    fn empty_registry_does_not_skip_local_looking_imports() {
        // Não-regressão: registro vazio (owner None) → `use EnumLocal::*` vira Unknown (legado), não Skip.
        let config = CrystallineConfig::default();
        let reg = CrateRegistry::empty();
        assert_eq!(
            classify_layer("EnumLocal::*", &config, &reg, None),
            Some(Layer::Unknown)
        );
    }

    // ── V9 cross-crate (resolve_subdir ciente) ───────────────────────────────

    #[test]
    fn v9_cross_crate_subdir_resolved_for_l1_member_submodule() {
        // L2 importando subdir interno de um membro L1 → subdir "internal" (V9 dispara).
        let config = CrystallineConfig::default();
        assert_eq!(
            resolve_subdir("proj_core::internal::X", &config, &Layer::L1),
            Some("internal")
        );
    }

    #[test]
    fn v9_cross_crate_two_segment_import_has_no_subdir() {
        // `proj_core::Thing` (2 segmentos) usa a API da raiz, não uma porta → sem subdir.
        let config = CrystallineConfig::default();
        assert_eq!(
            resolve_subdir("proj_core::Thing", &config, &Layer::L1),
            None
        );
    }

    #[test]
    fn v9_intra_crate_subdir_preserved() {
        // Legado: crate::entities::X (entities→L1) → Some("entities"); crate::shell::X → None.
        let config = CrystallineConfig::default();
        assert_eq!(
            resolve_subdir("crate::entities::Layer", &config, &Layer::L1),
            Some("entities")
        );
        assert_eq!(
            resolve_subdir("crate::shell::cli::X", &config, &Layer::L2),
            None
        );
    }

    // ── End-to-end via parse() ───────────────────────────────────────────────

    #[test]
    fn parse_emits_cross_crate_member_import_with_target_layer() {
        let reg = case_registry();
        let parser = make_parser_with_registry(reg);
        let file = source_file_at(
            "/proj/infra/src/x.rs",
            Layer::L3,
            "use proj_wiring::Algo;\nfn f() {}",
        );
        let parsed = parser.parse(&file).unwrap();
        let imp = parsed
            .imports
            .iter()
            .find(|i| i.path.starts_with("proj_wiring"));
        assert_eq!(imp.map(|i| i.target_layer.clone()), Some(Layer::L4));
    }

    #[test]
    fn parse_skips_local_type_import() {
        let reg = case_registry();
        let parser = make_parser_with_registry(reg);
        let file = source_file_at(
            "/proj/core/src/x.rs",
            Layer::L1,
            "use EnumLocal::*;\nfn f() {}",
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed
            .imports
            .iter()
            .all(|i| !i.path.starts_with("EnumLocal")));
    }

    // ── path-ref cross-crate fora do `use` (cego #2, 0060) ───────────────────
    //
    // `collect_imports` deve enxergar referências cross-crate por caminho
    // qualificado fora do `use`/`extern crate` — em expressão, tipo e atributo —
    // sem inventar aresta para caminho local (`crate::`/`self::`/`super::`) ou `std`.

    /// Helper: o Import cujo 1º segmento (normalizado) é `seg`, se houver.
    fn import_to<'a>(parsed: &'a ParsedFile<'a>, seg: &str) -> Option<&'a Import<'a>> {
        parsed.imports.iter().find(|i| first_segment(i.path) == seg)
    }

    #[test]
    fn pathref_expression_collected_as_cross_crate() {
        // A. expressão: L2 chama `proj_wiring::go()` (L4) sem `use` → aresta L2→L4.
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "fn f() { proj_wiring::go(); }",
        );
        let parsed = parser.parse(&file).unwrap();
        let imp = import_to(&parsed, "proj_wiring").expect("path-ref de expressão coletado");
        assert_eq!(imp.target_layer, Layer::L4);
    }

    #[test]
    fn pathref_type_position_collected_as_cross_crate() {
        // A. tipo: L2 com tipo de retorno `proj_wiring::T` (L4) sem `use` → aresta L2→L4.
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "fn f() -> proj_wiring::T { todo!() }",
        );
        let parsed = parser.parse(&file).unwrap();
        let imp = import_to(&parsed, "proj_wiring").expect("path-ref de tipo coletado");
        assert_eq!(imp.target_layer, Layer::L4);
    }

    #[test]
    fn pathref_attribute_token_tree_collected_as_cross_crate() {
        // B. atributo: L2 com `#[arg(default_value_t = proj_wiring::N)]` (L4) → aresta L2→L4.
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "struct S {\n  #[arg(default_value_t = proj_wiring::N)]\n  field: u32,\n}",
        );
        let parsed = parser.parse(&file).unwrap();
        let imp = import_to(&parsed, "proj_wiring").expect("path-ref de atributo coletado");
        assert_eq!(imp.target_layer, Layer::L4);
    }

    #[test]
    fn pathref_local_crate_path_creates_no_import() {
        // Negativa: `crate::interno::Foo` inline é local — não pode virar aresta.
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "fn f() { let _x = crate::interno::Foo; }",
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.imports.iter().all(|i| !i.path.contains("interno")));
    }

    #[test]
    fn pathref_super_path_creates_no_import() {
        // Negativa: `super::X` inline é local — não pode virar aresta.
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "fn f() { let _x = super::Algo; }",
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed
            .imports
            .iter()
            .all(|i| first_segment(i.path) != "super"));
    }

    #[test]
    fn pathref_std_path_creates_no_import() {
        // Negativa: `std::cmp::max(...)` inline é stdlib — não pode virar aresta.
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "fn f() { let _x = std::cmp::max(1, 2); }",
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed
            .imports
            .iter()
            .all(|i| first_segment(i.path) != "std"));
    }

    #[test]
    fn pathref_deduped_against_use_of_same_crate() {
        // C. dedup: `use proj_wiring::A;` + `proj_wiring::go()` inline → UMA aresta.
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "use proj_wiring::A;\nfn f() { proj_wiring::go(); }",
        );
        let parsed = parser.parse(&file).unwrap();
        let n = parsed
            .imports
            .iter()
            .filter(|i| first_segment(i.path) == "proj_wiring")
            .count();
        assert_eq!(n, 1, "uso + path-ref do mesmo crate = uma aresta");
    }

    #[test]
    fn pathref_multiple_inline_refs_dedup_to_one_edge() {
        // C. dedup: dois path-refs inline ao mesmo crate → UMA aresta.
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "fn f() { proj_wiring::a(); proj_wiring::b(); }",
        );
        let parsed = parser.parse(&file).unwrap();
        let n = parsed
            .imports
            .iter()
            .filter(|i| first_segment(i.path) == "proj_wiring")
            .count();
        assert_eq!(n, 1, "dois path-refs ao mesmo crate = uma aresta");
    }

    // ── is_test_origin: marca de origem test vs produção (0061) ──────────────
    //
    // Imports nascidos sob `#[cfg(test)]` (removidos do build de produção) são
    // marcados — a gravidade os pula por padrão. Cobrir as DUAS vias de coleta:
    // `use` (collect_imports) e path-ref (collect_path_refs).

    #[test]
    fn cfg_test_use_import_marked_test_origin() {
        // `use` dentro de `#[cfg(test)] mod tests` → is_test_origin = true.
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "#[cfg(test)]\nmod tests {\n    use proj_wiring::A;\n    fn t() { let _ = A; }\n}",
        );
        let parsed = parser.parse(&file).unwrap();
        let imp = import_to(&parsed, "proj_wiring").expect("import de teste coletado");
        assert!(imp.is_test_origin, "use sob #[cfg(test)] é test-origin");
    }

    #[test]
    fn cfg_test_pathref_marked_test_origin() {
        // path-ref (0060) dentro de `#[cfg(test)] fn` → is_test_origin = true.
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "#[cfg(test)]\nmod tests {\n    fn t() { proj_wiring::go(); }\n}",
        );
        let parsed = parser.parse(&file).unwrap();
        let imp = import_to(&parsed, "proj_wiring").expect("path-ref de teste coletado");
        assert!(
            imp.is_test_origin,
            "path-ref sob #[cfg(test)] é test-origin"
        );
    }

    #[test]
    fn cfg_test_inner_attribute_marks_test_origin() {
        // `#![cfg(test)]` interno marca toda a subárvore do módulo que o contém.
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "mod tests {\n    #![cfg(test)]\n    use proj_wiring::A;\n    fn t() { let _ = A; }\n}",
        );
        let parsed = parser.parse(&file).unwrap();
        let imp = import_to(&parsed, "proj_wiring").expect("import coletado");
        assert!(
            imp.is_test_origin,
            "use sob #![cfg(test)] interno é test-origin"
        );
    }

    #[test]
    fn production_import_not_test_origin() {
        // `use` de nível superior (produção) → is_test_origin = false.
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "use proj_wiring::A;\nfn f() { let _ = A; }",
        );
        let parsed = parser.parse(&file).unwrap();
        let imp = import_to(&parsed, "proj_wiring").expect("import de produção coletado");
        assert!(!imp.is_test_origin, "use de produção não é test-origin");
    }

    #[test]
    fn production_use_not_marked_when_cfg_test_sibling_mod_present() {
        // O `#[cfg(test)]` é IRMÃO do `mod tests` na grammar — não pode contaminar o
        // `use` de produção que o precede no mesmo nível. (Regressão do v14_fail.)
        let parser = make_parser_with_registry(case_registry());
        let file = source_file_at(
            "/proj/shell/src/x.rs",
            Layer::L2,
            "use proj_wiring::A;\nfn f() { let _ = A; }\n\
             #[cfg(test)]\nmod tests {\n    use proj_shared::B;\n    fn t() { let _ = B; }\n}",
        );
        let parsed = parser.parse(&file).unwrap();
        let prod = import_to(&parsed, "proj_wiring").expect("import de produção coletado");
        assert!(!prod.is_test_origin, "use de produção NÃO é test-origin");
        let test = import_to(&parsed, "proj_shared").expect("import de teste coletado");
        assert!(
            test.is_test_origin,
            "use sob #[cfg(test)] mod É test-origin"
        );
    }

    #[test]
    fn adjacent_test_sets_coverage() {
        let parser = make_parser();
        let mut file = source_file("fn foo() -> u32 { 42 }");
        file.has_adjacent_test = true;
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.has_test_coverage);
    }

    // ── declared_traits ───────────────────────────────────────────────────

    #[test]
    fn declared_traits_extracted_for_l1_contracts() {
        let parser = make_parser();
        let mut file = source_file(
            "pub trait FileProvider { fn files(&self); }\n\
             pub trait LanguageParser { fn parse(&self); }\n\
             trait InternalHelper { fn helper(&self); }",
        );
        file.path = PathBuf::from("01_core/contracts/file_provider.rs");
        file.layer = Layer::L1;
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.declared_traits.contains(&"FileProvider"));
        assert!(parsed.declared_traits.contains(&"LanguageParser"));
        assert!(!parsed.declared_traits.contains(&"InternalHelper"));
    }

    #[test]
    fn declared_traits_empty_for_l1_non_contracts_subdir() {
        let parser = make_parser();
        let mut file = source_file("pub trait HasImports<'a> { fn imports(&self); }");
        file.path = PathBuf::from("01_core/rules/forbidden_import.rs");
        file.layer = Layer::L1;
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.declared_traits.is_empty());
    }

    #[test]
    fn declared_traits_empty_for_l2() {
        let parser = make_parser();
        let mut file = source_file("pub trait SomeTrait { fn do_it(&self); }");
        file.path = PathBuf::from("02_shell/contracts/foo.rs");
        file.layer = Layer::L2;
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.declared_traits.is_empty());
    }

    // ── implemented_traits ────────────────────────────────────────────────

    #[test]
    fn implemented_traits_extracted_for_l3() {
        let parser = make_parser();
        let mut file = source_file(
            "pub struct FsWalker;\n\
             impl FileProvider for FsWalker { fn files(&self) {} }\n\
             impl LanguageParser for FsWalker { fn parse(&self) {} }\n\
             impl FsWalker { fn new() -> Self { FsWalker } }",
        );
        file.path = PathBuf::from("03_infra/walker.rs");
        file.layer = Layer::L3;
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.implemented_traits.contains(&"FileProvider"));
        assert!(parsed.implemented_traits.contains(&"LanguageParser"));
        assert!(!parsed.implemented_traits.contains(&"FsWalker"));
    }

    #[test]
    fn implemented_traits_extracted_for_l2() {
        let parser = make_parser();
        let mut file = source_file(
            "pub struct Cli;\n\
             impl PromptReader for Cli { fn read(&self) {} }",
        );
        file.path = PathBuf::from("02_shell/cli.rs");
        file.layer = Layer::L2;
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.implemented_traits.contains(&"PromptReader"));
    }

    #[test]
    fn implemented_traits_empty_for_l1() {
        let parser = make_parser();
        let mut file =
            source_file("impl HasImports for ParsedFile { fn layer(&self) -> u8 { 0 } }");
        file.path = PathBuf::from("01_core/entities/parsed_file.rs");
        file.layer = Layer::L1;
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.implemented_traits.is_empty());
    }

    #[test]
    fn implemented_traits_strips_path_prefix() {
        let parser = make_parser();
        let mut file = source_file(
            "pub struct R;\n\
             impl crate::contracts::FileProvider for R { fn files(&self) {} }",
        );
        file.path = PathBuf::from("03_infra/reader.rs");
        file.layer = Layer::L3;
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.implemented_traits.contains(&"FileProvider"));
    }

    // ── declarations ──────────────────────────────────────────────────────

    #[test]
    fn declarations_captures_struct_enum_impl_without_trait() {
        let parser = make_parser();
        let mut file = source_file(
            "pub struct OutputRewriter {}\n\
             impl OutputRewriter { pub fn new() -> Self { OutputRewriter {} } }\n\
             impl Formatter for OutputRewriter { fn fmt(&self) {} }\n\
             pub enum OutputMode { Text, Sarif }",
        );
        file.path = PathBuf::from("04_wiring/main.rs");
        file.layer = Layer::L4;
        let parsed = parser.parse(&file).unwrap();
        let kinds: Vec<_> = parsed
            .declarations
            .iter()
            .map(|d| (&d.kind, d.name))
            .collect();
        assert!(kinds.contains(&(&DeclarationKind::Struct, "OutputRewriter")));
        assert!(kinds.contains(&(&DeclarationKind::Impl, "OutputRewriter")));
        assert!(kinds.contains(&(&DeclarationKind::Enum, "OutputMode")));
        // impl with trait must NOT be captured
        assert!(!parsed.declarations.iter().any(|d| d.name == "Formatter"));
    }

    #[test]
    fn declarations_extracted_for_l3_too() {
        let parser = make_parser();
        let mut file = source_file("pub struct FileWalker { root: String }");
        file.path = PathBuf::from("03_infra/walker.rs");
        file.layer = Layer::L3;
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed
            .declarations
            .iter()
            .any(|d| d.kind == DeclarationKind::Struct && d.name == "FileWalker"));
    }

    #[test]
    fn declarations_impl_with_trait_not_captured() {
        let parser = make_parser();
        let mut file = source_file(
            "pub struct Rewriter;\n\
             impl HashRewriter for Rewriter { fn rewrite(&self) {} }",
        );
        file.path = PathBuf::from("04_wiring/main.rs");
        file.layer = Layer::L4;
        let parsed = parser.parse(&file).unwrap();
        // Only Struct captured — the impl Trait for ... must be absent
        assert_eq!(
            parsed
                .declarations
                .iter()
                .filter(|d| d.kind == DeclarationKind::Impl)
                .count(),
            0
        );
        assert_eq!(
            parsed
                .declarations
                .iter()
                .filter(|d| d.kind == DeclarationKind::Struct)
                .count(),
            1
        );
    }

    // ── trait_last_segment unit tests ─────────────────────────────────────

    #[test]
    fn trait_last_segment_strips_prefix() {
        assert_eq!(
            trait_last_segment("crate::contracts::FileProvider"),
            "FileProvider"
        );
    }

    #[test]
    fn trait_last_segment_strips_generics() {
        assert_eq!(trait_last_segment("LanguageParser<'a>"), "LanguageParser");
    }

    #[test]
    fn trait_last_segment_simple_name() {
        assert_eq!(trait_last_segment("PromptReader"), "PromptReader");
    }

    // ── ImportKind mapping — critérios ADR-0009 ────────────────────────────────

    #[test]
    fn use_statement_without_as_or_braces_is_direct() {
        // use crate::shell::api → ImportKind::Direct
        let parser = make_parser();
        let file = source_file("use crate::shell::cli;\nfn foo() {}");
        let parsed = parser.parse(&file).unwrap();
        let imp = parsed.imports.iter().find(|i| i.path.contains("shell"));
        assert!(imp.is_some(), "should have import for crate::shell::cli");
        assert_eq!(imp.unwrap().kind, ImportKind::Direct);
    }

    #[test]
    fn use_star_maps_to_glob() {
        // use crate::entities::* → ImportKind::Glob
        let parser = make_parser();
        let file = source_file("use crate::entities::*;\nfn foo() {}");
        let parsed = parser.parse(&file).unwrap();
        let imp = parsed.imports.iter().find(|i| i.path.contains("entities"));
        assert!(imp.is_some(), "should have import for crate::entities::*");
        assert_eq!(imp.unwrap().kind, ImportKind::Glob);
    }

    #[test]
    fn use_with_as_maps_to_alias() {
        // use std::fs as fs_io → ImportKind::Alias
        let parser = make_parser();
        let file = source_file("use std::fs as fs_io;\nfn foo() {}");
        let parsed = parser.parse(&file).unwrap();
        let imp = parsed.imports.iter().find(|i| i.path.contains("fs"));
        assert!(imp.is_some(), "should have import for std::fs as fs_io");
        assert_eq!(imp.unwrap().kind, ImportKind::Alias);
    }

    #[test]
    fn use_with_braces_maps_to_named() {
        // use crate::entities::{Layer, Language} → ImportKind::Named
        let parser = make_parser();
        let file = source_file("use crate::entities::{Layer, Language};\nfn foo() {}");
        let parsed = parser.parse(&file).unwrap();
        let imp = parsed.imports.iter().find(|i| i.path.contains("entities"));
        assert!(
            imp.is_some(),
            "should have import for crate::entities::{{...}}"
        );
        assert_eq!(imp.unwrap().kind, ImportKind::Named);
    }

    #[test]
    fn extern_crate_maps_to_direct() {
        // extern crate serde → ImportKind::Direct (não variante específica de linguagem)
        let parser = make_parser();
        let file = source_file("extern crate serde;\nfn foo() {}");
        let parsed = parser.parse(&file).unwrap();
        let imp = parsed.imports.iter().find(|i| i.path.contains("serde"));
        assert!(imp.is_some(), "extern crate serde should produce an Import");
        assert_eq!(imp.unwrap().kind, ImportKind::Direct);
    }

    #[test]
    fn mod_declaration_not_in_imports() {
        // mod foo; (sem bloco) → vai para module_decls, não para imports (ADR-0013)
        let parser = make_parser();
        let file = source_file("mod helpers;\nfn bar() {}");
        let parsed = parser.parse(&file).unwrap();
        let in_imports = parsed.imports.iter().any(|i| i.path.contains("helpers"));
        assert!(
            !in_imports,
            "mod declaration must NOT appear in imports after ADR-0013"
        );
    }

    // ── module_decls (ADR-0013) ───────────────────────────────────────────

    #[test]
    fn bare_mod_produces_module_decl() {
        let parser = make_parser();
        let file = source_file("mod helpers;\nfn bar() {}");
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.module_decls.len(), 1);
        let d = &parsed.module_decls[0];
        assert_eq!(d.name, "helpers");
        assert_eq!(d.target_layer, Layer::L1);
        assert_eq!(d.line, 1);
    }

    #[test]
    fn inline_mod_block_not_in_module_decls() {
        let parser = make_parser();
        let file = source_file("mod tests { fn t() {} }\nfn bar() {}");
        let parsed = parser.parse(&file).unwrap();
        assert!(
            parsed.module_decls.is_empty(),
            "inline mod block must NOT appear in module_decls"
        );
    }

    #[test]
    fn pub_mod_produces_module_decl() {
        let parser = make_parser();
        let file = source_file("pub mod rules;\nfn main() {}");
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.module_decls.len(), 1);
        assert_eq!(parsed.module_decls[0].name, "rules");
    }

    #[test]
    fn import_to_l1_has_target_subdir() {
        // use crate::entities::Layer → target_subdir = Some("entities")
        let parser = make_parser();
        let file = source_file("use crate::entities::Layer;\nfn foo() {}");
        let parsed = parser.parse(&file).unwrap();
        let imp = parsed.imports.iter().find(|i| i.target_layer == Layer::L1);
        assert!(imp.is_some(), "crate::entities should resolve to L1");
        assert_eq!(imp.unwrap().target_subdir, Some("entities"));
    }

    // ── Static declarations (V13) ─────────────────────────────────────────────

    #[test]
    fn extracts_static_mut() {
        let parser = make_parser();
        let file = source_file("static mut COUNTER: u32 = 0;\nfn foo() {}");
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.static_declarations.len(), 1);
        let s = &parsed.static_declarations[0];
        assert_eq!(s.name, "COUNTER");
        assert_eq!(s.type_text, "u32");
        assert!(s.is_mut);
        assert_eq!(s.line, 1);
    }

    #[test]
    fn extracts_mutex_static() {
        let parser = make_parser();
        let file = source_file(
            "use std::sync::Mutex;\nstatic CACHE: Mutex<u32> = Mutex::new(0);\nfn foo() {}",
        );
        let parsed = parser.parse(&file).unwrap();
        let s = parsed
            .static_declarations
            .iter()
            .find(|s| s.name == "CACHE");
        assert!(s.is_some());
        let s = s.unwrap();
        assert!(!s.is_mut);
        assert!(s.type_text.contains("Mutex"));
    }

    #[test]
    fn extracts_immutable_str_static() {
        let parser = make_parser();
        let file = source_file("static RULE_ID: &str = \"V13\";\nfn foo() {}");
        let parsed = parser.parse(&file).unwrap();
        let s = parsed
            .static_declarations
            .iter()
            .find(|s| s.name == "RULE_ID");
        assert!(s.is_some());
        let s = s.unwrap();
        assert!(!s.is_mut);
        assert_eq!(s.type_text, "&str");
    }

    #[test]
    fn syntax_error_reports_nonzero_line() {
        // Fonte sintaticamente inválida com erro na linha 2 → line > 0
        // (SyntaxError { line } deve ser ≥ 1 — nunca linha 0)
        let parser = make_parser();
        // Segunda linha é completamente inválida em Rust
        let file = source_file("fn valid() {}\n} } } invalid @ @ @");
        match parser.parse(&file) {
            Err(ParseError::SyntaxError { line, .. }) => {
                assert!(line > 0, "SyntaxError.line should be > 0, got {}", line);
            }
            Ok(_) => {
                // tree-sitter é error-tolerant; se não detectou SyntaxError,
                // o parser pode não implementar esta verificação.
                // Marcar como falha para forçar revisão.
                panic!("expected ParseError::SyntaxError for syntactically invalid source, got Ok");
            }
            Err(other) => panic!("expected SyntaxError, got {:?}", other),
        }
    }

    // ── blanket_impl_traits (ADR-0015) ────────────────────────────────────

    #[test]
    fn blanket_impl_single_bound_detected() {
        // impl<T: World> TrackedWorld for T  — padrão 1 (~60%)
        let parser = make_parser();
        let mut file = source_file(
            "pub struct Wrapper;\nimpl<T: World> TrackedWorld for T { fn method(&self) {} }",
        );
        file.path = PathBuf::from("03_infra/adapter.rs");
        file.layer = Layer::L3;
        let parsed = parser.parse(&file).unwrap();
        assert!(
            parsed.blanket_impl_traits.contains(&"TrackedWorld"),
            "blanket impl<T: B> Trait for T deve ser detectado"
        );
        // impl concreto não deve poluir blanket set
        assert!(!parsed.blanket_impl_traits.contains(&"Wrapper"));
    }

    #[test]
    fn blanket_impl_multi_bound_detected() {
        // impl<T: A + B> Contract for T — padrão 2 (~25%)
        let parser = make_parser();
        let mut file = source_file("impl<T: Alpha + Beta> MyContract for T { fn run(&self) {} }");
        file.path = PathBuf::from("02_shell/adapters.rs");
        file.layer = Layer::L2;
        let parsed = parser.parse(&file).unwrap();
        assert!(
            parsed.blanket_impl_traits.contains(&"MyContract"),
            "blanket impl<T: A + B> Trait for T deve ser detectado"
        );
    }

    #[test]
    fn blanket_impl_where_clause_detected() {
        // impl<T> Contract for T where T: Bound — padrão 3 (~10%)
        let parser = make_parser();
        let mut file =
            source_file("impl<T> WhereContract for T where T: SomeBound { fn exec(&self) {} }");
        file.path = PathBuf::from("03_infra/where_adapter.rs");
        file.layer = Layer::L3;
        let parsed = parser.parse(&file).unwrap();
        assert!(
            parsed.blanket_impl_traits.contains(&"WhereContract"),
            "blanket impl<T> Trait for T where T: B deve ser detectado"
        );
    }

    #[test]
    fn blanket_impl_empty_for_l1() {
        // blanket impls agora são coletados em L1, L2 e L3 (ajuste para TrackedWorld)
        let parser = make_parser();
        let mut file = source_file("impl<T: World> TrackedWorld for T { fn method(&self) {} }");
        file.path = PathBuf::from("01_core/entities/foo.rs");
        file.layer = Layer::L1;
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.blanket_impl_traits.contains(&"TrackedWorld"));
    }

    #[test]
    fn concrete_impl_not_in_blanket_traits() {
        // impl ConcreteType for Adapter — não é blanket
        let parser = make_parser();
        let mut file = source_file(
            "pub struct FsWalker;\nimpl FileProvider for FsWalker { fn files(&self) {} }",
        );
        file.path = PathBuf::from("03_infra/walker.rs");
        file.layer = Layer::L3;
        let parsed = parser.parse(&file).unwrap();
        // FileProvider aparece em implemented_traits, não em blanket_impl_traits
        assert!(parsed.implemented_traits.contains(&"FileProvider"));
        assert!(!parsed.blanket_impl_traits.contains(&"FileProvider"));
    }

    // ── V16–V20 Decision Expressions Unit Tests (Phase B) ──────────────────

    #[test]
    fn extract_decision_exprs_classifies_all_forms() {
        let code = r#"
        fn test_match(x: Option<Unit>, n: u32) {
            match x {
                Some(Unit::Pt) => Unit::Percent,
                Some(Unit::Percent) if n > 0 && n < 10 => panic!("fail"),
                0..=9 => false,
                A | B | C => 1.0,
                Some(Color::Rgb(r, g, b)) => f(r),
                other => f(other),
                _ => vec![],
                _ => {},
                _ => continue,
            }
        }
        "#;
        let parser = make_parser();
        let file = source_file(code);
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.decision_exprs.len(), 1);
        let expr = &parsed.decision_exprs[0];
        assert_eq!(expr.snippet_scrutinee, "x");
        assert_eq!(expr.scrutinee_form, ScrutineeForm::Path);
        assert_eq!(expr.arms.len(), 9);

        // 1. EnumPath
        assert_eq!(expr.arms[0].body_form, BodyForm::EnumPath);
        assert_eq!(expr.arms[0].qualified_prefixes, vec!["Unit"]);

        // 2. ErrorBarrier + Compound Guard
        assert_eq!(expr.arms[1].body_form, BodyForm::ErrorBarrier);
        assert!(expr.arms[1].has_guard);
        assert!(expr.arms[1].guard_is_compound);

        // 3. LiteralNeutral + Range Pattern
        assert_eq!(expr.arms[2].body_form, BodyForm::LiteralNeutral);
        assert!(expr.arms[2].pattern_is_range);

        // 4. LiteralOther + Or Alternatives
        assert_eq!(expr.arms[3].body_form, BodyForm::LiteralOther);
        assert_eq!(expr.arms[3].or_alternatives, 3);

        // 5. Call + Deep Pattern Nesting (depth > 2)
        assert_eq!(expr.arms[4].body_form, BodyForm::Call);
        assert!(expr.arms[4].pattern_depth >= 2);

        // 6. Catchall bound ident used in body
        assert!(expr.arms[5].is_catchall);
        assert!(expr.arms[5].bound_ident_used_in_body);

        // 7. vec![] empty constructor classified as LiteralNeutral (default neutro)
        assert!(expr.arms[6].is_catchall);
        assert!(!expr.arms[6].bound_ident_used_in_body);
        assert_eq!(expr.arms[6].body_form, BodyForm::LiteralNeutral);

        // 8. EmptyBlock
        assert_eq!(expr.arms[7].body_form, BodyForm::EmptyBlock);

        // 9. Continue
        assert_eq!(expr.arms[8].body_form, BodyForm::Continue);
    }

    #[test]
    fn extract_constants_identifies_all_targets_and_citations() {
        let code = r#"
        // ref: spec/pdf.md:42
        const MIN_LEADING: f64 = 12.5;

        // rationale: margem padrão
        fn layout_frame() -> Frame {
            let x = -0.5;
            let fmt = format!("{:.3}", x);
            match x {
                1..=5 => 10,
                _ => 0,
            }
        }
        "#;
        let parser = make_parser();
        let file = source_file(code);
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.constants.len() >= 5);
        let const_item = parsed
            .constants
            .iter()
            .find(|c| c.kind == ConstantKind::ItemDefinition)
            .unwrap();
        assert!(const_item.citation.is_some());
        let neg_item = parsed
            .constants
            .iter()
            .find(|c| c.kind == ConstantKind::NegativeLiteral)
            .unwrap();
        assert_eq!(neg_item.snippet, "-0.5");
        let fmt_item = parsed
            .constants
            .iter()
            .find(|c| c.kind == ConstantKind::FormatString)
            .unwrap();
        assert_eq!(fmt_item.snippet, "\"{:.3}\"");
    }

    #[test]
    fn extract_citations_recognizes_real_world_conventions() {
        let code = r#"
        // P813 — layout — lab/typst-original/src/layout/container.rs:342
        fn calculate_gap() -> f64 {
            let gap = 0.9 * 10.0;
            gap
        }

        // vanilla resolve.rs:1173
        fn calculate_stem() -> f64 {
            0.85 * 12.0
        }
        "#;
        let parser = make_parser();
        let file = source_file(code);
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.constants.len() >= 4);

        // Literal 0.9 (linha 5) associado ao comentário da linha 3 (2 linhas acima)
        let lit_09 = parsed
            .constants
            .iter()
            .find(|c| c.snippet == "0.9")
            .unwrap();
        assert!(
            lit_09.citation.is_some(),
            "P813 container.rs:342 deve ser reconhecido como citação"
        );
        if let Some(ref c) = lit_09.citation {
            if let CitationKind::Ref { path, line } = c.kind {
                assert_eq!(path, "lab/typst-original/src/layout/container.rs");
                assert_eq!(line, 342);
            } else {
                panic!("esperado CitationKind::Ref, obteve {:?}", c.kind);
            }
        }

        // Literal 0.85 associado ao comentário vanilla resolve.rs:1173
        let lit_085 = parsed
            .constants
            .iter()
            .find(|c| c.snippet == "0.85")
            .unwrap();
        assert!(
            lit_085.citation.is_some(),
            "vanilla resolve.rs:1173 deve ser reconhecido como citação"
        );
        if let Some(ref c) = lit_085.citation {
            if let CitationKind::Ref { path, line } = c.kind {
                assert_eq!(path, "resolve.rs");
                assert_eq!(line, 1173);
            } else {
                panic!("esperado CitationKind::Ref, obteve {:?}", c.kind);
            }
        }
    }
}
