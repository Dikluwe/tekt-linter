//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/parsers/go.md
//! @prompt-hash PENDING
//! @layer L3
//! @updated 2026-08-11

use std::borrow::Cow;
use std::path::PathBuf;

use tree_sitter::{Node, Parser as TsParserEngine};

use crate::contracts::file_provider::SourceFile;
use crate::contracts::language_parser::LanguageParser;
use crate::contracts::parse_error::ParseError;
use crate::contracts::prompt_reader::PromptReader;
use crate::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crate::entities::layer::{Language, Layer};
use crate::entities::parsed_file::{
    Declaration, DeclarationKind, Import, ImportKind, ParsedFile, PromptHeader, PublicInterface,
    Token, TokenKind,
};
use crate::infra::config::CrystallineConfig;

pub struct GoParser<R: PromptReader, S: PromptSnapshotReader> {
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
    pub project_root: PathBuf,
}

impl<R: PromptReader, S: PromptSnapshotReader> GoParser<R, S> {
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

impl<R: PromptReader, S: PromptSnapshotReader> LanguageParser for GoParser<R, S> {
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        if file.content.is_empty() {
            return Err(ParseError::EmptySource { path: file.path.clone() });
        }

        if file.language != Language::Go {
            return Err(ParseError::UnsupportedLanguage {
                path: file.path.clone(),
                language: file.language.clone(),
            });
        }

        let mut engine = TsParserEngine::new();
        engine
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .map_err(|_| ParseError::SyntaxError {
                path: file.path.clone(),
                line: 0,
                column: 0,
                message: "Failed to load Go grammar".to_string(),
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
        let source = file.content.as_bytes();

        let mut prompt_header = extract_header(&file.content);
        let prompt_file_exists = prompt_header
            .as_ref()
            .map(|h| self.prompt_reader.exists(h.prompt_path))
            .unwrap_or(false);
        if let Some(ref mut header) = prompt_header {
            header.current_hash = self.prompt_reader.read_hash(header.prompt_path);
        }

        let prompt_refs: Vec<&str> = prompt_header
            .as_ref()
            .map(|h| vec![h.prompt_path])
            .unwrap_or_default();

        let is_test_file = file
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .map_or(false, |name| name.ends_with("_test.go"));

        let is_decl_only = file
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .map_or(false, |name| name == "interfaces.go" || name == "types.go");

        let has_test_coverage = is_test_file || file.has_adjacent_test || is_decl_only;

        let mut imports = Vec::new();
        let mut declarations = Vec::new();

        collect_nodes(root, source, &mut imports, &mut declarations);
        let tokens = extract_tokens(root, source, &imports);

        Ok(ParsedFile {
            path: &file.path,
            layer: file.layer.clone(),
            language: file.language.clone(),
            prompt_refs,
            prompt_header,
            prompt_file_exists,
            has_test_coverage,
            prompt_snapshot: None,
            imports,
            module_decls: vec![],
            decision_exprs: vec![],
            constants: vec![],
            semantic_observations: vec![],
            declared_traits: vec![],
            implemented_traits: vec![],
            blanket_impl_traits: vec![],
            public_interface: PublicInterface {
                functions: vec![],
                types: vec![],
                reexports: vec![],
            },
            tokens,
            declarations,
            static_declarations: vec![],
        })
    }
}

fn extract_header(content: &str) -> Option<PromptHeader<'_>> {
    let mut prompt_path = None;
    let mut declared_layer = None;
    let mut prompt_hash = None;
    let mut updated_date = None;

    for line in content.lines().take(30) {
        let trimmed = line.trim();

        if let Some(pos) = trimmed.find("@prompt-hash ") {
            let val = trimmed[pos + 13..].trim();
            if let Some(end) = val.find(|c: char| c.is_whitespace() || c == '*') {
                prompt_hash = Some(val[..end].trim());
            } else {
                prompt_hash = Some(val.trim());
            }
        } else if let Some(pos) = trimmed.find("@prompt ") {
            let val = trimmed[pos + 8..].trim();
            if let Some(end) = val.find(|c: char| c.is_whitespace() || c == '*') {
                prompt_path = Some(val[..end].trim());
            } else {
                prompt_path = Some(val.trim());
            }
        }
        if let Some(pos) = trimmed.find("@layer ") {
            let val = trimmed[pos + 7..].trim();
            let raw = if let Some(end) = val.find(|c: char| c.is_whitespace() || c == '*') {
                val[..end].trim()
            } else {
                val.trim()
            };
            declared_layer = match raw.to_ascii_uppercase().as_str() {
                "L0" => Some(Layer::L0),
                "L1" => Some(Layer::L1),
                "L2" => Some(Layer::L2),
                "L3" => Some(Layer::L3),
                "L4" => Some(Layer::L4),
                "LAB" => Some(Layer::Lab),
                _ => Some(Layer::Unknown),
            };
        }
        if let Some(pos) = trimmed.find("@updated ") {
            let val = trimmed[pos + 9..].trim();
            if let Some(end) = val.find(|c: char| c.is_whitespace() || c == '*') {
                updated_date = Some(val[..end].trim());
            } else {
                updated_date = Some(val.trim());
            }
        }
    }

