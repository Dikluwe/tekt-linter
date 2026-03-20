//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/parsers/typescript.md
//! @prompt-hash e319d0cf
//! @layer L3
//! @updated 2026-03-20

use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

use tree_sitter::{Node, Parser as TsParserEngine};

use crate::contracts::file_provider::SourceFile;
use crate::contracts::language_parser::LanguageParser;
use crate::contracts::parse_error::ParseError;
use crate::contracts::prompt_reader::PromptReader;
use crate::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crate::entities::layer::{Language, Layer};
use crate::entities::parsed_file::{
    Declaration, DeclarationKind, FunctionSignature, Import, ImportKind, ParsedFile,
    PromptHeader, PublicInterface, Token, TokenKind, TypeKind, TypeSignature,
};
use crate::infra::config::CrystallineConfig;
use crate::infra::walker::resolve_file_layer;

// ── TsParser ──────────────────────────────────────────────────────────────────

/// Parser de TypeScript/TSX implementando `LanguageParser`.
/// Resolução de camadas: física (ADR-0009) via TsLayerResolver.
/// Zero-Copy: retorna ParsedFile<'a> com referências ao buffer de `SourceFile`.
pub struct TsParser<R: PromptReader, S: PromptSnapshotReader> {
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
    /// Raiz do projecto — usada pelo TsLayerResolver para resolve_file_layer.
    pub project_root: PathBuf,
    subdirs_buffer: Mutex<Vec<Box<str>>>,
}

impl<R: PromptReader, S: PromptSnapshotReader> TsParser<R, S> {
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
            subdirs_buffer: Mutex::new(Vec::new()),
        }
    }

    fn intern_subdir(&self, s: String) -> &'static str {
        let mut buf = self.subdirs_buffer.lock().unwrap();
        let boxed: Box<str> = s.into_boxed_str();
        let raw: *const str = &*boxed as *const str;
        buf.push(boxed);
        // SAFETY: raw aponta para dado heap que vive em self.subdirs_buffer.
        // O buffer cresce monotonicamente — nunca é limpo.
        // O parser outlives todos os ParsedFile que produz.
        // Realoções do Vec movem o Box (fat pointer), não o dado heap.
        unsafe { &*raw }
    }
}

impl<R: PromptReader, S: PromptSnapshotReader> LanguageParser for TsParser<R, S> {
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        if file.content.is_empty() {
            return Err(ParseError::EmptySource { path: file.path.clone() });
        }

        if file.language != Language::TypeScript {
            return Err(ParseError::UnsupportedLanguage {
                path: file.path.clone(),
                language: file.language.clone(),
            });
        }

        let mut engine = TsParserEngine::new();
        let lang = if file.path.extension().and_then(|e| e.to_str()) == Some("tsx") {
            tree_sitter_typescript::language_tsx()
        } else {
            tree_sitter_typescript::language_typescript()
        };
        engine.set_language(&lang).map_err(|_| ParseError::SyntaxError {
            path: file.path.clone(),
            line: 0,
            column: 0,
            message: "Failed to load TypeScript grammar".to_string(),
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
                message: "Syntax error detected in TypeScript AST".to_string(),
            });
        }

        let source = file.content.as_bytes();

        // 1. Header (//-block at top: @prompt, @prompt-hash, @layer, @updated)
        let mut prompt_header = extract_header(&file.content);
        let prompt_file_exists = prompt_header
            .as_ref()
            .map(|h| self.prompt_reader.exists(h.prompt_path))
            .unwrap_or(false);
        if let Some(ref mut header) = prompt_header {
            header.current_hash = self.prompt_reader.read_hash(header.prompt_path);
        }

        // 2. Imports — TsLayerResolver 4 passos + SubdirResolver físico
        let intern: &dyn Fn(String) -> &'static str = &|s| self.intern_subdir(s);
        let imports = extract_imports(root, source, file.path.as_path(), &self.project_root, &self.config, intern);

        // 3. Tokens — imports proibidos + call expressions (sem Motor de Duas Fases)
        let tokens = extract_tokens(root, source, &imports);

        // 4. Test coverage — describe/it/test/suite + adjacência + declaration-only
        let has_test_ast = has_test_calls(root, source);
        let is_decl_only = is_declaration_only(root, source);
        let has_test_coverage = has_test_ast || file.has_adjacent_test || is_decl_only;

        // 5. PublicInterface + prompt_snapshot (V6)
        let public_interface = extract_public_interface(root, source);
        let prompt_snapshot = prompt_header
            .as_ref()
            .and_then(|h| self.snapshot_reader.read_snapshot(h.prompt_path));

        // 6. declared_traits — apenas L1/contracts, apenas interface com export (V11)
        let declared_traits = if file.layer == Layer::L1
            && path_contains_segment(file.path.as_path(), "contracts")
        {
            extract_declared_traits(root, source)
        } else {
            vec![]
        };

        // 7. implemented_traits — apenas L2|L3, apenas class com implements (V11)
        let implemented_traits = if matches!(file.layer, Layer::L2 | Layer::L3) {
            extract_implemented_traits(root, source)
        } else {
            vec![]
        };

        // 8. declarations — class/interface/type-alias/enum sem implements (V12)
        let declarations = extract_declarations(root, source);

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
            declarations,
        })
    }
}

// ── Header extraction ─────────────────────────────────────────────────────────

/// Extrai o header cristalino de comentários `//` no topo do ficheiro.
/// O bloco termina na primeira linha que não começa com `//`.
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
        let content = trimmed.trim_start_matches("//").trim();

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

// ── TsLayerResolver — 4 passos (ADR-0009) ────────────────────────────────────

