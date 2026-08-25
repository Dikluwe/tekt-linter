//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/parsers/elixir.md
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

pub struct ElixirParser<R: PromptReader, S: PromptSnapshotReader> {
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
    pub project_root: PathBuf,
}

impl<R: PromptReader, S: PromptSnapshotReader> ElixirParser<R, S> {
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

impl<R: PromptReader, S: PromptSnapshotReader> LanguageParser for ElixirParser<R, S> {
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        if file.content.is_empty() {
            return Err(ParseError::EmptySource { path: file.path.clone() });
        }

        if file.language != Language::Elixir {
            return Err(ParseError::UnsupportedLanguage {
                path: file.path.clone(),
                language: file.language.clone(),
            });
        }

        let mut engine = TsParserEngine::new();
        engine
            .set_language(&tree_sitter_elixir::LANGUAGE.into())
            .map_err(|_| ParseError::SyntaxError {
                path: file.path.clone(),
                line: 0,
                column: 0,
                message: "Failed to load Elixir grammar".to_string(),
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

        let filename = file.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let is_test_file = filename.ends_with("_test.exs") || filename.ends_with("_test.ex");
        let has_exunit_test = has_test_blocks(root, source);
        let has_test_coverage = is_test_file || file.has_adjacent_test || has_exunit_test;

        let mut imports = Vec::new();
        let mut declarations = Vec::new();
        collect_imports_and_decls(root, source, &mut imports, &mut declarations);

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

fn collect_imports_and_decls<'a>(
    node: Node,
    source: &'a [u8],
    imports: &mut Vec<Import<'a>>,
    declarations: &mut Vec<Declaration<'a>>,
) {
    if let Ok(text) = node.utf8_text(source) {
        let trimmed = text.trim();
        if trimmed.starts_with("alias ") || trimmed.starts_with("import ") || trimmed.starts_with("use ") {
            let clean = trimmed
                .trim_start_matches("alias ")
                .trim_start_matches("import ")
                .trim_start_matches("use ")
                .trim();
            let target_layer = if clean.contains("01_core") || clean.contains("Core") {
                Layer::L1
            } else if clean.contains("02_shell") || clean.contains("Shell") {
                Layer::L2
            } else if clean.contains("03_infra") || clean.contains("Infra") {
                Layer::L3
            } else if clean.contains("04_wiring") || clean.contains("Wiring") {
                Layer::L4
            } else {
                Layer::Unknown
            };

            imports.push(Import {
                path: clean,
                line: node.start_position().row + 1,
                kind: ImportKind::Direct,
                target_layer,
                target_subdir: None,
                is_test_origin: false,
            });
        } else if trimmed.starts_with("defmodule ") {
            let name = trimmed.trim_start_matches("defmodule ").split_whitespace().next().unwrap_or("Module");
            declarations.push(Declaration {
                name,
                kind: DeclarationKind::Struct,
                line: node.start_position().row + 1,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_imports_and_decls(child, source, imports, declarations);
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
    if let Ok(text) = node.utf8_text(source) {
        if text.starts_with("File.")
            || text.starts_with("IO.")
            || text.starts_with("Path.")
            || text.starts_with("System.cmd")
            || text.starts_with("HTTPoison.")
            || text.starts_with("Req.")
            || text.starts_with("Ecto.")
        {
            let pos = node.start_position();
            tokens.push(Token {
                symbol: Cow::Borrowed(text),
                line: pos.row + 1,
                column: pos.column,
                kind: TokenKind::CallExpression,
            });
            return;
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_call_tokens(child, source, tokens);
    }
}

fn has_test_blocks(node: Node, source: &[u8]) -> bool {
    if let Ok(text) = node.utf8_text(source) {
        let trimmed = text.trim();
        if trimmed.starts_with("test ") || trimmed.starts_with("use ExUnit.Case") {
            return true;
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if has_test_blocks(child, source) {
            return true;
        }
    }
    false
}
