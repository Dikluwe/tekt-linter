//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/parsers/python.md
//! @prompt-hash eb784a66
//! @layer L3
//! @updated 2026-03-20

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

use tree_sitter::{Node, Parser as PyParserEngine};

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

// ── PyParser ──────────────────────────────────────────────────────────────────

/// Parser de Python implementando `LanguageParser`.
/// Resolução de camadas: física (ADR-0009) via PyLayerResolver.
/// Zero-Copy: retorna ParsedFile<'a> com referências ao buffer de `SourceFile`.
pub struct PyParser<R: PromptReader, S: PromptSnapshotReader> {
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
    /// Raiz do projecto — usada pelo PyLayerResolver para resolve_file_layer.
    pub project_root: PathBuf,
    subdirs_buffer: Mutex<Vec<Box<str>>>,
}

impl<R: PromptReader, S: PromptSnapshotReader> PyParser<R, S> {
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

impl<R: PromptReader, S: PromptSnapshotReader> LanguageParser for PyParser<R, S> {
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        if file.content.is_empty() {
            return Err(ParseError::EmptySource { path: file.path.clone() });
        }

        if file.language != Language::Python {
            return Err(ParseError::UnsupportedLanguage {
                path: file.path.clone(),
                language: file.language.clone(),
            });
        }

        let mut engine = PyParserEngine::new();
        engine.set_language(&tree_sitter_python::language()).map_err(|_| {
            ParseError::SyntaxError {
                path: file.path.clone(),
                line: 0,
                column: 0,
                message: "Failed to load Python grammar".to_string(),
            }
        })?;

        let tree = engine.parse(file.content.as_bytes(), None).ok_or_else(|| {
            ParseError::SyntaxError {
                path: file.path.clone(),
                line: 0,
                column: 0,
                message: "Parser returned None — possible timeout".to_string(),
            }
        })?;

        let root = tree.root_node();

        if root.has_error() {
            let (line, column) = find_first_error_pos(root);
            return Err(ParseError::SyntaxError {
                path: file.path.clone(),
                line,
                column,
                message: "Syntax error detected in Python AST".to_string(),
            });
        }

        let source = file.content.as_bytes();

        // 1. Header (bloco # no topo: @prompt, @prompt-hash, @layer, @updated)
        let mut prompt_header = extract_header(&file.content);
        let prompt_file_exists = prompt_header
            .as_ref()
            .map(|h| self.prompt_reader.exists(h.prompt_path))
            .unwrap_or(false);
        if let Some(ref mut header) = prompt_header {
            header.current_hash = self.prompt_reader.read_hash(header.prompt_path);
        }

        // 2. Imports + import_name_map (PyLayerResolver 4 passos + resolve_py_subdir)
        let intern: &dyn Fn(String) -> &'static str = &|s| self.intern_subdir(s);
        let (imports, import_name_map) =
            extract_imports(root, source, file.path.as_path(), &self.project_root, &self.config, intern);

        // 3. Tokens — imports proibidos + call nodes (sem Motor de Duas Fases)
        let tokens = extract_tokens(root, source, &imports);

        // 4. Test coverage — call pytest/unittest + adjacência + declaration-only
        let has_test_ast = has_test_calls(root, source);
        let is_decl_only = is_declaration_only(root, source);
        let has_test_coverage = has_test_ast || file.has_adjacent_test || is_decl_only;

        // 5. PublicInterface + prompt_snapshot (V6)
        let public_interface = extract_public_interface(root, source);
        let prompt_snapshot = prompt_header
            .as_ref()
            .and_then(|h| self.snapshot_reader.read_snapshot(h.prompt_path));

        // 6. declared_traits — apenas L1/contracts, apenas class com Protocol/ABC (V11)
        let declared_traits = if file.layer == Layer::L1
            && path_contains_segment(file.path.as_path(), "contracts")
        {
            extract_declared_traits(root, source)
        } else {
            vec![]
        };

        // 7. implemented_traits — apenas L2|L3, bases de L1/contracts/ (V11)
        let implemented_traits = if matches!(file.layer, Layer::L2 | Layer::L3) {
            extract_implemented_traits(root, source, &import_name_map)
        } else {
            vec![]
        };