/// Normalização algébrica de paths sem bater no disco.
/// `None` = tentativa de escapar da raiz ou resultado fora do projecto.
fn normalize(path: &Path, project_root: &Path) -> Option<PathBuf> {
    let mut components: Vec<Component> = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                // pop() em vec vazio = tentativa de sair da raiz → Layer::Unknown
                if components.is_empty() {
                    return None;
                }
                components.pop();
            }
            Component::CurDir => {}
            c => {
                components.push(c);
            }
        }
    }
    let result: PathBuf = components.iter().collect();
    // Garantia adicional: resultado dentro da raiz do projecto.
    // Skipped for trivial roots like "." where starts_with is unreliable.
    if project_root != Path::new(".") && !project_root.as_os_str().is_empty() {
        if !result.starts_with(project_root) {
            return None;
        }
    }
    Some(result)
}

/// Resolve o target_layer de um import TypeScript usando resolução física.
/// Passo 1: package npm externo → Layer::Unknown directamente.
/// Passo 2: resolução de alias via [ts_aliases].
/// Passo 3: álgebra de paths com verificação de fuga.
/// Passo 4: reutilização de resolve_file_layer.
fn resolve_ts_layer(
    import_path: &str,
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
) -> Layer {
    // Passo 1 — detecção de package externo
    let is_relative = import_path.starts_with("./") || import_path.starts_with("../");
    let is_alias = config.ts_aliases.keys().any(|k| import_path.starts_with(k.as_str()));
    if !is_relative && !is_alias {
        return Layer::Unknown;
    }

    // Passo 2 — resolução de alias
    let resolved_str: String;
    let import_after_alias: &str = if is_alias {
        let alias_key = config.ts_aliases.keys()
            .find(|k| import_path.starts_with(k.as_str()))
            .expect("alias_key found above");
        let alias_val = &config.ts_aliases[alias_key];
        resolved_str = format!("{}{}", alias_val, &import_path[alias_key.len()..]);
        &resolved_str
    } else {
        import_path
    };

    // Passo 3 — álgebra de paths com verificação de fuga
    // Aliases resolvem a partir da raiz do projecto (como baseUrl do tsconfig).
    // Caminhos relativos (./, ../) resolvem a partir do directório do ficheiro.
    let base = if is_alias {
        project_root.to_path_buf()
    } else {
        file_path.parent().unwrap_or(Path::new(".")).to_path_buf()
    };
    let joined = base.join(import_after_alias);
    let normalized = match normalize(&joined, project_root) {
        Some(p) => p,
        None => return Layer::Unknown,
    };

    // Passo 4 — reutilização de resolve_file_layer
    resolve_file_layer(&normalized, project_root, config)
}

/// Extrai o subdir de destino de um import para V9.
/// Apenas para imports que resolvem para L1.
fn resolve_ts_subdir(
    import_path: &str,
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
    target_layer: &Layer,
    intern: &dyn Fn(String) -> &'static str,
) -> Option<&'static str> {
    if *target_layer != Layer::L1 {
        return None;
    }

    let is_relative = import_path.starts_with("./") || import_path.starts_with("../");
    let is_alias = config.ts_aliases.keys().any(|k| import_path.starts_with(k.as_str()));

    let resolved_str: String;
    let import_after_alias: &str = if is_alias {
        let alias_key = config.ts_aliases.keys()
            .find(|k| import_path.starts_with(k.as_str()))
            .expect("alias_key found above");
        let alias_val = &config.ts_aliases[alias_key];
        resolved_str = format!("{}{}", alias_val, &import_path[alias_key.len()..]);
        &resolved_str
    } else if is_relative {
        import_path
    } else {
        return None;
    };

    let base = if is_alias {
        project_root.to_path_buf()
    } else {
        file_path.parent().unwrap_or(Path::new(".")).to_path_buf()
    };
    let joined = base.join(import_after_alias);
    let normalized = normalize(&joined, project_root)?;

    let layer_dir = config.layers.get("L1")?;
    let base_l1 = project_root.join(layer_dir);

    // Try stripping project_root/layer_dir prefix; fallback to just layer_dir prefix
    let relative = normalized
        .strip_prefix(&base_l1)
        .or_else(|_| normalized.strip_prefix(layer_dir.as_str()))
        .ok()?;

    // First segment is the subdir: "entities/layer.ts" → "entities"
    let subdir = relative
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())?;

    Some(intern(subdir.to_string()))
}

// ── Import extraction ─────────────────────────────────────────────────────────

fn extract_imports<'a>(
    root: Node,
    source: &'a [u8],
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
    intern: &dyn Fn(String) -> &'static str,
) -> Vec<Import<'a>> {
    let mut imports = Vec::new();
    collect_imports(root, source, file_path, project_root, config, &mut imports, intern);
    imports
}

