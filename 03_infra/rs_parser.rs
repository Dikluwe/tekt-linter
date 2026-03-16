//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rs-parser.md
//! @prompt-hash 36b3db44
//! @layer L3
//! @updated 2026-03-14

use std::borrow::Cow;

use tree_sitter::{Node, Parser as TsParser};

use crate::contracts::file_provider::SourceFile;
use crate::contracts::language_parser::LanguageParser;
use crate::contracts::parse_error::ParseError;
use crate::contracts::prompt_reader::PromptReader;
use crate::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crate::entities::layer::{Language, Layer};
use crate::entities::parsed_file::{
    FunctionSignature, Import, ImportKind, ParsedFile, PromptHeader, PublicInterface, Token,
    TokenKind, TypeKind, TypeSignature,
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
            imports.push(Import { path, line, kind: ImportKind::Use, target_layer, target_subdir });
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
            imports.push(Import { path, line, kind: ImportKind::ExternCrate, target_layer, target_subdir });
        }
        "mod_item" => {
            // Only bare `mod foo;` declarations (no block body)
            if !node_has_child_kind(node, "declaration_list") {
                let line = node.start_position().row + 1;
                let text = node_text(node, source);
                let path = text
                    .trim_start_matches("pub ")
                    .trim_start_matches("mod ")
                    .trim_end_matches(';')
                    .trim();
                imports.push(Import {
                    path,
                    line,
                    kind: ImportKind::ModDecl,
                    target_layer: Layer::Unknown,
                    target_subdir: None,
                });
            }
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
    if node.is_error() {
        let pos = node.start_position();
        return (pos.row + 1, pos.column);
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.has_error() {
                return find_first_error_pos(child);
            }
        }
    }
    (0, 0)
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
}
