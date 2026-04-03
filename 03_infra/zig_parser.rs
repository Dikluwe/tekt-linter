//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/parsers/zig.md
//! @prompt-hash 789e7202
//! @layer L3
//! @updated 2026-04-03

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use tree_sitter::{Node, Parser as TsParserEngine};

use crate::contracts::file_provider::SourceFile;
use crate::contracts::language_parser::LanguageParser;
use crate::contracts::parse_error::ParseError;
use crate::contracts::prompt_reader::PromptReader;
use crate::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crate::entities::layer::{Language, Layer};
use crate::entities::parsed_file::{
    Declaration, DeclarationKind, FunctionSignature, Import, ImportKind, ParsedFile,
    PromptHeader, PublicInterface, StaticDeclaration, Token, TokenKind, TypeKind, TypeSignature,
};
use crate::infra::config::CrystallineConfig;
use crate::infra::walker::resolve_file_layer;

pub struct ZigParser<R: PromptReader, S: PromptSnapshotReader> {
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
    pub project_root: PathBuf,
}

impl<R: PromptReader, S: PromptSnapshotReader> ZigParser<R, S> {
    pub fn new(
        prompt_reader: R,
        snapshot_reader: S,
        config: CrystallineConfig,
        project_root: PathBuf,
    ) -> Self {
        Self {
            prompt_reader,
            snapshot_reader,
            config,
            project_root,
        }
    }
}

impl<R: PromptReader, S: PromptSnapshotReader> LanguageParser for ZigParser<R, S> {
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        if file.content.is_empty() {
            return Err(ParseError::EmptySource { path: file.path.clone() });
        }

        if file.language != Language::Zig {
            return Err(ParseError::UnsupportedLanguage {
                path: file.path.clone(),
                language: file.language.clone(),
            });
        }

        let mut engine = TsParserEngine::new();
        engine.set_language(&tree_sitter_zig::LANGUAGE.into()).map_err(|_| ParseError::SyntaxError {
            path: file.path.clone(),
            line: 0,
            column: 0,
            message: "Failed to load Zig grammar".to_string(),
        })?;

        let tree = engine
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
                message: "Syntax error detected in Zig AST".to_string(),
            });
        }

        let source = file.content.as_bytes();

        let mut prompt_header = extract_header(&file.content);
        let prompt_file_exists = prompt_header
            .as_ref()
            .map(|h| self.prompt_reader.exists(h.prompt_path))
            .unwrap_or(false);
        if let Some(ref mut header) = prompt_header {
            header.current_hash = self.prompt_reader.read_hash(header.prompt_path);
        }

        let imports = extract_imports(root, source, file.path.as_path(), &self.project_root, &self.config);

        let tokens = extract_tokens(root, source);

        // Zig uses `test "name" { ... }` blocks
        let has_test_ast = has_test_blocks(root, source);
        let has_test_coverage = has_test_ast || file.has_adjacent_test;

        let public_interface = extract_public_interface(root, source, file.path.as_path());
        let prompt_snapshot = prompt_header
            .as_ref()
            .and_then(|h| self.snapshot_reader.read_snapshot(h.prompt_path));

        let declarations = extract_declarations(root, source);
        let static_declarations = extract_static_declarations(root, source);

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
            declared_traits: vec![],
            implemented_traits: vec![],
            blanket_impl_traits: vec![],
            declarations,
            static_declarations,
            module_decls: vec![],
        })
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_first_error_pos(node: Node) -> (usize, usize) {
    if node.is_error() || node.is_missing() {
        return (node.start_position().row + 1, node.start_position().column);
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.has_error() {
                return find_first_error_pos(child);
            }
        }
    }
    (node.start_position().row + 1, node.start_position().column)
}

fn node_text<'a>(node: Node, source: &'a [u8]) -> &'a str {
    std::str::from_utf8(&source[node.start_byte()..node.end_byte()]).unwrap_or("")
}

// ── Header extraction ─────────────────────────────────────────────────────────

fn extract_header<'a>(source: &'a str) -> Option<PromptHeader<'a>> {
    let mut prompt_path: Option<&'a str> = None;
    let mut prompt_hash: Option<&'a str> = None;
    let mut layer: Option<Layer> = None;
    let mut updated: Option<&'a str> = None;

    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("//") {
            break;
        }
        let content = trimmed.trim_start_matches('/').trim();

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
        current_hash: None,
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
        "Lab"| "lab" => Layer::Lab,
        _ => Layer::Unknown,
    }
}

// ── Import extraction ─────────────────────────────────────────────────────────

fn extract_imports<'a>(
    root: Node,
    source: &'a [u8],
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
) -> Vec<Import<'a>> {
    let mut imports = Vec::new();
    collect_imports(root, source, file_path, project_root, config, &mut imports);
    imports
}

fn collect_imports<'a>(
    node: Node,
    source: &'a [u8],
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
    imports: &mut Vec<Import<'a>>,
) {
    // Zig imports: `@import("std")`, `@import("./local.zig")`
    if node.kind() == "builtin_call_expr" {
        let name_node = node.child(0); // usually @import
        if let Some(n) = name_node {
            if node_text(n, source) == "@import" {
                if let Some(args) = node.child_by_field_name("arguments") {
                    if let Some(arg) = args.child(1) { // 0 is '(', 1 is the string
                        let text = node_text(arg, source);
                        if text.len() >= 2 {
                            let p = &text[1..text.len()-1];
                            let line = node.start_position().row + 1;
                            let target_layer = resolve_zig_layer(p, file_path, project_root, config);
                            imports.push(Import {
                                path: p,
                                line,
                                kind: ImportKind::Direct,
                                target_layer,
                                target_subdir: None,
                            });
                        }
                    }
                }
            }
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_imports(child, source, file_path, project_root, config, imports);
        }
    }
}

