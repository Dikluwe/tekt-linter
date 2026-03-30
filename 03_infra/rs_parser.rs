//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/parsers/rust.md
//! @prompt-hash 73539903
//! @layer L3
//! @updated 2026-03-22

use std::borrow::Cow;

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
use crate::infra::config::CrystallineConfig;

// ── RustParser ────────────────────────────────────────────────────────────────

pub struct RustParser<R: PromptReader, S: PromptSnapshotReader> {
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
}

impl<R: PromptReader, S: PromptSnapshotReader> RustParser<R, S> {
    pub fn new(prompt_reader: R, snapshot_reader: S, config: CrystallineConfig) -> Self {
        Self { prompt_reader, snapshot_reader, config }
    }
}

impl<R: PromptReader, S: PromptSnapshotReader> LanguageParser for RustParser<R, S> {
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        if file.content.is_empty() {
            return Err(ParseError::EmptySource { path: file.path.clone() });
        }

        if file.language != Language::Rust {
            return Err(ParseError::UnsupportedLanguage {
                path: file.path.clone(),
                language: file.language.clone(),
            });
        }

        let mut ts_parser = TsParser::new();
        ts_parser
            .set_language(&tree_sitter_rust::language())
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
        let mut prompt_header = extract_header(&file.content);

        let prompt_file_exists = prompt_header
            .as_ref()
            .map(|h| self.prompt_reader.exists(h.prompt_path))
            .unwrap_or(false);

        if let Some(ref mut header) = prompt_header {
            header.current_hash = self.prompt_reader.read_hash(header.prompt_path);
        }

        // ── Imports ───────────────────────────────────────────────────────────
        let imports = extract_imports(root, source, &self.config);

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
        let declared_traits = if file.layer == Layer::L1
            && path_contains_segment(file.path.as_path(), "contracts")
        {
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

        Ok(ParsedFile {
            path: file.path.as_path(),
            layer: file.layer.clone(),
            language: file.language.clone(),
            prompt_header,
            prompt_file_exists,
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
        })
    }
}

// ── Header extraction ─────────────────────────────────────────────────────────

fn extract_header<'a>(source: &'a str) -> Option<PromptHeader<'a>> {
    let mut prompt_path: Option<&'a str> = None;
    let mut prompt_hash: Option<&'a str> = None;
    let mut layer: Option<Layer> = None;
    let mut updated: Option<&'a str> = None;

    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("//!") {
            break;
        }
        let content = trimmed.trim_start_matches("//!").trim();

        if let Some(val) = content.strip_prefix("@prompt-hash ") {
            prompt_hash = Some(val.trim());
        } else if let Some(val) = content.strip_prefix("@prompt ") {
            prompt_path = Some(val.trim());
        } else if let Some(val) = content.strip_prefix("@layer ") {
            layer = Some(parse_layer_tag(val.trim()));
        } else if let Some(val) = content.strip_prefix("@updated ") {
            updated = Some(val.trim());
        }
    }

    prompt_path.map(|path| PromptHeader {
        prompt_path: path,
        prompt_hash,
        current_hash: None, // filled in after header extraction
        layer: layer.unwrap_or(Layer::Unknown),
        updated,
    })
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
) -> Vec<Import<'a>> {
    let mut imports = Vec::new();
    collect_imports(root, source, config, &mut imports);
    imports
}