        // 8. declarations — class sem Protocol/ABC/contracts (V12)
        let declarations = extract_declarations(root, source, &import_name_map);

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

/// Extrai o header cristalino de comentários `#` no topo do ficheiro.
/// O bloco termina na primeira linha que não começa com `#`.
fn extract_header<'a>(source: &'a str) -> Option<PromptHeader<'a>> {
    let mut prompt_path: Option<&'a str> = None;
    let mut prompt_hash: Option<&'a str> = None;
    let mut layer: Option<Layer> = None;
    let mut updated: Option<&'a str> = None;

    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('#') {
            break;
        }
        let content = trimmed.trim_start_matches('#').trim();

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

// ── PyLayerResolver — 4 passos (ADR-0009) ─────────────────────────────────────

/// Normalização algébrica de paths sem bater no disco.
/// `None` = tentativa de escapar da raiz ou resultado fora do projecto.
fn normalize(path: &Path, project_root: &Path) -> Option<PathBuf> {
    let mut components: Vec<Component> = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                if components.is_empty() {
                    return None; // tentativa de sair da raiz
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
    if project_root != Path::new(".") && !project_root.as_os_str().is_empty() {
        if !result.starts_with(project_root) {
            return None;
        }
    }
    Some(result)
}

/// Resolve o módulo Python de um `import_from_statement` relativo.
/// `prefix_dots` = número de pontos do `import_prefix` (1 = ".", 2 = "..", etc.)
/// `module_dotted` = texto do `dotted_name` filho (pode ser "" se não presente)
/// Retorna o path relativo ao directório do ficheiro que deve ser joined + normalized.
fn relative_module_to_path(prefix_dots: usize, module_dotted: &str) -> String {
    // n_dots=1 (.) → permanecer no dir actual
    // n_dots=2 (..) → subir 1 nível
    // n_dots=3 (...) → subir 2 níveis
    let ups = if prefix_dots > 1 {
        "../".repeat(prefix_dots - 1)
    } else {
        String::new()
    };
    let module_path = module_dotted.replace('.', "/");
    if module_path.is_empty() {
        if prefix_dots == 1 {
            ".".to_string()
        } else {
            // e.g. ".." → "../"
            ups.trim_end_matches('/').to_string()
        }
    } else {
        format!("{}{}", ups, module_path)
    }
}

/// Resolve o target_layer de um import Python relativo.
fn resolve_relative_layer(
    rel_path: &str,
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
) -> Layer {
    let base = file_path.parent().unwrap_or(Path::new("."));
    let joined = base.join(rel_path);
    match normalize(&joined, project_root) {
        Some(normalized) => resolve_file_layer(&normalized, project_root, config),
        None => Layer::Unknown,
    }
}

/// Resolve o target_layer de um módulo absoluto (alias ou externo).
fn resolve_absolute_layer(
    module: &str,
    project_root: &Path,
    config: &CrystallineConfig,
) -> Layer {
    // Passo 1 — verificar alias
    if let Some((alias_key, alias_val)) = config
        .py_aliases
        .iter()
        .find(|(k, _)| module == k.as_str() || module.starts_with(&format!("{}.", k)))
    {
        // Passo 2 — resolução de alias
        let after_alias = if module == alias_key.as_str() {
            String::new()
        } else {
            // e.g. module = "core.contracts", alias_key = "core"
            let suffix = &module[alias_key.len() + 1..]; // skip the dot
            suffix.replace('.', "/")
        };
        let resolved = if after_alias.is_empty() {
            alias_val.clone()
        } else {
            format!("{}/{}", alias_val, after_alias)
        };
        let joined = project_root.join(&resolved);
        return match normalize(&joined, project_root) {
            Some(normalized) => resolve_file_layer(&normalized, project_root, config),
            None => Layer::Unknown,
        };
    }
    // Passo 1 — package externo → Layer::Unknown directamente
    Layer::Unknown
}

/// Extrai o subdir de destino de um caminho normalizado para V9.
fn resolve_py_subdir(
    normalized: &Path,
    target_layer: &Layer,
    project_root: &Path,
    config: &CrystallineConfig,
    intern: &dyn Fn(String) -> &'static str,
) -> Option<&'static str> {
    if *target_layer != Layer::L1 {
        return None;
    }
    let layer_dir = config.layers.get("L1")?;
    let base_l1 = project_root.join(layer_dir);
    let relative = normalized
        .strip_prefix(&base_l1)
        .or_else(|_| normalized.strip_prefix(layer_dir.as_str()))
        .ok()?;
    let subdir = relative.components().next().and_then(|c| c.as_os_str().to_str())?;
    Some(intern(subdir.to_string()))
}

/// Calcula o subdir para um import relativo (usa normalização + resolve_py_subdir).
fn resolve_relative_subdir(
    rel_path: &str,
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
    target_layer: &Layer,
    intern: &dyn Fn(String) -> &'static str,
) -> Option<&'static str> {
    if *target_layer != Layer::L1 {
        return None;
    }
    let base = file_path.parent().unwrap_or(Path::new("."));
    let joined = base.join(rel_path);
    let normalized = normalize(&joined, project_root)?;
    resolve_py_subdir(&normalized, target_layer, project_root, config, intern)
}

/// Calcula o subdir para um módulo absoluto com alias.
fn resolve_absolute_subdir(
    module: &str,
    project_root: &Path,
    config: &CrystallineConfig,
    target_layer: &Layer,
    intern: &dyn Fn(String) -> &'static str,
) -> Option<&'static str> {
    if *target_layer != Layer::L1 {
        return None;
    }
    let alias_key = config
        .py_aliases
        .keys()
        .find(|k| module == k.as_str() || module.starts_with(&format!("{}.", k)))?;
    let alias_val = &config.py_aliases[alias_key];
    let after_alias = if module == alias_key.as_str() {
        String::new()
    } else {
        let suffix = &module[alias_key.len() + 1..];
        suffix.replace('.', "/")
    };
    let resolved = if after_alias.is_empty() {
        alias_val.clone()
    } else {
        format!("{}/{}", alias_val, after_alias)
    };
    let joined = project_root.join(&resolved);
    let normalized = normalize(&joined, project_root)?;
    resolve_py_subdir(&normalized, target_layer, project_root, config, intern)
}

// ── Import extraction ─────────────────────────────────────────────────────────

/// Extrai imports e constrói o import_name_map interno.
/// `import_name_map` mapeia nome local importado → (Layer, Option<subdir>).
/// Usado para resolver bases de classes em implemented_traits e declarations.
fn extract_imports<'a>(
    root: Node,
    source: &'a [u8],
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
    intern: &dyn Fn(String) -> &'static str,
) -> (Vec<Import<'a>>, HashMap<&'a str, (Layer, Option<&'static str>)>) {
    let mut imports = Vec::new();
    let mut name_map: HashMap<&'a str, (Layer, Option<&'static str>)> = HashMap::new();
    collect_imports(root, source, file_path, project_root, config, &mut imports, &mut name_map, intern);
    (imports, name_map)
}

fn collect_imports<'a>(
    node: Node,
    source: &'a [u8],
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
    imports: &mut Vec<Import<'a>>,
    name_map: &mut HashMap<&'a str, (Layer, Option<&'static str>)>,
    intern: &dyn Fn(String) -> &'static str,
) {
    match node.kind() {
        "import_statement" => {
            process_import_statement(
                node, source, file_path, project_root, config, imports, name_map, intern,
            );
        }
        "import_from_statement" => {
            process_import_from_statement(
                node, source, file_path, project_root, config, imports, name_map, intern,
            );
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_imports(child, source, file_path, project_root, config, imports, name_map, intern);
        }
    }
}