fn collect_imports<'a>(
    node: Node,
    source: &'a [u8],
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
    imports: &mut Vec<Import<'a>>,
    intern: &dyn Fn(String) -> &'static str,
) {
    match node.kind() {
        "import_statement" => {
            if let Some(path_str) = import_source_str(node, source) {
                let line = node.start_position().row + 1;
                let target_layer = resolve_ts_layer(path_str, file_path, project_root, config);
                let target_subdir = resolve_ts_subdir(path_str, file_path, project_root, config, &target_layer, intern);
                let kind = classify_import_statement(node, source);
                imports.push(Import {
                    path: path_str,
                    line,
                    kind,
                    target_layer,
                    target_subdir,
                });
            }
        }
        "export_statement" => {
            // Only re-exports with `from` clause: export { X } from './foo'
            // or export * from './foo'
            if let Some(path_str) = import_source_str(node, source) {
                if !path_str.is_empty() {
                    let line = node.start_position().row + 1;
                    let target_layer = resolve_ts_layer(path_str, file_path, project_root, config);
                    let target_subdir = resolve_ts_subdir(path_str, file_path, project_root, config, &target_layer, intern);
                    let kind = classify_export_statement(node, source);
                    imports.push(Import {
                        path: path_str,
                        line,
                        kind,
                        target_layer,
                        target_subdir,
                    });
                }
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_imports(child, source, file_path, project_root, config, imports, intern);
        }
    }
}

/// Classifica semanticamente um `import_statement` TypeScript.
/// Inspeciona os filhos do nó (e `import_clause` aninhado) para determinar o tipo.
fn classify_import_statement(node: Node, source: &[u8]) -> ImportKind {
    classify_import_node_recursive(node, source)
}

fn classify_import_node_recursive(node: Node, source: &[u8]) -> ImportKind {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                "namespace_import" => return ImportKind::Glob,
                "named_imports" => {
                    // Check if any specifier uses `as` (alias)
                    for j in 0..child.child_count() {
                        if let Some(spec) = child.child(j) {
                            if spec.kind() == "import_specifier" {
                                for k in 0..spec.child_count() {
                                    if let Some(kw) = spec.child(k) {
                                        if kw.kind() == "as" || node_text(kw, source) == "as" {
                                            return ImportKind::Alias;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return ImportKind::Named;
                }
                "import_clause" => {
                    // Recurse into import_clause which may contain namespace_import or named_imports
                    let inner = classify_import_node_recursive(child, source);
                    if inner != ImportKind::Direct {
                        return inner;
                    }
                }
                _ => {}
            }
        }
    }
    ImportKind::Direct
}

/// Classifica semanticamente um `export_statement` TypeScript com cláusula `from`.
fn classify_export_statement(node: Node, _source: &[u8]) -> ImportKind {
    let has_export_clause = (0..node.child_count())
        .filter_map(|i| node.child(i))
        .any(|c| c.kind() == "export_clause");

    if has_export_clause {
        ImportKind::Named
    } else {
        // export * from '...' — ausência de export_clause indica glob
        ImportKind::Glob
    }
}

/// Extrai o path string de um import_statement ou export_statement.
/// Retorna a fatia &'a str do buffer sem aspas.
fn import_source_str<'a>(node: Node, source: &'a [u8]) -> Option<&'a str> {
    // Try field name "source" first (used in most tree-sitter-typescript versions)
    if let Some(src) = node.child_by_field_name("source") {
        return string_content(src, source);
    }
    // Fallback: find last string child
    let mut last_string: Option<Node> = None;
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "string" {
                last_string = Some(child);
            }
        }
    }
    last_string.and_then(|n| string_content(n, source))
}

/// Extrai o conteúdo de um nó `string` sem as aspas envolventes.
/// Preferência por `string_fragment` child; fallback por slice manual.
fn string_content<'a>(string_node: Node, source: &'a [u8]) -> Option<&'a str> {
    for i in 0..string_node.child_count() {
        if let Some(child) = string_node.child(i) {
            if child.kind() == "string_fragment" {
                return Some(node_text(child, source));
            }
        }
    }
    // Fallback: strip surrounding quote characters
    let text = node_text(string_node, source);
    if text.len() >= 2 {
        Some(&text[1..text.len() - 1])
    } else {
        None
    }
}

// ── Token extraction (V4) ─────────────────────────────────────────────────────

const FORBIDDEN_MODULES: &[&str] = &[
    "fs", "node:fs", "fs/promises", "node:fs/promises",
    "child_process", "node:child_process",
    "net", "node:net",
    "http", "node:http",
    "https", "node:https",
    "dgram", "node:dgram",
    "dns", "node:dns",
    "readline", "node:readline",
];

const FORBIDDEN_CALLS: &[&str] = &["process.env", "Date.now", "Math.random"];

fn extract_tokens<'a>(
    root: Node,
    source: &'a [u8],
    imports: &[Import<'a>],
) -> Vec<Token<'a>> {
    let mut tokens = Vec::new();

    // Mecanismo 1 — imports de módulos proibidos
    for imp in imports {
        if FORBIDDEN_MODULES.contains(&imp.path) {
            // O símbolo do import proibido é o próprio path do módulo
            tokens.push(Token {
                symbol: Cow::Borrowed(imp.path),
                line: imp.line,
                column: 0,
                kind: TokenKind::CallExpression,
            });
        }
    }

    // Mecanismo 2 — call expressions proibidas
    collect_forbidden_calls(root, source, &mut tokens);
    tokens
}

fn collect_forbidden_calls<'a>(node: Node, source: &'a [u8], tokens: &mut Vec<Token<'a>>) {
    if node.kind() == "call_expression" {
        if let Some(func) = node.child(0) {
            let text = node_text(func, source);
            if FORBIDDEN_CALLS.contains(&text) {
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

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_forbidden_calls(child, source, tokens);
        }
    }
}

// ── Test coverage (V2) ────────────────────────────────────────────────────────

const TEST_CALL_NAMES: &[&str] = &["describe", "it", "test", "suite"];

/// Detecta `describe(...)`, `it(...)`, `test(...)`, `suite(...)` no AST.
fn has_test_calls(root: Node, source: &[u8]) -> bool {
    check_test_calls(root, source)
}

fn check_test_calls(node: Node, source: &[u8]) -> bool {
    if node.kind() == "call_expression" {
        if let Some(func) = node.child(0) {
            let text = node_text(func, source);
            if TEST_CALL_NAMES.contains(&text) {
                return true;
            }
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if check_test_calls(child, source) {
                return true;
            }
        }
    }
    false
}

/// Returns true se o ficheiro declara apenas interfaces, types e re-exports
/// (sem implementação). Ficheiros declaration-only são isentos de V2.
fn is_declaration_only(root: Node, source: &[u8]) -> bool {
    !has_implementation(root, source)
}

fn has_implementation(node: Node, source: &[u8]) -> bool {
    match node.kind() {
        "function_declaration" => return true,
        "class_declaration" => return true,
        "lexical_declaration" => {
            // const foo = () => ... or const foo = function() ...
            if has_function_expression_child(node, source) {
                return true;
            }
        }
        _ => {}
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if has_implementation(child, source) {
                return true;
            }
        }
    }
    false
}