fn collect_imports<'a>(
    node: Node,
    source: &'a [u8],
    config: &CrystallineConfig,
    imports: &mut Vec<Import<'a>>,
) {
    match node.kind() {
        "use_declaration" => {
            let line = node.start_position().row + 1;
            let path = use_declaration_path(node, source);
            let target_layer = resolve_layer(path, config);
            let target_subdir = resolve_subdir(path, config);
            let kind = if path.ends_with("::*") {
                ImportKind::Glob
            } else if path.contains(" as ") {
                ImportKind::Alias
            } else if path.contains('{') && path.contains('}') {
                ImportKind::Named
            } else {
                ImportKind::Direct
            };
            imports.push(Import { path, line, kind, target_layer, target_subdir });
        }
        "extern_crate_declaration" => {
            let line = node.start_position().row + 1;
            let text = node_text(node, source);
            let path = text
                .trim_start_matches("extern crate ")
                .trim_end_matches(';')
                .trim();
            let target_layer = resolve_layer(path, config);
            let target_subdir = resolve_subdir(path, config);
            imports.push(Import { path, line, kind: ImportKind::Direct, target_layer, target_subdir });
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_imports(child, source, config, imports);
        }
    }
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

/// Resolve the import layer from its path string.
/// Only inspects the second segment of `crate::` paths.
/// Everything else is Layer::Unknown.
fn resolve_layer(path: &str, config: &CrystallineConfig) -> Layer {
    let path = path.trim_start_matches('{').trim();

    if !path.starts_with("crate::") && !path.starts_with("super::") {
        return Layer::Unknown;
    }

    let segments: Vec<&str> = path.splitn(4, "::").collect();
    // segments[0] = "crate" | "super", segments[1] = module name
    if let Some(module_name) = segments.get(1) {
        config.layer_for_module(module_name)
    } else {
        Layer::Unknown
    }
}

/// Resolve o subdiretório de destino de um import para V9.
/// Retorna Some("entities") se import aponta para crate::entities::...
/// Retorna None para crates externas (não começam com "crate::" ou "super::").
/// Inspeciona o segundo segmento do path — o nome do módulo de L1.
fn resolve_subdir<'a>(path: &'a str, config: &CrystallineConfig) -> Option<&'a str> {
    let path = path.trim_start_matches('{').trim();

    if !path.starts_with("crate::") && !path.starts_with("super::") {
        return None; // crate externa — sem subdir
    }

    let segments: Vec<&'a str> = path.splitn(4, "::").collect();
    // segments[0] = "crate" | "super", segments[1] = module name
    let module_name = segments.get(1).copied()?;

    // Só é relevante para imports que apontam para L1
    if config.layer_for_module(module_name) == crate::entities::layer::Layer::L1 {
        Some(module_name)
    } else {
        None
    }
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

    PublicInterface { functions, types, reexports }
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
    let name = node.child_by_field_name("name").map(|n| node_text(n, source))?;

    let params = node
        .child_by_field_name("parameters")
        .map(|p| extract_param_types(p, source))
        .unwrap_or_default();

    let return_type = node
        .child_by_field_name("return_type")
        .map(|rt| node_text(rt, source).trim_start_matches("->").trim());

    Some(FunctionSignature { name, params, return_type })
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