/// Processa `import X` ou `import X as Y`.
fn process_import_statement<'a>(
    node: Node,
    source: &'a [u8],
    _file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
    imports: &mut Vec<Import<'a>>,
    name_map: &mut HashMap<&'a str, (Layer, Option<&'static str>)>,
    intern: &dyn Fn(String) -> &'static str,
) {
    let line = node.start_position().row + 1;
    // Children: "import", then dotted_name or aliased_import (comma-separated)
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                "dotted_name" => {
                    let module = node_text(child, source);
                    let target_layer = resolve_absolute_layer(module, project_root, config);
                    let target_subdir =
                        resolve_absolute_subdir(module, project_root, config, &target_layer, intern);
                    imports.push(Import {
                        path: module,
                        line,
                        kind: ImportKind::Direct,
                        target_layer: target_layer.clone(),
                        target_subdir,
                    });
                    // Last segment as local name: "os.path" → "path", "os" → "os"
                    let local = module.rsplit('.').next().unwrap_or(module);
                    name_map.insert(local, (target_layer, target_subdir));
                }
                "aliased_import" => {
                    // `X as Y` — extract original name and alias
                    let orig = child
                        .child_by_field_name("name")
                        .map(|n| node_text(n, source))
                        .unwrap_or("");
                    let alias = child
                        .child_by_field_name("alias")
                        .map(|n| node_text(n, source))
                        .unwrap_or(orig);
                    if !orig.is_empty() {
                        let target_layer = resolve_absolute_layer(orig, project_root, config);
                        let target_subdir =
                            resolve_absolute_subdir(orig, project_root, config, &target_layer, intern);
                        imports.push(Import {
                            path: orig,
                            line,
                            kind: ImportKind::Alias,
                            target_layer: target_layer.clone(),
                            target_subdir,
                        });
                        if !alias.is_empty() {
                            name_map.insert(alias, (target_layer, target_subdir));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Processa `from X import Y` ou `from .X import Y`.
fn process_import_from_statement<'a>(
    node: Node,
    source: &'a [u8],
    file_path: &Path,
    project_root: &Path,
    config: &CrystallineConfig,
    imports: &mut Vec<Import<'a>>,
    name_map: &mut HashMap<&'a str, (Layer, Option<&'static str>)>,
    intern: &dyn Fn(String) -> &'static str,
) {
    let line = node.start_position().row + 1;

    // Find module_name child (relative_import or dotted_name)
    let module_child = node.child_by_field_name("module_name").or_else(|| {
        // Fallback: second child (after "from")
        (0..node.child_count()).filter_map(|i| node.child(i)).find(|c| {
            matches!(c.kind(), "relative_import" | "dotted_name" | "identifier")
        })
    });

    let Some(mod_node) = module_child else { return };

    let (path_str, target_layer, target_subdir) = if mod_node.kind() == "relative_import" {
        // Relative import: extract prefix dots + optional dotted_name
        let mut n_dots = 0usize;
        let mut dotted_module = "";
        for i in 0..mod_node.child_count() {
            if let Some(c) = mod_node.child(i) {
                match c.kind() {
                    "import_prefix" => {
                        n_dots = node_text(c, source).len();
                    }
                    "dotted_name" => {
                        dotted_module = node_text(c, source);
                    }
                    _ => {}
                }
            }
        }
        if n_dots == 0 {
            n_dots = 1; // fallback
        }
        let rel = relative_module_to_path(n_dots, dotted_module);
        let layer = resolve_relative_layer(&rel, file_path, project_root, config);
        let subdir = resolve_relative_subdir(&rel, file_path, project_root, config, &layer, intern);
        // Store the original relative_import text as path (e.g. ".utils")
        let path_text = node_text(mod_node, source);
        (path_text, layer, subdir)
    } else {
        // Absolute import (dotted_name or identifier)
        let module = node_text(mod_node, source);
        let layer = resolve_absolute_layer(module, project_root, config);
        let subdir = resolve_absolute_subdir(module, project_root, config, &layer, intern);
        (module, layer, subdir)
    };

    // Determine import kind: wildcard vs named subset
    let is_wildcard = (0..node.child_count())
        .filter_map(|i| node.child(i))
        .any(|c| c.kind() == "wildcard_import");
    let kind = if is_wildcard { ImportKind::Glob } else { ImportKind::Named };

    imports.push(Import {
        path: path_str,
        line,
        kind,
        target_layer: target_layer.clone(),
        target_subdir,
    });

    // Extract imported names for import_name_map
    // Names come after the "import" keyword
    let mut past_import_kw = false;
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "import" {
                past_import_kw = true;
                continue;
            }
            if !past_import_kw {
                continue;
            }
            match child.kind() {
                "dotted_name" | "identifier" => {
                    let name = node_text(child, source);
                    // Last segment
                    let local = name.rsplit('.').next().unwrap_or(name);
                    name_map.insert(local, (target_layer.clone(), target_subdir));
                }
                "aliased_import" => {
                    let alias = child
                        .child_by_field_name("alias")
                        .map(|n| node_text(n, source))
                        .unwrap_or_else(|| {
                            child
                                .child_by_field_name("name")
                                .map(|n| {
                                    let t = node_text(n, source);
                                    t.rsplit('.').next().unwrap_or(t)
                                })
                                .unwrap_or("")
                        });
                    if !alias.is_empty() {
                        name_map.insert(alias, (target_layer.clone(), target_subdir));
                    }
                }
                "wildcard_import" => {
                    // `from X import *` — cannot map individual names
                }
                _ => {}
            }
        }
    }
}

// ── Token extraction (V4) ─────────────────────────────────────────────────────

const FORBIDDEN_MODULES: &[&str] = &[
    "os", "os.path", "pathlib", "shutil", "subprocess", "socket",
    "urllib", "http.client", "ftplib", "smtplib",
];

const FORBIDDEN_CALLS: &[&str] = &[
    "open",
    "random.random",
    "time.time",
    "datetime.now",
    "datetime.datetime.now",
];

