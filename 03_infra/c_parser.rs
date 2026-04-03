//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/parsers/c.md
//! @prompt-hash 6dd066e5
//! @layer L3
//! @updated 2026-03-30

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

pub struct CParser<R: PromptReader, S: PromptSnapshotReader> {
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
    pub project_root: PathBuf,
}

impl<R: PromptReader, S: PromptSnapshotReader> CParser<R, S> {
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

impl<R: PromptReader, S: PromptSnapshotReader> LanguageParser for CParser<R, S> {
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        if file.content.is_empty() {
            return Err(ParseError::EmptySource { path: file.path.clone() });
        }

        if file.language != Language::C {
            return Err(ParseError::UnsupportedLanguage {
                path: file.path.clone(),
                language: file.language.clone(),
            });
        }

        let mut engine = TsParserEngine::new();
        engine.set_language(&tree_sitter_c::LANGUAGE.into()).map_err(|_| ParseError::SyntaxError {
            path: file.path.clone(),
            line: 0,
            column: 0,
            message: "Failed to load C grammar".to_string(),
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
                message: "Syntax error detected in C AST".to_string(),
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

        // For C, testing is very macro dependent (e.g. Unity, Check, cmocka)
        let has_test_ast = has_test_calls(root, source);
        let is_decl_only = file.path.extension().unwrap_or_default() == "h";
        let has_test_coverage = has_test_ast || file.has_adjacent_test || is_decl_only;

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

// ── Header extraction ─────────────────────────────────────────────────────────

fn extract_header<'a>(source: &'a str) -> Option<PromptHeader<'a>> {
    let mut prompt_path: Option<&'a str> = None;
    let mut prompt_hash: Option<&'a str> = None;
    let mut layer: Option<Layer> = None;
    let mut updated: Option<&'a str> = None;

    for line in source.lines() {
        let trimmed = line.trim();
        // C comments: // or ///
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
        "Lab" | "lab" => Layer::Lab,
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
    if node.kind() == "preproc_include" {
        // Find the string or system_lib_string
        let mut path_str = None;
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "string_literal" || child.kind() == "system_lib_string" {
                    let text = node_text(child, source);
                    if text.len() >= 2 {
                        path_str = Some(&text[1..text.len() - 1]);
                    }
                }
            }
        }

        if let Some(p) = path_str {
            let line = node.start_position().row + 1;
            let target_layer = resolve_c_layer(p, file_path, project_root, config);
            imports.push(Import {
                path: p,
                line,
                kind: ImportKind::Direct,
                target_layer,
                target_subdir: None,
            });
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
                if components.is_empty() {
                    return None;
                }
                components.pop();
            }
            Component::CurDir => {}
            c => components.push(c),
        }
    }
    let result: PathBuf = components.iter().collect();
    if project_root != Path::new(".") && !project_root.as_os_str().is_empty() {
        if !result.starts_with(project_root) {
            return None;
        }
    }
    Some(result)
}

fn resolve_c_layer(
    import_path: &str,
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
) -> Layer {
    let is_relative = import_path.starts_with("./") || import_path.starts_with("../");
    
    // Simplification for C: if it's not relative we treat it as Layer::Unknown 
    // Usually system headers or local include dirs (-I) which we don't resolve strictly yet.
    if !is_relative {
        return Layer::Unknown;
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

fn extract_public_interface<'a>(root: Node, source: &'a [u8], file_path: &Path) -> PublicInterface<'a> {
    let mut functions = Vec::new();
    let mut types = Vec::new();
    
    let is_header = file_path.extension().unwrap_or_default() == "h";

    for i in 0..root.child_count() {
        if let Some(child) = root.child(i) {
            match child.kind() {
                "function_definition" | "declaration" => {
                    // For C, consider what is "public". If it is in a `.h`, it's public.
                    // If it's a `.c` file and doesn't have `static`, it might be public, but
                    // let's only expose those actually in `.h` files or non-static functions as a fallback.
                    let is_static = child.child(0).map_or(false, |n| node_text(n, source) == "static");
                    
                    if !is_static || is_header {
                        if child.kind() == "function_definition" {
                            if let Some(sig) = extract_fn_sig(child, source) {
                                functions.push(sig);
                            }
                        } else if let Some(sig) = extract_decl_sig(child, source) {
                            // Extract struct/typedef signals from declarations
                            types.push(sig);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    PublicInterface { functions, types, reexports: vec![] }
}

fn extract_fn_sig<'a>(node: Node, source: &'a [u8]) -> Option<FunctionSignature<'a>> {
    let decl = node.child_by_field_name("declarator")?;
    let mut name = None;
    let mut params = Vec::new();
    
    // In C, declarator can be a function_declarator
    if decl.kind() == "function_declarator" {
        if let Some(n) = decl.child_by_field_name("declarator") {
            name = Some(node_text(n, source));
        }
        if let Some(p) = decl.child_by_field_name("parameters") {
            params.push(node_text(p, source));
        }
    }
    
    let return_type = node.child_by_field_name("type").map(|n| node_text(n, source));
    
    name.map(|n| FunctionSignature { name: n, params, return_type })
}

fn extract_decl_sig<'a>(node: Node, source: &'a [u8]) -> Option<TypeSignature<'a>> {
    let type_node = node.child_by_field_name("type")?;
    
    if type_node.kind() == "struct_specifier" || type_node.kind() == "enum_specifier" {
        let name = type_node.child_by_field_name("name").map(|n| node_text(n, source))?;
        let kind = if type_node.kind() == "struct_specifier" { TypeKind::Struct } else { TypeKind::Enum };
        return Some(TypeSignature { name, kind, members: vec![] });
    }
    
    None
}

// ── Token extraction ──────────────────────────────────────────────────────────

fn extract_tokens<'a>(root: Node, source: &'a [u8]) -> Vec<Token<'a>> {
    let mut tokens = Vec::new();
    collect_tokens(root, source, &mut tokens);
    tokens
}