fn extract_type_sig<'a>(
    node: Node,
    source: &'a [u8],
    kind: TypeKind,
) -> Option<TypeSignature<'a>> {
    let name = node.child_by_field_name("name").map(|n| node_text(n, source))?;

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

    Some(TypeSignature { name, kind, members })
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
                            if item.kind() == "function_item"
                                && node_has_child_kind(item, "block")
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
    path.components().any(|c| c.as_os_str().to_str().unwrap_or("") == segment)
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
fn collect_type_param_names<'a>(node: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut names = Vec::new();
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            // type_identifier é o nome simples do parâmetro (ex: T, U)
            // constrained_type_parameter tem field "left" com o nome
            if child.kind() == "type_identifier" {
                names.push(node_text(child, source));
            } else if child.kind() == "constrained_type_parameter" {
                if let Some(left) = child.child_by_field_name("left") {
                    names.push(node_text(left, source));
                }
            } else if child.kind() == "lifetime" {
                // ignorar lifetimes ('a) — não são parâmetros de tipo
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
                    decls.push(StaticDeclaration { name, type_text, is_mut, line });
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
            decls.push(ModuleDecl { name, target_layer: file_layer.clone(), line });
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::prompt_reader::PromptReader;
    use crate::contracts::prompt_snapshot_reader::PromptSnapshotReader;
    use crate::entities::parsed_file::PublicInterface;
    use std::path::PathBuf;

    struct NullPromptReader;
    impl PromptReader for NullPromptReader {
        fn read_hash(&self, _: &str) -> Option<String> { None }
        fn exists(&self, _: &str) -> bool { false }
    }

    struct NullSnapshotReader;
    impl PromptSnapshotReader for NullSnapshotReader {
        fn read_snapshot(&self, _: &str) -> Option<PublicInterface<'static>> { None }
        fn serialize_snapshot(&self, _: &PublicInterface<'_>) -> String { String::new() }
    }

    fn make_parser() -> RustParser<NullPromptReader, NullSnapshotReader> {
        RustParser::new(NullPromptReader, NullSnapshotReader, CrystallineConfig::default())
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
    fn returns_empty_source_error() {
        let parser = make_parser();
        let file = source_file("");
        assert!(matches!(parser.parse(&file), Err(ParseError::EmptySource { .. })));
    }

    #[test]
    fn returns_unsupported_language_error() {
        let parser = make_parser();
        let mut file = source_file("fn main() {}");
        file.language = Language::TypeScript;
        assert!(matches!(parser.parse(&file), Err(ParseError::UnsupportedLanguage { .. })));
    }

    #[test]
    fn extracts_prompt_header() {
        let parser = make_parser();
        let file = source_file(
            "//! Crystalline Lineage\n\
//! @prompt 00_nucleo/prompts/linter-core.md\n\
//! @prompt-hash c0d309ae\n\
//! @layer L1\n\
//! @updated 2026-03-13\n\
fn main() {}",
        );
        let parsed = parser.parse(&file).unwrap();
        let header = parsed.prompt_header.unwrap();
        assert_eq!(header.prompt_path, "00_nucleo/prompts/linter-core.md");
        assert_eq!(header.prompt_hash, Some("c0d309ae"));
        assert_eq!(header.layer, Layer::L1);
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
        assert_eq!(resolve_layer("crate::entities::layer::Layer", &config), Layer::L1);
        assert_eq!(resolve_layer("crate::shell::cli::Cli", &config), Layer::L2);
        assert_eq!(resolve_layer("crate::infra::walker::FileWalker", &config), Layer::L3);
    }

    #[test]
    fn external_crate_resolves_to_unknown() {
        let config = CrystallineConfig::default();
        assert_eq!(resolve_layer("reqwest::Client", &config), Layer::Unknown);
        assert_eq!(resolve_layer("std::fs::read", &config), Layer::Unknown);
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
        let mut file = source_file(
            "impl HasImports for ParsedFile { fn layer(&self) -> u8 { 0 } }",
        );
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
        let kinds: Vec<_> = parsed.declarations.iter().map(|d| (&d.kind, d.name)).collect();
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
        assert!(parsed.declarations.iter().any(|d| d.kind == DeclarationKind::Struct && d.name == "FileWalker"));
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
        assert_eq!(parsed.declarations.iter().filter(|d| d.kind == DeclarationKind::Impl).count(), 0);
        assert_eq!(parsed.declarations.iter().filter(|d| d.kind == DeclarationKind::Struct).count(), 1);
    }

    // ── trait_last_segment unit tests ─────────────────────────────────────

    #[test]
    fn trait_last_segment_strips_prefix() {
        assert_eq!(trait_last_segment("crate::contracts::FileProvider"), "FileProvider");
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
        assert!(imp.is_some(), "should have import for crate::entities::{{...}}");
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
        assert!(!in_imports, "mod declaration must NOT appear in imports after ADR-0013");
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
        let s = parsed.static_declarations.iter().find(|s| s.name == "CACHE");
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
        let s = parsed.static_declarations.iter().find(|s| s.name == "RULE_ID");
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
        let mut file = source_file(
            "impl<T: Alpha + Beta> MyContract for T { fn run(&self) {} }",
        );
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
        let mut file = source_file(
            "impl<T> WhereContract for T where T: SomeBound { fn exec(&self) {} }",
        );
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
        let mut file = source_file(
            "impl<T: World> TrackedWorld for T { fn method(&self) {} }",
        );
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
}