fn normalize(path: &Path, project_root: &Path) -> Option<PathBuf> {
    use std::path::Component;
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                if components.is_empty() { return None; }
                components.pop();
            }
            Component::CurDir => {}
            c => components.push(c),
        }
    }
    let result: PathBuf = components.iter().collect();
    if project_root != Path::new(".") && !project_root.as_os_str().is_empty() {
        if !result.starts_with(project_root) { return None; }
    }
    Some(result)
}

fn resolve_zig_layer(
    import_path: &str,
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
) -> Layer {
    if !import_path.starts_with(".") {
        return Layer::Unknown; // stdlib or package
    }
    let base = file_path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let joined = base.join(import_path);
    if let Some(normalized) = normalize(&joined, project_root) {
        resolve_file_layer(&normalized, project_root, config)
    } else {
        Layer::Unknown
    }
}

// ── PublicInterface extraction ────────────────────────────────────────────────

fn extract_public_interface<'a>(root: Node, source: &'a [u8], _file_path: &Path) -> PublicInterface<'a> {
    let mut functions = Vec::new();
    let mut types = Vec::new();

    collect_public_members(root, source, &mut functions, &mut types);

    PublicInterface { functions, types, reexports: vec![] }
}

fn collect_public_members<'a>(
    node: Node,
    source: &'a [u8],
    functions: &mut Vec<FunctionSignature<'a>>,
    types: &mut Vec<TypeSignature<'a>>,
) {
    match node.kind() {
        "fn_proto" => {
            if is_pub(node, source) {
                if let Some(name_node) = node.child_by_field_name("name") {
                    functions.push(FunctionSignature {
                        name: node_text(name_node, source),
                        params: vec![], // simplified
                        return_type: None, // simplified
                    });
                }
            }
        }
        "variable_declaration" => {
            if is_pub(node, source) {
                // If it's a const X = struct { ... }, treat as a type
                if let Some(value) = node.child_by_field_name("value") {
                    if value.kind() == "container_decl" {
                        if let Some(name_node) = node.child_by_field_name("name") {
                            types.push(TypeSignature {
                                name: node_text(name_node, source),
                                kind: TypeKind::Struct,
                                members: vec![],
                            });
                        }
                    }
                }
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_public_members(child, source, functions, types);
        }
    }
}

fn is_pub(node: Node, source: &[u8]) -> bool {
    // Check if the node or its parent has "pub" keyword
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if node_text(child, source) == "pub" {
                return true;
            }
        }
    }
    false
}

// ── Token extraction ──────────────────────────────────────────────────────────

fn extract_tokens<'a>(root: Node, source: &'a [u8]) -> Vec<Token<'a>> {
    let mut tokens = Vec::new();
    collect_tokens(root, source, &mut tokens);
    tokens
}

fn collect_tokens<'a>(node: Node, source: &'a [u8], tokens: &mut Vec<Token<'a>>) {
    if node.kind() == "call_expr" {
        if let Some(func) = node.child(0) {
            let symbol = Cow::Borrowed(node_text(func, source));
            let pos = node.start_position();
            tokens.push(Token {
                symbol,
                line: pos.row + 1,
                column: pos.column,
                kind: TokenKind::CallExpression,
            });
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_tokens(child, source, tokens);
        }
    }
}

// ── Test Coverage ────────────────────────────────────────────────────────────

fn has_test_blocks(root: Node, source: &[u8]) -> bool {
    let mut found = false;
    check_test_nodes(root, source, &mut found);
    found
}

fn check_test_nodes(node: Node, _source: &[u8], found: &mut bool) {
    if *found { return; }
    if node.kind() == "test_declaration" {
        *found = true;
        return;
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            check_test_nodes(child, _source, found);
        }
    }
}

// ── Declarations ─────────────────────────────────────────────────────────────

fn extract_declarations<'a>(root: Node, source: &'a [u8]) -> Vec<Declaration<'a>> {
    let mut decls = Vec::new();
    collect_decls(root, source, &mut decls);
    decls
}

fn collect_decls<'a>(node: Node, source: &'a [u8], decls: &mut Vec<Declaration<'a>>) {
    if node.kind() == "variable_declaration" {
        if let Some(value) = node.child_by_field_name("value") {
            if value.kind() == "container_decl" {
                if let Some(name_node) = node.child_by_field_name("name") {
                    decls.push(Declaration {
                        kind: DeclarationKind::Struct,
                        name: node_text(name_node, source),
                        line: node.start_position().row + 1,
                    });
                }
            }
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_decls(child, source, decls);
        }
    }
}

fn extract_static_declarations<'a>(root: Node, source: &'a [u8]) -> Vec<StaticDeclaration<'a>> {
    let mut decls = Vec::new();
    collect_statics(root, source, &mut decls);
    decls
}

fn collect_statics<'a>(node: Node, source: &'a [u8], decls: &mut Vec<StaticDeclaration<'a>>) {
    if node.kind() == "variable_declaration" {
        // Top-level variable in source_file usually
        if node.parent().map_or(false, |p| p.kind() == "source_file") {
            let is_mut = node_text(node, source).contains("var");
            if let Some(name_node) = node.child_by_field_name("name") {
                decls.push(StaticDeclaration {
                    name: node_text(name_node, source),
                    type_text: "", // simplified
                    is_mut,
                    line: node.start_position().row + 1,
                });
            }
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_statics(child, source, decls);
        }
    }
}