fn collect_tokens<'a>(node: Node, source: &'a [u8], tokens: &mut Vec<Token<'a>>) {
    if node.kind() == "call_expression" {
        if let Some(func) = node.child_by_field_name("function") {
            let symbol = Cow::Borrowed(node_text(func, source));
            let pos = node.start_position();
            tokens.push(Token {
                symbol,
                line: pos.row + 1,
                column: pos.column,
                kind: TokenKind::CallExpression,
            });
        }
    } else if node.kind() == "identifier" {
        // Collect macros or constants if needed
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_tokens(child, source, tokens);
        }
    }
}

// ── Test Coverage ────────────────────────────────────────────────────────────

fn has_test_calls(root: Node, source: &[u8]) -> bool {
    let mut found = false;
    check_c_test_calls(root, source, &mut found);
    found
}

fn check_c_test_calls(node: Node, source: &[u8], found: &mut bool) {
    if *found { return; }
    
    if node.kind() == "call_expression" {
        if let Some(func) = node.child_by_field_name("function") {
            let text = node_text(func, source);
            // Some generic C test macros:
            if text == "TEST" || text == "RUN_TEST" || text.starts_with("assert_") {
                *found = true;
                return;
            }
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            check_c_test_calls(child, source, found);
        }
    }
}

// ── Declarations ─────────────────────────────────────────────────────────────

fn extract_declarations<'a>(root: Node, source: &'a [u8]) -> Vec<Declaration<'a>> {
    let mut decls = Vec::new();
    for i in 0..root.child_count() {
        if let Some(node) = root.child(i) {
            if node.kind() == "declaration" {
                if let Some(sig) = extract_decl_sig(node, source) {
                    let d_kind = match sig.kind {
                        TypeKind::Struct => DeclarationKind::Struct,
                        TypeKind::Enum => DeclarationKind::Enum,
                        _ => DeclarationKind::TypeAlias,
                    };
                    decls.push(Declaration {
                        kind: d_kind,
                        name: sig.name,
                        line: node.start_position().row + 1,
                    });
                }
            }
        }
    }
    decls
}

fn extract_static_declarations<'a>(root: Node, source: &'a [u8]) -> Vec<StaticDeclaration<'a>> {
    let mut decls = Vec::new();
    for i in 0..root.child_count() {
        if let Some(node) = root.child(i) {
            if node.kind() == "declaration" {
                let is_static = node.child(0).map_or(false, |n| node_text(n, source) == "static");
                // For global variables with static keyword
                if is_static {
                    if let Some(decl) = node.child_by_field_name("declarator") {
                        let name = node_text(decl, source); // simplified
                        let type_text = node.child_by_field_name("type").map(|n| node_text(n, source)).unwrap_or("");
                        decls.push(StaticDeclaration {
                            name,
                            type_text,
                            is_mut: true, // simplified
                            line: node.start_position().row + 1,
                        });
                    }
                }
            }
        }
    }
    decls
}