fn extract_tokens<'a>(
    root: Node,
    source: &'a [u8],
    imports: &[Import<'a>],
) -> Vec<Token<'a>> {
    let mut tokens = Vec::new();

    // Mecanismo 1 — imports de módulos proibidos
    for imp in imports {
        if FORBIDDEN_MODULES.contains(&imp.path) {
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
    if node.kind() == "call" {
        // In tree-sitter-python, `call` has child `function` (identifier or attribute)
        if let Some(func) = node.child_by_field_name("function").or_else(|| node.child(0)) {
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

const TEST_CALL_NAMES: &[&str] = &["unittest", "pytest", "describe", "it", "test", "suite"];

fn has_test_calls(root: Node, source: &[u8]) -> bool {
    // Verificar call expressions de teste em qualquer nível
    if check_test_call_nodes(root, source) {
        return true;
    }
    // Verificar classes unittest apenas no nível de topo
    for i in 0..root.child_count() {
        if let Some(child) = root.child(i) {
            if child.kind() == "class_definition" {
                if is_unittest_class(child, source) {
                    return true;
                }
            }
        }
    }
    false
}

fn is_unittest_class(node: Node, source: &[u8]) -> bool {
    // Condição 1: nome termina em Test ou Tests
    let name_ok = node.child_by_field_name("name")
        .map(|n| {
            let name = node_text(n, source);
            name.ends_with("Test") || name.ends_with("Tests")
        })
        .unwrap_or(false);
    if !name_ok { return false; }

    // Condição 2: herda de TestCase
    node.child_by_field_name("superclasses")
        .map(|bases| node_text(bases, source).contains("TestCase"))
        .unwrap_or(false)
}

fn check_test_call_nodes(node: Node, source: &[u8]) -> bool {
    if node.kind() == "call" {
        if let Some(func) = node.child_by_field_name("function").or_else(|| node.child(0)) {
            let text = node_text(func, source);
            let first_seg = text.split('.').next().unwrap_or(text);
            if TEST_CALL_NAMES.contains(&first_seg) || TEST_CALL_NAMES.contains(&text) {
                return true;
            }
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if check_test_call_nodes(child, source) {
                return true;
            }
        }
    }
    false
}

/// Returns true se o ficheiro declara apenas Protocol/ABC classes, imports e __all__.
/// Ficheiros declaration-only são isentos de V2.
fn is_declaration_only(root: Node, source: &[u8]) -> bool {
    !has_implementation(root, source)
}

fn has_implementation(node: Node, source: &[u8]) -> bool {
    match node.kind() {
        "function_definition" => {
            // Check if body is just `...` (ellipsis) or `pass`
            return !is_trivial_body(node, source);
        }
        "decorated_definition" => {
            // decorated function or class
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() == "function_definition"
                        && !is_trivial_body(child, source)
                    {
                        return true;
                    }
                    if child.kind() == "class_definition"
                        && !is_protocol_abc_class(child, source)
                    {
                        return true;
                    }
                }
            }
            return false;
        }
        "class_definition" => {
            // Protocol/ABC classes are allowed in declaration-only files
            return !is_protocol_abc_class(node, source);
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

/// Returns true if a function body consists only of `...` (ellipsis) or `pass`.
fn is_trivial_body(func_node: Node, _source: &[u8]) -> bool {
    let body = match func_node.child_by_field_name("body") {
        Some(b) => b,
        None => return true,
    };
    // block contains statements; check if all are trivial
    for i in 0..body.child_count() {
        if let Some(stmt) = body.child(i) {
            match stmt.kind() {
                "expression_statement" => {
                    // Check if contains ellipsis or string (docstring)
                    if let Some(inner) = stmt.child(0) {
                        match inner.kind() {
                            "ellipsis" | "string" => {}
                            _ => return false,
                        }
                    }
                }
                "pass_statement" => {}
                "\n" | "comment" => {}
                _ => return false,
            }
        }
    }
    true
}

/// Returns true if a class_definition inherits from Protocol, ABC, or ABCMeta.
fn is_protocol_abc_class(class_node: Node, source: &[u8]) -> bool {
    if let Some(bases) = class_node.child_by_field_name("superclasses") {
        let bases_text = node_text(bases, source);
        // Check for Protocol, ABC, ABCMeta in the base list
        return bases_text.contains("Protocol")
            || bases_text.contains("ABC")
            || bases_text.contains("ABCMeta");
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
                "function_definition" => {
                    if let Some(sig) = extract_fn_sig(child, source) {
                        if !sig.name.starts_with('_') {
                            functions.push(sig);
                        }
                    }
                }
                "decorated_definition" => {
                    for j in 0..child.child_count() {
                        if let Some(inner) = child.child(j) {
                            if inner.kind() == "function_definition" {
                                if let Some(sig) = extract_fn_sig(inner, source) {
                                    if !sig.name.starts_with('_') {
                                        functions.push(sig);
                                    }
                                }
                            }
                            if inner.kind() == "class_definition" {
                                if let Some(sig) = extract_class_sig(inner, source) {
                                    if !sig.name.starts_with('_') {
                                        types.push(sig);
                                    }
                                }
                            }
                        }
                    }
                }
                "class_definition" => {
                    if let Some(sig) = extract_class_sig(child, source) {
                        if !sig.name.starts_with('_') {
                            types.push(sig);
                        }
                    }
                }
                "expression_statement" => {
                    // __all__ = ['foo', 'bar']
                    if let Some(assign) = find_child_by_kind(child, "assignment") {
                        collect_all_exports(assign, source, &mut reexports);
                    }
                }
                "assignment" => {
                    collect_all_exports(child, source, &mut reexports);
                }
                _ => {}
            }
        }
    }

    PublicInterface { functions, types, reexports }
}

fn extract_fn_sig<'a>(node: Node, source: &'a [u8]) -> Option<FunctionSignature<'a>> {
    let name_node = node.child_by_field_name("name")?;
    let name = node_text(name_node, source);

    let params = node
        .child_by_field_name("parameters")
        .map(|p| collect_params(p, source))
        .unwrap_or_default();

    let return_type = node
        .child_by_field_name("return_type")
        .map(|r| normalize_whitespace(node_text(r, source)));

    Some(FunctionSignature {
        name,
        params,
        return_type: match return_type {
            Some(s) if !s.is_empty() && s != "None" => {
                Some(Box::leak(s.into_boxed_str()) as &'static str)
            }
            _ => None,
        },
    })
}

fn collect_params<'a>(params_node: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut params = Vec::new();
    for i in 0..params_node.child_count() {
        if let Some(child) = params_node.child(i) {
            match child.kind() {
                "identifier" => {
                    // Plain parameter — skip self/cls
                    let name = node_text(child, source);
                    if name != "self" && name != "cls" {
                        params.push(name);
                    }
                }
                "typed_parameter" => {
                    // x: int — extract the type annotation, skip self/cls
                    if let Some(name_node) = child.child(0) {
                        let name = node_text(name_node, source);
                        if name == "self" || name == "cls" {
                            continue;
                        }
                    }
                    // Find type annotation
                    if let Some(type_node) = child.child_by_field_name("type") {
                        params.push(node_text(type_node, source));
                    } else {
                        // Fallback: full node text
                        params.push(node_text(child, source));
                    }
                }
                "default_parameter" | "typed_default_parameter" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = node_text(name_node, source);
                        if name == "self" || name == "cls" {
                            continue;
                        }
                    }
                    if let Some(type_node) = child.child_by_field_name("type") {
                        params.push(node_text(type_node, source));
                    } else if let Some(name_node) = child.child_by_field_name("name") {
                        params.push(node_text(name_node, source));
                    }
                }
                "list_splat_pattern" | "dictionary_splat_pattern" => {
                    // *args, **kwargs — include as-is
                    params.push(node_text(child, source));
                }
                _ => {}
            }
        }
    }
    params
}