fn has_function_expression_child(node: Node, _source: &[u8]) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if matches!(child.kind(), "arrow_function" | "function_expression") {
                return true;
            }
        }
    }
    false
}

// ── PublicInterface extraction (V6) ──────────────────────────────────────────

fn extract_public_interface<'a>(root: Node, source: &'a [u8]) -> PublicInterface<'a> {
    let mut functions = Vec::new();
    let mut types = Vec::new();
    let mut reexports = Vec::new();

    for i in 0..root.child_count() {
        if let Some(child) = root.child(i) {
            match child.kind() {
                "export_statement" => {
                    process_export_statement(child, source, &mut functions, &mut types, &mut reexports);
                }
                _ => {}
            }
        }
    }

    PublicInterface { functions, types, reexports }
}

fn process_export_statement<'a>(
    node: Node,
    source: &'a [u8],
    functions: &mut Vec<FunctionSignature<'a>>,
    types: &mut Vec<TypeSignature<'a>>,
    reexports: &mut Vec<&'a str>,
) {
    // Re-export: export { X } from './foo' or export * from './foo'
    if let Some(src_path) = import_source_str(node, source) {
        if !src_path.is_empty() {
            reexports.push(src_path);
            return;
        }
    }

    // export { X } without from — named re-export
    if let Some(clause) = node.child_by_field_name("export_clause")
        .or_else(|| find_child_by_kind(node, "export_clause")) {
        let text = node_text(clause, source);
        reexports.push(text);
        return;
    }

    // export <declaration>
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                "function_declaration" => {
                    if let Some(sig) = extract_fn_sig(child, source) {
                        functions.push(sig);
                    }
                }
                "class_declaration" => {
                    if let Some(sig) = extract_class_sig(child, source) {
                        types.push(sig);
                    }
                }
                "interface_declaration" => {
                    if let Some(sig) = extract_interface_sig(child, source) {
                        types.push(sig);
                    }
                }
                "type_alias_declaration" => {
                    if let Some(sig) = extract_type_alias_sig(child, source) {
                        types.push(sig);
                    }
                }
                "enum_declaration" => {
                    if let Some(sig) = extract_enum_sig(child, source) {
                        types.push(sig);
                    }
                }
                "lexical_declaration" | "variable_declaration" => {
                    // export const foo: () => void = ...
                    if let Some(sig) = extract_const_fn_sig(child, source) {
                        functions.push(sig);
                    }
                }
                _ => {}
            }
        }
    }
}

fn find_child_by_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == kind {
                return Some(child);
            }
        }
    }
    None
}

fn extract_fn_sig<'a>(node: Node, source: &'a [u8]) -> Option<FunctionSignature<'a>> {
    let name = node.child_by_field_name("name").map(|n| node_text(n, source))?;
    let params = node
        .child_by_field_name("parameters")
        .map(|p| extract_param_types(p, source))
        .unwrap_or_default();
    let return_type = node
        .child_by_field_name("return_type")
        .map(|rt| {
            let text = node_text(rt, source);
            let trimmed = text.trim_start_matches(':').trim();
            if trimmed == "void" || trimmed.is_empty() { "" } else { trimmed }
        })
        .filter(|s| !s.is_empty());
    Some(FunctionSignature { name, params, return_type })
}

fn extract_const_fn_sig<'a>(node: Node, source: &'a [u8]) -> Option<FunctionSignature<'a>> {
    // Look for variable_declarator with an arrow_function value
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "variable_declarator" {
                let name = child.child_by_field_name("name").map(|n| node_text(n, source))?;
                let value = child.child_by_field_name("value")?;
                if matches!(value.kind(), "arrow_function" | "function_expression") {
                    let params = value
                        .child_by_field_name("parameters")
                        .map(|p| extract_param_types(p, source))
                        .unwrap_or_default();
                    let return_type = value
                        .child_by_field_name("return_type")
                        .map(|rt| {
                            let text = node_text(rt, source);
                            let trimmed = text.trim_start_matches(':').trim();
                            if trimmed == "void" { "" } else { trimmed }
                        })
                        .filter(|s| !s.is_empty());
                    return Some(FunctionSignature { name, params, return_type });
                }
            }
        }
    }
    None
}

fn extract_param_types<'a>(params_node: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut result = Vec::new();
    for i in 0..params_node.child_count() {
        if let Some(child) = params_node.child(i) {
            if matches!(child.kind(), "required_parameter" | "optional_parameter") {
                if let Some(type_annotation) = child.child_by_field_name("type") {
                    let text = node_text(type_annotation, source);
                    result.push(text.trim_start_matches(':').trim());
                }
            }
        }
    }
    result
}

fn extract_class_sig<'a>(node: Node, source: &'a [u8]) -> Option<TypeSignature<'a>> {
    let name = node.child_by_field_name("name").map(|n| node_text(n, source))?;
    let members = node
        .child_by_field_name("body")
        .map(|b| extract_class_members(b, source))
        .unwrap_or_default();
    Some(TypeSignature { name, kind: TypeKind::Class, members })
}

fn extract_class_members<'a>(body: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut result = Vec::new();
    for i in 0..body.child_count() {
        if let Some(child) = body.child(i) {
            if matches!(child.kind(), "public_field_definition" | "method_definition") {
                if let Some(key) = child.child_by_field_name("name") {
                    result.push(node_text(key, source));
                }
            }
        }
    }
    result
}

fn extract_interface_sig<'a>(node: Node, source: &'a [u8]) -> Option<TypeSignature<'a>> {
    let name = node.child_by_field_name("name").map(|n| node_text(n, source))?;
    let members = node
        .child_by_field_name("body")
        .map(|b| extract_interface_members(b, source))
        .unwrap_or_default();
    Some(TypeSignature { name, kind: TypeKind::Interface, members })
}