    if let (Some(path), Some(layer)) = (prompt_path, declared_layer) {
        Some(PromptHeader {
            prompt_path: path,
            prompt_hash,
            current_hash: None,
            layer,
            updated: updated_date,
        })
    } else {
        None
    }
}

fn collect_nodes<'a>(
    node: Node,
    source: &'a [u8],
    imports: &mut Vec<Import<'a>>,
    declarations: &mut Vec<Declaration<'a>>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "import_declaration" => {
                extract_imports(child, source, imports);
            }
            "function_declaration" | "method_declaration" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source) {
                        if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                            declarations.push(Declaration {
                                name,
                                kind: DeclarationKind::Impl,
                                line: child.start_position().row + 1,
                            });
                        }
                    }
                }
            }
            "type_declaration" => {
                let mut type_cursor = child.walk();
                for type_child in child.children(&mut type_cursor) {
                    if type_child.kind() == "type_spec" {
                        if let Some(name_node) = type_child.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source) {
                                if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                                    declarations.push(Declaration {
                                        name,
                                        kind: DeclarationKind::Struct,
                                        line: type_child.start_position().row + 1,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                if child.child_count() > 0 {
                    collect_nodes(child, source, imports, declarations);
                }
            }
        }
    }
}

fn extract_imports<'a>(node: Node, source: &'a [u8], imports: &mut Vec<Import<'a>>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "import_spec" || child.kind() == "interpreted_string_literal" || child.kind() == "raw_string_literal" {
            let text = if child.kind() == "import_spec" {
                let mut inner_cursor = child.walk();
                let found = child.children(&mut inner_cursor)
                    .find(|c| c.kind() == "interpreted_string_literal" || c.kind() == "raw_string_literal");
                found.and_then(|c| c.utf8_text(source).ok())
            } else {
                child.utf8_text(source).ok()
            };

            if let Some(raw_path) = text {
                let clean_path = raw_path.trim_matches('"').trim_matches('`');
                let target_layer = if clean_path.contains("01_core") {
                    Layer::L1
                } else if clean_path.contains("02_shell") {
                    Layer::L2
                } else if clean_path.contains("03_infra") {
                    Layer::L3
                } else if clean_path.contains("04_wiring") {
                    Layer::L4
                } else {
                    Layer::Unknown
                };

                let target_subdir = if clean_path.contains("01_core/domain") {
                    Some("domain")
                } else if clean_path.contains("01_core/parser") {
                    Some("parser")
                } else if clean_path.contains("01_core/usecase") {
                    Some("usecase")
                } else {
                    None
                };

                imports.push(Import {
                    path: clean_path,
                    line: child.start_position().row + 1,
                    kind: ImportKind::Direct,
                    target_layer,
                    target_subdir,
                    is_test_origin: false,
                });
            }
        } else if child.child_count() > 0 {
            extract_imports(child, source, imports);
        }
    }
}

fn extract_tokens<'a>(root: Node, source: &'a [u8], imports: &[Import<'a>]) -> Vec<Token<'a>> {
    let mut tokens: Vec<Token<'a>> = imports
        .iter()
        .map(|imp| Token {
            symbol: Cow::Borrowed(imp.path),
            line: imp.line,
            column: 1,
            kind: TokenKind::CallExpression,
        })
        .collect();

    collect_call_tokens(root, source, &mut tokens);
    tokens
}

fn collect_call_tokens<'a>(node: Node, source: &'a [u8], tokens: &mut Vec<Token<'a>>) {
    if node.kind() == "call_expression" {
        if let Some(func) = node.child_by_field_name("function") {
            if let Ok(text) = func.utf8_text(source) {
                if !text.is_empty() {
                    let pos = node.start_position();
                    tokens.push(Token {
                        symbol: Cow::Borrowed(text),
                        line: pos.row + 1,
                        column: pos.column,
                        kind: TokenKind::CallExpression,
                    });
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_call_tokens(child, source, tokens);
    }
}