fn extract_class_sig<'a>(node: Node, source: &'a [u8]) -> Option<TypeSignature<'a>> {
    let name_node = node.child_by_field_name("name")?;
    let name = node_text(name_node, source);

    let kind = if is_protocol_abc_class(node, source) {
        TypeKind::Interface
    } else {
        TypeKind::Class
    };

    let members = collect_class_members(node, source);

    Some(TypeSignature { name, kind, members })
}

fn collect_class_members<'a>(class_node: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut members = Vec::new();
    if let Some(body) = class_node.child_by_field_name("body") {
        for i in 0..body.child_count() {
            if let Some(child) = body.child(i) {
                match child.kind() {
                    "function_definition" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name = node_text(name_node, source);
                            if !name.starts_with('_') {
                                members.push(name);
                            }
                        }
                    }
                    "decorated_definition" => {
                        for j in 0..child.child_count() {
                            if let Some(inner) = child.child(j) {
                                if inner.kind() == "function_definition" {
                                    if let Some(name_node) = inner.child_by_field_name("name") {
                                        let name = node_text(name_node, source);
                                        if !name.starts_with('_') {
                                            members.push(name);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    members
}

/// Colecta os nomes de `__all__ = ['foo', 'bar']` como reexports.
fn collect_all_exports<'a>(
    assign_node: Node,
    source: &'a [u8],
    reexports: &mut Vec<&'a str>,
) {
    // Check if left side is __all__
    let left = match assign_node.child_by_field_name("left") {
        Some(l) => l,
        None => return,
    };
    if node_text(left, source) != "__all__" {
        return;
    }
    // Right side is a list literal
    let right = match assign_node.child_by_field_name("right") {
        Some(r) => r,
        None => return,
    };
    if right.kind() == "list" {
        // The full list text as reexport (consistent with how TsParser handles export clauses)
        reexports.push(node_text(right, source));
    }
}

// ── declared_traits (V11) ─────────────────────────────────────────────────────

/// Extrai nomes de classes que herdam de Protocol/ABC em L1/contracts/.
fn extract_declared_traits<'a>(root: Node, source: &'a [u8]) -> Vec<&'a str> {
    let mut traits = Vec::new();
    for i in 0..root.child_count() {
        if let Some(child) = root.child(i) {
            if child.kind() == "class_definition" {
                if is_protocol_abc_class(child, source) {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = node_text(name_node, source);
                        if !name.starts_with('_') {
                            traits.push(name);
                        }
                    }
                }
            }
        }
    }
    traits
}

// ── implemented_traits (V11) ──────────────────────────────────────────────────

/// Extrai nomes das bases que vêm de L1/contracts/ em L2/L3.
fn extract_implemented_traits<'a>(
    root: Node,
    source: &'a [u8],
    import_name_map: &HashMap<&'a str, (Layer, Option<&'static str>)>,
) -> Vec<&'a str> {
    let mut traits = Vec::new();
    for i in 0..root.child_count() {
        if let Some(child) = root.child(i) {
            collect_implemented_from_class(child, source, import_name_map, &mut traits);
        }
    }
    traits
}

fn collect_implemented_from_class<'a>(
    node: Node,
    source: &'a [u8],
    import_name_map: &HashMap<&'a str, (Layer, Option<&'static str>)>,
    traits: &mut Vec<&'a str>,
) {
    if node.kind() == "class_definition" {
        collect_contract_bases(node, source, import_name_map, traits);
    }
    // Also handle decorated class
    if node.kind() == "decorated_definition" {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "class_definition" {
                    collect_contract_bases(child, source, import_name_map, traits);
                }
            }
        }
    }
}