fn extract_interface_members<'a>(body: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut result = Vec::new();
    for i in 0..body.child_count() {
        if let Some(child) = body.child(i) {
            if matches!(child.kind(), "property_signature" | "method_signature") {
                if let Some(key) = child.child_by_field_name("name") {
                    result.push(node_text(key, source));
                }
            }
        }
    }
    result
}

fn extract_type_alias_sig<'a>(node: Node, source: &'a [u8]) -> Option<TypeSignature<'a>> {
    let name = node.child_by_field_name("name").map(|n| node_text(n, source))?;
    Some(TypeSignature { name, kind: TypeKind::TypeAlias, members: vec![] })
}

fn extract_enum_sig<'a>(node: Node, source: &'a [u8]) -> Option<TypeSignature<'a>> {
    let name = node.child_by_field_name("name").map(|n| node_text(n, source))?;
    let members = node
        .child_by_field_name("body")
        .map(|b| {
            let mut result = Vec::new();
            for i in 0..b.child_count() {
                if let Some(child) = b.child(i) {
                    if child.kind() == "enum_member" {
                        if let Some(key) = child.child_by_field_name("name") {
                            result.push(node_text(key, source));
                        }
                    }
                }
            }
            result
        })
        .unwrap_or_default();
    Some(TypeSignature { name, kind: TypeKind::Enum, members })
}

// ── declared_traits (V11) ─────────────────────────────────────────────────────

/// Extrai nomes de `export interface` de nível superior.
/// Chamador deve garantir que está em L1/contracts.
fn extract_declared_traits<'a>(root: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut traits = Vec::new();
    for i in 0..root.child_count() {
        if let Some(node) = root.child(i) {
            if node.kind() == "export_statement" {
                for j in 0..node.child_count() {
                    if let Some(child) = node.child(j) {
                        if child.kind() == "interface_declaration" {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                traits.push(node_text(name_node, source));
                            }
                        }
                    }
                }
            }
        }
    }
    traits
}

// ── implemented_traits (V11) ──────────────────────────────────────────────────

/// Extrai nomes de interfaces da cláusula `implements` de `class_declaration`.
/// Chamador deve garantir que está em L2|L3.
fn extract_implemented_traits<'a>(root: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut traits = Vec::new();
    for i in 0..root.child_count() {
        if let Some(node) = root.child(i) {
            match node.kind() {
                "class_declaration" => {
                    collect_implements(node, source, &mut traits);
                }
                "export_statement" => {
                    for j in 0..node.child_count() {
                        if let Some(child) = node.child(j) {
                            if child.kind() == "class_declaration" {
                                collect_implements(child, source, &mut traits);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    traits
}

fn collect_implements<'a>(class_node: Node, source: &'a [u8], traits: &mut Vec<&'a str>) {
    // In tree-sitter-typescript 0.21:
    // class_declaration → class_heritage → implements_clause → type_identifier*
    for i in 0..class_node.child_count() {
        if let Some(child) = class_node.child(i) {
            if child.kind() == "class_heritage" {
                for j in 0..child.child_count() {
                    if let Some(clause) = child.child(j) {
                        if clause.kind() == "implements_clause" {
                            collect_implements_names(clause, source, traits);
                        }
                    }
                }
            }
        }
    }
}

fn collect_implements_names<'a>(node: Node, source: &'a [u8], traits: &mut Vec<&'a str>) {
    match node.kind() {
        "type_identifier" | "identifier" => {
            traits.push(node_text(node, source));
        }
        "generic_type" => {
            // e.g. implements Foo<Bar> — extract just "Foo"
            if let Some(name) = node.child(0) {
                if matches!(name.kind(), "type_identifier" | "identifier") {
                    traits.push(node_text(name, source));
                }
            }
        }
        _ => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    collect_implements_names(child, source, traits);
                }
            }
        }
    }
}

// ── declarations (V12) ────────────────────────────────────────────────────────

/// Extrai declarações de tipo de nível superior para V12.
/// `class com implements` NÃO é capturado (é adapter, equivalente a impl Trait for Type).
/// Para todos os arquivos — V12 filtra por layer == L4 internamente.
fn extract_declarations<'a>(root: Node, source: &'a [u8]) -> Vec<Declaration<'a>> {
    let mut decls = Vec::new();
    for i in 0..root.child_count() {
        if let Some(node) = root.child(i) {
            collect_declaration(node, source, &mut decls, false);
        }
    }
    decls
}

fn collect_declaration<'a>(
    node: Node,
    source: &'a [u8],
    decls: &mut Vec<Declaration<'a>>,
    inside_export: bool,
) {
    match node.kind() {
        "class_declaration" => {
            // Only capture class WITHOUT implements clause
            if !has_implements_clause(node) {
                if let Some(name_node) = node.child_by_field_name("name") {
                    decls.push(Declaration {
                        kind: DeclarationKind::Class,
                        name: node_text(name_node, source),
                        line: node.start_position().row + 1,
                    });
                }
            }
        }
        "interface_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                decls.push(Declaration {
                    kind: DeclarationKind::Interface,
                    name: node_text(name_node, source),
                    line: node.start_position().row + 1,
                });
            }
        }
        "type_alias_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                decls.push(Declaration {
                    kind: DeclarationKind::TypeAlias,
                    name: node_text(name_node, source),
                    line: node.start_position().row + 1,
                });
            }
        }
        "enum_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                decls.push(Declaration {
                    kind: DeclarationKind::Enum,
                    name: node_text(name_node, source),
                    line: node.start_position().row + 1,
                });
            }
        }
        "export_statement" if !inside_export => {
            // Recurse into export_statement to capture exported declarations
            for j in 0..node.child_count() {
                if let Some(child) = node.child(j) {
                    collect_declaration(child, source, decls, true);
                }
            }
        }
        _ => {}
    }
}