fn collect_contract_bases<'a>(
    class_node: Node,
    source: &'a [u8],
    import_name_map: &HashMap<&'a str, (Layer, Option<&'static str>)>,
    traits: &mut Vec<&'a str>,
) {
    let bases = match class_node.child_by_field_name("superclasses") {
        Some(b) => b,
        None => return,
    };
    // Iterate base class names
    for i in 0..bases.child_count() {
        if let Some(base) = bases.child(i) {
            if matches!(base.kind(), "identifier" | "dotted_name" | "attribute") {
                let base_name = node_text(base, source);
                // Simple name (last segment for dotted)
                let simple = base_name.rsplit('.').next().unwrap_or(base_name);
                if let Some((layer, subdir)) = import_name_map.get(simple) {
                    if *layer == Layer::L1 && *subdir == Some("contracts") {
                        traits.push(simple);
                    }
                }
            }
        }
    }
}

// ── declarations (V12) ────────────────────────────────────────────────────────

/// Extrai declarações de classe em nível superior para todos os arquivos.
/// Exclui: classes com base Protocol/ABC (contrato) e classes com base de contracts/ (adapter).
fn extract_declarations<'a>(
    root: Node,
    source: &'a [u8],
    import_name_map: &HashMap<&'a str, (Layer, Option<&'static str>)>,
) -> Vec<Declaration<'a>> {
    let mut decls = Vec::new();
    for i in 0..root.child_count() {
        if let Some(child) = root.child(i) {
            match child.kind() {
                "class_definition" => {
                    if let Some(decl) = maybe_declaration(child, source, import_name_map) {
                        decls.push(decl);
                    }
                }
                "decorated_definition" => {
                    for j in 0..child.child_count() {
                        if let Some(inner) = child.child(j) {
                            if inner.kind() == "class_definition" {
                                if let Some(decl) =
                                    maybe_declaration(inner, source, import_name_map)
                                {
                                    decls.push(decl);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    decls
}

fn maybe_declaration<'a>(
    class_node: Node,
    source: &'a [u8],
    import_name_map: &HashMap<&'a str, (Layer, Option<&'static str>)>,
) -> Option<Declaration<'a>> {
    // Exclude Protocol/ABC contracts
    if is_protocol_abc_class(class_node, source) {
        return None;
    }
    // Exclude adapter classes (base from L1/contracts/)
    if has_contracts_base(class_node, source, import_name_map) {
        return None;
    }
    let name_node = class_node.child_by_field_name("name")?;
    let name = node_text(name_node, source);
    let line = class_node.start_position().row + 1;
    Some(Declaration { kind: DeclarationKind::Class, name, line })
}

fn has_contracts_base(
    class_node: Node,
    source: &[u8],
    import_name_map: &HashMap<&str, (Layer, Option<&'static str>)>,
) -> bool {
    let bases = match class_node.child_by_field_name("superclasses") {
        Some(b) => b,
        None => return false,
    };
    for i in 0..bases.child_count() {
        if let Some(base) = bases.child(i) {
            if matches!(base.kind(), "identifier" | "dotted_name" | "attribute") {
                let base_name = node_text(base, source);
                let simple = base_name.rsplit('.').next().unwrap_or(base_name);
                if let Some((layer, subdir)) = import_name_map.get(simple) {
                    if *layer == Layer::L1 && *subdir == Some("contracts") {
                        return true;
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

fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
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

fn find_child_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == kind {
                return Some(child);
            }
        }
    }
    None
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    use crate::contracts::file_provider::SourceFile;
    use crate::entities::layer::{Language, Layer};

    // ── Mock infrastructure ───────────────────────────────────────────────────

    struct NullPromptReader;
    impl crate::contracts::prompt_reader::PromptReader for NullPromptReader {
        fn exists(&self, _path: &str) -> bool { false }
        fn read_hash(&self, _path: &str) -> Option<String> { None }
    }

    struct NullSnapshotReader;
    impl crate::contracts::prompt_snapshot_reader::PromptSnapshotReader for NullSnapshotReader {
        fn read_snapshot(&self, _path: &str) -> Option<crate::entities::parsed_file::PublicInterface<'static>> { None }
        fn serialize_snapshot(&self, _iface: &crate::entities::parsed_file::PublicInterface<'_>) -> String { String::new() }
    }

    fn make_parser() -> PyParser<NullPromptReader, NullSnapshotReader> {
        PyParser::new(
            NullPromptReader,
            NullSnapshotReader,
            CrystallineConfig::default(),
            PathBuf::from("."),
        )
    }

    fn make_file(path: &'static str, content: &'static str, layer: Layer) -> SourceFile {
        SourceFile {
            path: PathBuf::from(path),
            content: content.to_string(),
            language: Language::Python,
            layer,
            has_adjacent_test: false,
        }
    }

    // ── Header tests ──────────────────────────────────────────────────────────

    #[test]
    fn header_all_fields_extracted() {
        let src = "\
# Crystalline Lineage
# @prompt 00_nucleo/prompts/foo.md
# @prompt-hash abcd1234
# @layer L3
# @updated 2026-03-19
import os
";
        let file = make_file("03_infra/foo.py", src, Layer::L3);
        let result = make_parser().parse(&file).unwrap();
        let h = result.prompt_header.unwrap();
        assert_eq!(h.prompt_path, "00_nucleo/prompts/foo.md");
        assert_eq!(h.prompt_hash, Some("abcd1234"));
        assert_eq!(h.layer, Layer::L3);
        assert_eq!(h.updated, Some("2026-03-19"));
    }

    #[test]
    fn header_stops_at_non_comment_line() {
        let src = "import os\n# @prompt foo.md\n";
        let file = make_file("03_infra/foo.py", src, Layer::L3);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.prompt_header.is_none(), "non-# first line should stop header scan");
    }

    // ── UnsupportedLanguage / EmptySource ─────────────────────────────────────

    #[test]
    fn unsupported_language_returns_error() {
        let file = SourceFile {
            path: PathBuf::from("01_core/foo.rs"),
            content: "pub fn foo() {}".to_string(),
            language: Language::Rust,
            layer: Layer::L1,
            has_adjacent_test: false,
        };
        assert!(matches!(make_parser().parse(&file), Err(ParseError::UnsupportedLanguage { .. })));
    }

    #[test]
    fn empty_source_returns_error() {
        let file = make_file("03_infra/empty.py", "", Layer::L3);
        assert!(matches!(make_parser().parse(&file), Err(ParseError::EmptySource { .. })));
    }

    // ── Import resolution tests ───────────────────────────────────────────────

    #[test]
    fn import_os_is_external() {
        let src = "import os\n";
        let file = make_file("01_core/entities/layer.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.imports.iter().any(|i| i.target_layer == Layer::Unknown));
    }

    #[test]
    fn from_typing_import_is_external() {
        let src = "from typing import Protocol\n";
        let file = make_file("01_core/contracts/fp.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.imports.iter().any(|i| i.target_layer == Layer::Unknown));
    }

    #[test]
    fn relative_import_resolves_to_correct_layer() {
        // file in 01_core/contracts/fp.py
        // from ..entities import Layer  → 01_core/entities → L1
        let src = "from ..entities import Layer\n";
        let file = make_file("01_core/contracts/fp.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.imports.iter().any(|i| i.target_layer == Layer::L1));
    }

    #[test]
    fn relative_import_with_escaping_dots_gives_unknown() {
        // file in 01_core/fp.py, ../../../../../../etc → escapes root
        let src = "from ......etc import passwd\n";
        let file = make_file("01_core/fp.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.imports.iter().any(|i| i.target_layer == Layer::Unknown));
    }

    #[test]
    fn alias_resolves_to_correct_layer() {
        let mut config = CrystallineConfig::default();
        config.py_aliases.insert("core".to_string(), "01_core".to_string());
        let parser = PyParser::new(
            NullPromptReader,
            NullSnapshotReader,
            config,
            PathBuf::from("."),
        );
        let src = "from core.contracts import FileProvider\n";
        let file = make_file("03_infra/walker.py", src, Layer::L3);
        let result = parser.parse(&file).unwrap();
        let imp = result.imports.iter().find(|i| i.target_layer == Layer::L1);
        assert!(imp.is_some(), "alias should resolve to L1");
        assert_eq!(imp.unwrap().target_subdir, Some("contracts"));
    }

    #[test]
    fn import_target_subdir_is_none_for_l3() {
        let src = "from .walker import FileWalker\n";
        let file = make_file("03_infra/mod.py", src, Layer::L3);
        let result = make_parser().parse(&file).unwrap();
        // walker.py resolves to L3, subdir should be None
        assert!(result.imports.iter().any(|i| i.target_subdir.is_none()));
    }

    #[test]
    fn import_lab_resolves_to_lab_layer() {
        // file in 01_core/foo.py
        // from ../../lab import experiment  → lab/ → Lab
        let src = "from ...lab import experiment\n";
        let file = make_file("01_core/entities/foo.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        // 01_core/entities + ../../lab = lab → Layer::Lab
        assert!(result.imports.iter().any(|i| i.target_layer == Layer::Lab));
    }

    // ── V4 token tests ────────────────────────────────────────────────────────

    #[test]
    fn import_os_generates_token() {
        let src = "import os\n";
        let file = make_file("01_core/entities/layer.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.tokens.iter().any(|t| t.symbol.as_ref() == "os"));
    }

    #[test]
    fn import_pathlib_generates_token() {
        let src = "import pathlib\n";
        let file = make_file("01_core/entities/layer.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.tokens.iter().any(|t| t.symbol.as_ref() == "pathlib"));
    }

    #[test]
    fn open_call_generates_token() {
        let src = "x = open('file.txt')\n";
        let file = make_file("01_core/entities/layer.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.tokens.iter().any(|t| t.symbol.as_ref() == "open"));
    }

    #[test]
    fn random_random_call_generates_token() {
        let src = "x = random.random()\n";
        let file = make_file("01_core/entities/layer.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.tokens.iter().any(|t| t.symbol.as_ref() == "random.random"));
    }

    // ── V2 test coverage tests ────────────────────────────────────────────────

    #[test]
    fn test_class_inheriting_testcase_gives_coverage() {
        let src = "import unittest\nclass FooTest(unittest.TestCase):\n    def test_it(self): pass\n";
        let file = make_file("01_core/entities/layer.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.has_test_coverage);
    }

    #[test]
    fn class_without_inheritance_does_not_give_coverage() {
        let src = "class FooTest:\n    pass\n";
        let file = make_file("01_core/entities/layer.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(!result.has_test_coverage);
    }

    #[test]
    fn class_with_non_testcase_inheritance_does_not_give_coverage() {
        let src = "class FooTest(SomethingElse):\n    pass\n";
        let file = make_file("01_core/entities/layer.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(!result.has_test_coverage);
    }

    #[test]
    fn nested_test_class_does_not_give_coverage() {
        let src = "def outer():\n    class InnerTest(unittest.TestCase):\n        pass\n";
        let file = make_file("01_core/entities/layer.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(!result.has_test_coverage);
    }

    #[test]
    fn adjacent_test_file_gives_coverage() {
        let src = "def foo(): pass\n";
        let mut file = make_file("01_core/entities/layer.py", src, Layer::L1);
        file.has_adjacent_test = true;
        let result = make_parser().parse(&file).unwrap();
        assert!(result.has_test_coverage);
    }

    #[test]
    fn declaration_only_file_is_exempt_from_v2() {
        let src = "\
from typing import Protocol

class FileProvider(Protocol):
    def files(self) -> list: ...
";
        let file = make_file("01_core/contracts/fp.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.has_test_coverage, "declaration-only should be exempt");
    }

    #[test]
    fn file_with_implementation_is_not_declaration_only() {
        let src = "def real_fn(x):\n    return x + 1\n";
        let file = make_file("01_core/rules/check.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        // real function body → not declaration-only → no test calls → no adjacent test
        assert!(!result.has_test_coverage);
    }

    // ── V6 public interface tests ─────────────────────────────────────────────

    #[test]
    fn public_function_extracted() {
        let src = "def check(file: ParsedFile) -> list:\n    return []\n";
        let file = make_file("01_core/rules/check.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert_eq!(result.public_interface.functions.len(), 1);
        assert_eq!(result.public_interface.functions[0].name, "check");
    }

    #[test]
    fn private_function_not_in_interface() {
        let src = "def _helper(): pass\n";
        let file = make_file("01_core/rules/check.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.public_interface.functions.is_empty());
    }

    #[test]
    fn protocol_class_has_interface_kind() {
        let src = "from typing import Protocol\nclass FileProvider(Protocol):\n    def files(self): ...\n";
        let file = make_file("01_core/contracts/fp.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        let type_sig = result.public_interface.types.iter().find(|t| t.name == "FileProvider");
        assert!(type_sig.is_some());
        assert_eq!(type_sig.unwrap().kind, TypeKind::Interface);
    }

    #[test]
    fn plain_class_has_class_kind() {
        let src = "class FileWalker:\n    def walk(self): pass\n";
        let file = make_file("03_infra/walker.py", src, Layer::L3);
        let result = make_parser().parse(&file).unwrap();
        let type_sig = result.public_interface.types.iter().find(|t| t.name == "FileWalker");
        assert!(type_sig.is_some());
        assert_eq!(type_sig.unwrap().kind, TypeKind::Class);
    }

    // ── V11 declared_traits tests ─────────────────────────────────────────────

    #[test]
    fn protocol_class_in_contracts_adds_declared_trait() {
        let src = "from typing import Protocol\nclass FileProvider(Protocol):\n    def files(self): ...\n";
        let file = make_file("01_core/contracts/fp.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.declared_traits.contains(&"FileProvider"));
    }

    #[test]
    fn protocol_class_outside_contracts_not_in_declared_traits() {
        let src = "from typing import Protocol\nclass HasImports(Protocol):\n    def imports(self): ...\n";
        let file = make_file("01_core/rules/check.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.declared_traits.is_empty());
    }

    #[test]
    fn private_protocol_not_in_declared_traits() {
        let src = "from typing import Protocol\nclass _Internal(Protocol):\n    pass\n";
        let file = make_file("01_core/contracts/fp.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.declared_traits.is_empty());
    }

    // ── V11 implemented_traits tests ──────────────────────────────────────────

    #[test]
    fn class_with_contracts_base_adds_implemented_trait() {
        let mut config = CrystallineConfig::default();
        config.py_aliases.insert("core".to_string(), "01_core".to_string());
        let parser = PyParser::new(
            NullPromptReader,
            NullSnapshotReader,
            config,
            PathBuf::from("."),
        );
        let src = "\
from core.contracts import FileProvider
class FileWalker(FileProvider):
    def walk(self): pass
class InternalHelper:
    pass
";
        let file = SourceFile {
            path: PathBuf::from("03_infra/walker.py"),
            content: src.to_string(),
            language: Language::Python,
            layer: Layer::L3,
            has_adjacent_test: false,
        };
        let result = parser.parse(&file).unwrap();
        assert!(result.implemented_traits.contains(&"FileProvider"));
        assert!(!result.implemented_traits.contains(&"InternalHelper"));
    }

    #[test]
    fn implemented_traits_empty_in_l1() {
        let src = "from typing import Protocol\nclass Foo(Protocol): pass\n";
        let file = make_file("01_core/contracts/fp.py", src, Layer::L1);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.implemented_traits.is_empty());
    }

    // ── V12 declarations tests ────────────────────────────────────────────────

    #[test]
    fn plain_class_in_l4_generates_declaration() {
        let src = "class OutputFormatter:\n    pass\n";
        let file = make_file("04_wiring/main.py", src, Layer::L4);
        let result = make_parser().parse(&file).unwrap();
        assert!(result.declarations.iter().any(|d| d.name == "OutputFormatter"));
    }

    #[test]
    fn protocol_class_not_in_declarations() {
        let src = "from typing import Protocol\nclass Config(Protocol): pass\n";
        let file = make_file("04_wiring/main.py", src, Layer::L4);
        let result = make_parser().parse(&file).unwrap();
        assert!(!result.declarations.iter().any(|d| d.name == "Config"));
    }

    #[test]
    fn adapter_class_not_in_declarations() {
        let mut config = CrystallineConfig::default();
        config.py_aliases.insert("core".to_string(), "01_core".to_string());
        let parser = PyParser::new(
            NullPromptReader,
            NullSnapshotReader,
            config,
            PathBuf::from("."),
        );
        let src = "\
from core.contracts import HashRewriter
class L3HashAdapter(HashRewriter):
    pass
class OutputFormatter:
    pass
";
        let file = SourceFile {
            path: PathBuf::from("04_wiring/main.py"),
            content: src.to_string(),
            language: Language::Python,
            layer: Layer::L4,
            has_adjacent_test: false,
        };
        let result = parser.parse(&file).unwrap();
        // Adapter should NOT be in declarations
        assert!(!result.declarations.iter().any(|d| d.name == "L3HashAdapter"));
        // Plain class SHOULD be in declarations
        assert!(result.declarations.iter().any(|d| d.name == "OutputFormatter"));
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

    // ── relative_module_to_path tests ─────────────────────────────────────────

    #[test]
    fn single_dot_no_module_gives_cur_dir() {
        assert_eq!(relative_module_to_path(1, ""), ".");
    }

    #[test]
    fn single_dot_with_module_gives_module() {
        assert_eq!(relative_module_to_path(1, "utils"), "utils");
    }

    #[test]
    fn double_dot_with_module_gives_parent_join() {
        assert_eq!(relative_module_to_path(2, "core"), "../core");
    }

    #[test]
    fn double_dot_no_module_gives_parent() {
        assert_eq!(relative_module_to_path(2, ""), "..");
    }

    #[test]
    fn triple_dot_with_module() {
        assert_eq!(relative_module_to_path(3, "lab"), "../../lab");
    }

}