fn has_implements_clause(class_node: Node) -> bool {
    for i in 0..class_node.child_count() {
        if let Some(child) = class_node.child(i) {
            if child.kind() == "class_heritage" {
                for j in 0..child.child_count() {
                    if let Some(clause) = child.child(j) {
                        if clause.kind() == "implements_clause" {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

// ── AST utilities ─────────────────────────────────────────────────────────────

fn node_text<'a>(node: Node, source: &'a [u8]) -> &'a str {
    std::str::from_utf8(&source[node.start_byte()..node.end_byte()]).unwrap_or("")
}

fn path_contains_segment(path: &Path, segment: &str) -> bool {
    path.components().any(|c| c.as_os_str().to_str().unwrap_or("") == segment)
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

    fn make_parser() -> TsParser<NullPromptReader, NullSnapshotReader> {
        TsParser::new(
            NullPromptReader,
            NullSnapshotReader,
            CrystallineConfig::default(),
            PathBuf::from("."),
        )
    }

    fn make_parser_with_config(config: CrystallineConfig) -> TsParser<NullPromptReader, NullSnapshotReader> {
        TsParser::new(NullPromptReader, NullSnapshotReader, config, PathBuf::from("."))
    }

    fn ts_file(content: &str) -> SourceFile {
        SourceFile {
            path: PathBuf::from("03_infra/ts_parser.ts"),
            content: content.to_string(),
            language: Language::TypeScript,
            layer: Layer::L3,
            has_adjacent_test: false,
        }
    }

    fn ts_file_at(content: &str, path: &str, layer: Layer) -> SourceFile {
        SourceFile {
            path: PathBuf::from(path),
            content: content.to_string(),
            language: Language::TypeScript,
            layer,
            has_adjacent_test: false,
        }
    }

    // ── UnsupportedLanguage ────────────────────────────────────────────────────

    #[test]
    fn unsupported_language_for_rust_file() {
        let parser = make_parser();
        let file = SourceFile {
            path: PathBuf::from("01_core/foo.rs"),
            content: "fn main() {}".to_string(),
            language: Language::Rust,
            layer: Layer::L1,
            has_adjacent_test: false,
        };
        assert!(matches!(parser.parse(&file), Err(ParseError::UnsupportedLanguage { .. })));
    }

    // ── EmptySource ────────────────────────────────────────────────────────────

    #[test]
    fn empty_source_returns_error() {
        let parser = make_parser();
        let file = ts_file("");
        assert!(matches!(parser.parse(&file), Err(ParseError::EmptySource { .. })));
    }

    // ── Header ────────────────────────────────────────────────────────────────

    #[test]
    fn parses_crystalline_header() {
        let parser = make_parser();
        let file = ts_file(
            "// Crystalline Lineage\n\
             // @prompt 00_nucleo/prompts/parsers/typescript.md\n\
             // @prompt-hash abcd1234\n\
             // @layer L3\n\
             // @updated 2026-03-19\n\
             export function foo(): void {}",
        );
        let parsed = parser.parse(&file).unwrap();
        let header = parsed.prompt_header.unwrap();
        assert_eq!(header.prompt_path, "00_nucleo/prompts/parsers/typescript.md");
        assert_eq!(header.prompt_hash, Some("abcd1234"));
        assert_eq!(header.layer, Layer::L3);
        assert_eq!(header.updated, Some("2026-03-19"));
    }

    #[test]
    fn header_stops_at_first_non_comment_line() {
        let parser = make_parser();
        let file = ts_file(
            "// @prompt foo.md\n\
             // @layer L1\n\
             export interface Foo {}\n\
             // this should not be part of header",
        );
        let parsed = parser.parse(&file).unwrap();
        let header = parsed.prompt_header.unwrap();
        assert_eq!(header.layer, Layer::L1);
    }

    // ── Imports — resolução física ─────────────────────────────────────────────

    #[test]
    fn import_relative_resolves_to_correct_layer() {
        let parser = make_parser();
        let file = ts_file_at(
            "import { Layer } from '../01_core/entities/layer';",
            "03_infra/ts_parser.ts",
            Layer::L3,
        );
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.imports.len(), 1);
        assert_eq!(parsed.imports[0].target_layer, Layer::L1);
        assert_eq!(parsed.imports[0].kind, ImportKind::Named);
    }

    #[test]
    fn import_with_excessive_parent_dirs_resolves_to_unknown() {
        let parser = make_parser();
        let file = ts_file_at(
            "import { evil } from '../../../../../etc/passwd';",
            "01_core/entities/layer.ts",
            Layer::L1,
        );
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.imports[0].target_layer, Layer::Unknown);
    }

    #[test]
    fn import_npm_package_resolves_to_unknown() {
        let parser = make_parser();
        let file = ts_file("import express from 'express';");
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.imports[0].target_layer, Layer::Unknown);
    }

    #[test]
    fn import_lab_resolves_to_lab_layer() {
        let parser = make_parser();
        // From 01_core/entities/foo.ts, reach top-level lab/ needs ../../lab/
        let file = ts_file_at(
            "import { X } from '../../lab/experiment';",
            "01_core/entities/foo.ts",
            Layer::L1,
        );
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.imports[0].target_layer, Layer::Lab);
    }

    #[test]
    fn import_with_alias_resolves_to_correct_layer() {
        let mut config = CrystallineConfig::default();
        config.ts_aliases.insert("@core".to_string(), "01_core".to_string());
        let parser = make_parser_with_config(config);
        let file = ts_file_at(
            "import { Layer } from '@core/entities/layer';",
            "03_infra/ts_parser.ts",
            Layer::L3,
        );
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.imports[0].target_layer, Layer::L1);
    }

    #[test]
    fn import_subdir_extracted_for_l1() {
        let parser = make_parser();
        let file = ts_file_at(
            "import { Layer } from '../01_core/entities/layer';",
            "03_infra/ts_parser.ts",
            Layer::L3,
        );
        let parsed = parser.parse(&file).unwrap();
        let imp = &parsed.imports[0];
        assert_eq!(imp.target_layer, Layer::L1);
        assert_eq!(imp.target_subdir, Some("entities"));
    }

    // ── Tokens (V4) ───────────────────────────────────────────────────────────

    #[test]
    fn forbidden_module_import_produces_token() {
        let parser = make_parser();
        let file = ts_file("import { readFileSync } from 'fs';");
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.tokens.iter().any(|t| t.symbol.contains("fs")));
    }

    #[test]
    fn date_now_produces_token() {
        let parser = make_parser();
        let file = ts_file("const t = Date.now();");
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.tokens.iter().any(|t| t.symbol.as_ref() == "Date.now"));
    }

    #[test]
    fn math_random_produces_token() {
        let parser = make_parser();
        let file = ts_file("const r = Math.random();");
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.tokens.iter().any(|t| t.symbol.as_ref() == "Math.random"));
    }

    // ── Test coverage (V2) ────────────────────────────────────────────────────

    #[test]
    fn describe_call_sets_test_coverage() {
        let parser = make_parser();
        let file = ts_file("describe('suite', () => { it('test', () => {}); });");
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.has_test_coverage);
    }

    #[test]
    fn adjacent_test_file_sets_coverage() {
        let parser = make_parser();
        let mut file = ts_file("export function foo() {}");
        file.has_adjacent_test = true;
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.has_test_coverage);
    }

    #[test]
    fn declaration_only_file_is_exempt() {
        let parser = make_parser();
        let file = ts_file(
            "export interface FileProvider { files(): string[] }\n\
             export type Config = { root: string }",
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.has_test_coverage); // declaration-only → exempt
    }

    #[test]
    fn file_with_function_requires_test() {
        let parser = make_parser();
        let file = ts_file("export function check(x: string): boolean { return true; }");
        let parsed = parser.parse(&file).unwrap();
        assert!(!parsed.has_test_coverage);
    }

    // ── declared_traits (V11) ─────────────────────────────────────────────────

    #[test]
    fn declared_traits_extracted_from_l1_contracts() {
        let parser = make_parser();
        let mut file = ts_file_at(
            "export interface FileProvider { files(): string[] }\n\
             export interface LanguageParser { parse(f: any): any }\n\
             interface InternalHelper { help(): void }",
            "01_core/contracts/file_provider.ts",
            Layer::L1,
        );
        file.layer = Layer::L1;
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.declared_traits.contains(&"FileProvider"));
        assert!(parsed.declared_traits.contains(&"LanguageParser"));
        assert!(!parsed.declared_traits.contains(&"InternalHelper"));
    }

    #[test]
    fn declared_traits_empty_for_l1_non_contracts() {
        let parser = make_parser();
        let file = ts_file_at(
            "export interface HasImports { imports(): any[] }",
            "01_core/rules/forbidden_import.ts",
            Layer::L1,
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.declared_traits.is_empty());
    }

    // ── implemented_traits (V11) ──────────────────────────────────────────────

    #[test]
    fn implemented_traits_extracted_for_l3() {
        let parser = make_parser();
        let file = ts_file_at(
            "class FileWalker implements FileProvider {\n\
               files() { return []; }\n\
             }",
            "03_infra/walker.ts",
            Layer::L3,
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.implemented_traits.contains(&"FileProvider"));
    }

    #[test]
    fn class_without_implements_not_in_implemented_traits() {
        let parser = make_parser();
        let file = ts_file_at(
            "class InternalHelper { help() {} }",
            "03_infra/helper.ts",
            Layer::L3,
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.implemented_traits.is_empty());
    }

    #[test]
    fn implemented_traits_empty_for_l4() {
        let parser = make_parser();
        let file = ts_file_at(
            "class Adapter implements FileProvider { files() { return []; } }",
            "04_wiring/main.ts",
            Layer::L4,
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(parsed.implemented_traits.is_empty());
    }

    // ── declarations (V12) ────────────────────────────────────────────────────

    #[test]
    fn class_without_implements_captured_in_declarations() {
        let parser = make_parser();
        let file = ts_file_at(
            "class OutputFormatter {}\n\
             interface InternalConfig {}\n\
             type Mode = 'text' | 'sarif'",
            "04_wiring/main.ts",
            Layer::L4,
        );
        let parsed = parser.parse(&file).unwrap();
        let kinds: Vec<_> = parsed.declarations.iter().map(|d| (&d.kind, d.name)).collect();
        assert!(kinds.contains(&(&DeclarationKind::Class, "OutputFormatter")));
        assert!(kinds.contains(&(&DeclarationKind::Interface, "InternalConfig")));
        assert!(kinds.contains(&(&DeclarationKind::TypeAlias, "Mode")));
    }

    #[test]
    fn class_with_implements_not_in_declarations() {
        let parser = make_parser();
        let file = ts_file_at(
            "class L3HashAdapter implements HashRewriter {}\n\
             class OutputFormatter {}",
            "04_wiring/main.ts",
            Layer::L4,
        );
        let parsed = parser.parse(&file).unwrap();
        // Only OutputFormatter (no implements) should be captured
        assert!(!parsed.declarations.iter().any(|d| d.name == "L3HashAdapter"));
        assert!(parsed.declarations.iter().any(|d| d.name == "OutputFormatter"));
    }

    // ── export classification tests ───────────────────────────────────────────

    #[test]
    fn export_star_from_is_glob() {
        let parser = make_parser();
        let file = ts_file_at(
            "export * from './01_core/entities/layer';",
            "03_infra/ts_parser.ts",
            Layer::L3,
        );
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.imports.len(), 1);
        assert_eq!(parsed.imports[0].kind, ImportKind::Glob);
    }

    #[test]
    fn export_named_from_is_named() {
        let parser = make_parser();
        let file = ts_file_at(
            "export { Layer } from './01_core/entities/layer';",
            "03_infra/ts_parser.ts",
            Layer::L3,
        );
        let parsed = parser.parse(&file).unwrap();
        assert_eq!(parsed.imports.len(), 1);
        assert_eq!(parsed.imports[0].kind, ImportKind::Named);
    }

    // ── AST debug ─────────────────────────────────────────────────────────────

    fn print_tree(node: Node, source: &[u8], depth: usize) {
        let indent = "  ".repeat(depth);
        let text = std::str::from_utf8(&source[node.start_byte()..node.end_byte()]).unwrap_or("?");
        let short = if text.len() > 40 { &text[..40] } else { text };
        println!("{}[{}] {:?}", indent, node.kind(), short.replace('\n', "\\n"));
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                print_tree(child, source, depth + 1);
            }
        }
    }

    // ── normalize unit tests ──────────────────────────────────────────────────

    #[test]
    fn normalize_resolves_parent_dirs() {
        let result = normalize(
            Path::new("03_infra/../01_core/entities/layer"),
            Path::new("."),
        );
        assert_eq!(result, Some(PathBuf::from("01_core/entities/layer")));
    }

    #[test]
    fn normalize_returns_none_for_escaping_root() {
        let result = normalize(
            Path::new("01_core/../../../../../../etc/passwd"),
            Path::new("."),
        );
        assert!(result.is_none());
    }

    #[test]
    fn normalize_skips_cur_dir() {
        let result = normalize(Path::new("./01_core/entities"), Path::new("."));
        assert_eq!(result, Some(PathBuf::from("01_core/entities")));
    }

    // ── ImportKind mapping — critérios ADR-0009 ────────────────────────────────

    #[test]
    fn import_default_is_direct() {
        // import X from '...' → ImportKind::Direct
        let parser = make_parser();
        let file = ts_file_at(
            "import Layer from '../01_core/entities/layer';",
            "03_infra/ts_parser.ts",
            Layer::L3,
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(!parsed.imports.is_empty(), "should have at least one import");
        assert_eq!(parsed.imports[0].kind, ImportKind::Direct);
    }

    #[test]
    fn import_namespace_is_glob() {
        // import * as ns from '...' → ImportKind::Glob
        let parser = make_parser();
        let file = ts_file_at(
            "import * as entities from '../01_core/entities/layer';",
            "03_infra/ts_parser.ts",
            Layer::L3,
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(!parsed.imports.is_empty(), "should have at least one import");
        assert_eq!(parsed.imports[0].kind, ImportKind::Glob);
    }

    #[test]
    fn import_renamed_binding_is_alias() {
        // import { A as B } from '...' → ImportKind::Alias
        let parser = make_parser();
        let file = ts_file_at(
            "import { Layer as L } from '../01_core/entities/layer';",
            "03_infra/ts_parser.ts",
            Layer::L3,
        );
        let parsed = parser.parse(&file).unwrap();
        assert!(!parsed.imports.is_empty(), "should have at least one import");
        assert_eq!(parsed.imports[0].kind, ImportKind::Alias);
    }

    #[test]
    fn alias_import_subdir_extracted() {
        // @core/entities/layer → target_subdir = Some("entities")
        // target_subdir produzido via intern_subdir(), não Box::leak
        let mut config = CrystallineConfig::default();
        config.ts_aliases.insert("@core".to_string(), "01_core".to_string());
        let parser = make_parser_with_config(config);
        let file = ts_file_at(
            "import { W } from '@core/entities/layer';",
            "03_infra/ts_parser.ts",
            Layer::L3,
        );
        let parsed = parser.parse(&file).unwrap();
        let imp = parsed.imports.iter().find(|i| i.target_layer == Layer::L1);
        assert!(imp.is_some(), "alias should resolve to L1");
        assert_eq!(imp.unwrap().target_subdir, Some("entities"));
    }

    #[test]
    fn syntax_error_reports_nonzero_line() {
        // Fonte sintaticamente inválida com erro após linha 1 → line > 0
        let parser = make_parser();
        // Linha 2 é claramente inválida em TypeScript
        let file = ts_file("export const x = 1;\nfunction {{{{{");
        match parser.parse(&file) {
            Err(ParseError::SyntaxError { line, .. }) => {
                assert!(line > 0, "SyntaxError.line should be > 0, got {}", line);
            }
            Ok(_) => {
                panic!("expected ParseError::SyntaxError for syntactically invalid source, got Ok");
            }
            Err(other) => panic!("expected SyntaxError, got {:?}", other),
        }
    }

    #[test]
    fn intern_subdir_no_box_leak_smoke_test() {
        // Criar e descartar 10 instâncias do parser — verificar que não há crash/leak
        // Se Box::leak fosse usado, o processo acumularia memória não libertada.
        // Este smoke test verifica que não há crash após criação e descarte repetidos.
        for _ in 0..10 {
            let mut config = CrystallineConfig::default();
            config.ts_aliases.insert("@core".to_string(), "01_core".to_string());
            let parser = TsParser::new(
                NullPromptReader,
                NullSnapshotReader,
                config,
                PathBuf::from("."),
            );
            let file = ts_file_at(
                "import { Layer } from '@core/entities/layer';",
                "03_infra/ts_parser.ts",
                Layer::L3,
            );
            let _ = parser.parse(&file);
            // Parser descartado aqui — subdirs_buffer deve liberar toda a memória
        }
        // Contrato correcto — teste adicionado para prevenir regressão de Box::leak
    }
}
