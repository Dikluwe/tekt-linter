//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/file-walker.md
//! @prompt-hash b13dc387
//! @layer L3
//! @updated 2026-03-20

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::contracts::file_provider::{FileProvider, SourceError, SourceFile};
use crate::entities::layer::{Language, Layer};
use crate::infra::config::CrystallineConfig;

pub struct FileWalker {
    root: PathBuf,
    config: CrystallineConfig,
}

impl FileWalker {
    pub fn new(root: PathBuf, config: CrystallineConfig) -> Self {
        Self { root, config }
    }
}

impl FileProvider for FileWalker {
    fn files(&self) -> impl Iterator<Item = Result<SourceFile, SourceError>> {
        let root = self.root.clone();
        // Build two sets once (O(1) lookup) before the iterator.
        // Separate from config clone so filter_entry can capture them by move.
        let excluded_dirs: HashSet<String> =
            self.config.excluded.values().cloned().collect();
        let excluded_files: HashSet<String> =
            self.config.excluded_files.values().cloned().collect();
        let config = self.config.clone();
        // root is moved into filter_entry; root2 is moved into map.
        let root2 = root.clone();

        WalkDir::new(&root)
            .into_iter()
            .filter_entry(move |e| {
                !is_ignored(e.path(), &root, &excluded_dirs, &excluded_files)
            })
            .filter_map(|entry| entry.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| language_for_path(e.path()).is_some())
            .map(move |entry| {
                let path = entry.path().to_path_buf();
                let language = language_for_path(&path).expect("filtered above");
                let layer = resolve_file_layer(&path, &root2, &config);
                let has_adjacent_test = check_adjacent_test(&path);

                match std::fs::read_to_string(&path) {
                    Ok(content) => Ok(SourceFile { path, content, language, layer, has_adjacent_test }),
                    Err(e) => Err(SourceError::Unreadable { path, reason: e.to_string() }),
                }
            })
    }
}

/// Retorna true se o path deve ser ignorado.
///
/// Verifica primeiro `excluded_dirs` (componentes de path — para directórios)
/// e depois `excluded_files` (path relativo exacto — para ficheiros individuais).
/// `excluded_dirs` é construído de `config.excluded` — zero valores hardcoded (ADR-0006).
/// `excluded_files` é construído de `config.excluded_files` — exclusão por path relativo (ADR-0010).
fn is_ignored(
    path: &Path,
    root: &Path,
    excluded_dirs: &HashSet<String>,
    excluded_files: &HashSet<String>,
) -> bool {
    // 1. Dir exclusion: only check the last component.
    // filter_entry ensures we don't descend if a parent is ignored.
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if excluded_dirs.contains(name) {
            return true;
        }
    }

    // 2. File exclusion: check exact relative path.
    if let Ok(relative) = path.strip_prefix(root) {
        if let Some(rel_str) = relative.to_str() {
            let normalized = rel_str.replace('\\', "/");
            if excluded_files.contains(&normalized) {
                return true;
            }
        }
    }
    false
}

/// Map file extension to Language.
fn language_for_path(path: &Path) -> Option<Language> {
    match path.extension()?.to_str()? {
        "rs" => Some(Language::Rust),
        "ts" | "tsx" => Some(Language::TypeScript),
        "py" => Some(Language::Python),
        "c" | "h" => Some(Language::C),
        "cpp" | "hpp" | "cc" | "cxx" | "hxx" => Some(Language::Cpp),
        "zig" => Some(Language::Zig),
        _ => None,
    }
}

/// Determine the layer of a file from its path relative to the project root.
/// Uses the [layers] table in crystalline.toml to match path prefixes.
pub fn resolve_file_layer(path: &Path, root: &Path, config: &CrystallineConfig) -> Layer {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let first_component = relative
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("");

    for (layer_key, dir_name) in &config.layers {
        if first_component == dir_name.as_str() {
            return match layer_key.as_str() {
                "L0" => Layer::L0,
                "L1" => Layer::L1,
                "L2" => Layer::L2,
                "L3" => Layer::L3,
                "L4" => Layer::L4,
                "lab" | "Lab" => Layer::Lab,
                _ => Layer::Unknown,
            };
        }
    }

    Layer::Unknown
}

/// Returns true if a sibling test file exists in the same directory.
/// Patterns checked:
/// - Rust:       `<stem>_test.rs`
/// - TypeScript: `<stem>.test.ts`, `<stem>.spec.ts`, `<stem>.test.tsx`, `<stem>.spec.tsx`
/// - Python:     `<stem>_test.py` or `test_<stem>.py`
/// - Zig:        `<stem>_test.zig`
fn check_adjacent_test(path: &Path) -> bool {
    let stem = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return false,
    };
    let dir = match path.parent() {
        Some(d) => d,
        None => return false,
    };

    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => {
            if stem.ends_with("_test") { return false; }
            dir.join(format!("{}_test.rs", stem)).exists()
        }
        Some("ts") | Some("tsx") => {
            if stem.contains(".test") || stem.contains(".spec") { return false; }
            dir.join(format!("{}.test.ts",  stem)).exists()
                || dir.join(format!("{}.spec.ts",  stem)).exists()
                || dir.join(format!("{}.test.tsx", stem)).exists()
                || dir.join(format!("{}.spec.tsx", stem)).exists()
        }
        Some("py") => {
            if stem.ends_with("_test") || stem.starts_with("test_") { return false; }
            dir.join(format!("{}_test.py", stem)).exists()
                || dir.join(format!("test_{}.py", stem)).exists()
        }
        Some("zig") => {
            if stem.ends_with("_test") { return false; }
            dir.join(format!("{}_test.zig", stem)).exists()
        }
        Some("c") => {
            if stem.ends_with("_test") || stem.starts_with("test_") { return false; }
            dir.join(format!("{}_test.c", stem)).exists()
                || dir.join(format!("test_{}.c", stem)).exists()
        }
        Some("cpp") | Some("cc") | Some("cxx") => {
            if stem.ends_with("_test") || stem.starts_with("test_") { return false; }
            dir.join(format!("{}_test.cpp", stem)).exists()
                || dir.join(format!("test_{}.cpp", stem)).exists()
                || dir.join(format!("{}_test.cc", stem)).exists()
                || dir.join(format!("test_{}.cc", stem)).exists()
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_project() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write_file(dir: &Path, rel: &str, content: &str) {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn collect_ok(walker: &FileWalker) -> Vec<SourceFile> {
        walker.files().filter_map(|r| r.ok()).collect()
    }

    #[test]
    fn walker_finds_rs_files() {
        let dir = setup_project();
        write_file(dir.path(), "01_core/foo.rs", "fn foo() {}");
        let config = CrystallineConfig::default();
        let walker = FileWalker::new(dir.path().to_path_buf(), config);
        let files = collect_ok(&walker);
        assert_eq!(files.len(), 1);
        assert!(files[0].path.ends_with("foo.rs"));
    }

    #[test]
    fn walker_skips_target_directory() {
        let dir = setup_project();
        write_file(dir.path(), "01_core/foo.rs", "fn foo() {}");
        write_file(dir.path(), "target/debug/build.rs", "fn build() {}");
        let config = CrystallineConfig::default();
        let walker = FileWalker::new(dir.path().to_path_buf(), config);
        let files = collect_ok(&walker);
        assert_eq!(files.len(), 1);
        assert!(!files[0].path.to_str().unwrap().contains("target"));
    }

    #[test]
    fn walker_with_empty_excluded_does_not_skip_target() {
        let dir = setup_project();
        write_file(dir.path(), "01_core/foo.rs", "fn foo() {}");
        write_file(dir.path(), "target/debug/build.rs", "fn build() {}");
        let mut config = CrystallineConfig::default();
        config.excluded.clear(); // zero exclusões → target não é excluído
        let walker = FileWalker::new(dir.path().to_path_buf(), config);
        let files = collect_ok(&walker);
        // Agora target/debug/build.rs deve aparecer com Layer::Unknown
        assert_eq!(files.len(), 2);
        let target_file = files.iter().find(|f| f.path.to_str().unwrap().contains("target")).unwrap();
        assert_eq!(target_file.layer, Layer::Unknown);
    }

    #[test]
    fn walker_detects_adjacent_test() {
        let dir = setup_project();
        write_file(dir.path(), "01_core/foo.rs", "fn foo() {}");
        write_file(dir.path(), "01_core/foo_test.rs", "#[test] fn t() {}");
        let config = CrystallineConfig::default();
        let walker = FileWalker::new(dir.path().to_path_buf(), config);
        let files = collect_ok(&walker);
        // both files are returned; foo.rs should have has_adjacent_test = true
        let foo = files.iter().find(|f| f.path.ends_with("foo.rs")).unwrap();
        assert!(foo.has_adjacent_test);
    }

    #[test]
    fn walker_sets_layer_from_config() {
        let dir = setup_project();
        write_file(dir.path(), "02_shell/cli.rs", "fn cli() {}");
        let config = CrystallineConfig::default();
        let walker = FileWalker::new(dir.path().to_path_buf(), config);
        let files = collect_ok(&walker);
        assert_eq!(files[0].layer, Layer::L2);
    }

    #[test]
    fn walker_unknown_layer_not_dropped() {
        let dir = setup_project();
        write_file(dir.path(), "src/utils/helper.rs", "fn help() {}");
        let config = CrystallineConfig::default(); // "src" not in [layers]
        let walker = FileWalker::new(dir.path().to_path_buf(), config);
        let files = collect_ok(&walker);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].layer, Layer::Unknown);
    }

    #[test]
    fn resolve_file_layer_returns_l1_for_core() {
        let config = CrystallineConfig::default();
        let root = Path::new("/project");
        let path = Path::new("/project/01_core/entities/layer.rs");
        assert_eq!(resolve_file_layer(path, root, &config), Layer::L1);
    }

    #[test]
    fn adjacent_test_false_when_no_test_file() {
        let dir = setup_project();
        write_file(dir.path(), "01_core/bar.rs", "fn bar() {}");
        assert!(!check_adjacent_test(&dir.path().join("01_core/bar.rs")));
    }

    #[test]
    fn ts_adjacent_test_detected_for_test_ts() {
        let dir = tempfile::tempdir().unwrap();
        let foo_ts = dir.path().join("foo.ts");
        let foo_test_ts = dir.path().join("foo.test.ts");
        std::fs::write(&foo_ts, "export const x = 1;").unwrap();
        std::fs::write(&foo_test_ts, "test('x', () => {});").unwrap();
        assert!(check_adjacent_test(&foo_ts));
    }

    #[test]
    fn ts_adjacent_spec_ts_detected() {
        let dir = tempfile::tempdir().unwrap();
        let foo_ts = dir.path().join("foo.ts");
        let foo_spec_ts = dir.path().join("foo.spec.ts");
        std::fs::write(&foo_ts, "export const x = 1;").unwrap();
        std::fs::write(&foo_spec_ts, "it('x', () => {});").unwrap();
        assert!(check_adjacent_test(&foo_ts));
    }

    #[test]
    fn tsx_adjacent_test_tsx_detected() {
        let dir = tempfile::tempdir().unwrap();
        let foo_tsx = dir.path().join("foo.tsx");
        let foo_test_tsx = dir.path().join("foo.test.tsx");
        std::fs::write(&foo_tsx, "export const C = () => <div/>;").unwrap();
        std::fs::write(&foo_test_tsx, "test('C', () => {});").unwrap();
        assert!(check_adjacent_test(&foo_tsx));
    }

    #[test]
    fn ts_no_adjacent_test_when_none_exists() {
        let dir = tempfile::tempdir().unwrap();
        let bar_ts = dir.path().join("bar.ts");
        std::fs::write(&bar_ts, "export const y = 2;").unwrap();
        assert!(!check_adjacent_test(&bar_ts));
    }

    #[test]
    fn ts_test_file_itself_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        let foo_test_ts = dir.path().join("foo.test.ts");
        std::fs::write(&foo_test_ts, "test('x', () => {});").unwrap();
        assert!(!check_adjacent_test(&foo_test_ts));
    }

    // ── Critérios adicionais do prompt file-walker.md ─────────────────────────

    #[test]
    fn tsx_adjacent_spec_tsx_detected() {
        // Dado diretório com foo.tsx e foo.spec.tsx
        // Então SourceFile para foo.tsx tem has_adjacent_test = true
        let dir = tempfile::tempdir().unwrap();
        let foo_tsx = dir.path().join("foo.tsx");
        let foo_spec_tsx = dir.path().join("foo.spec.tsx");
        std::fs::write(&foo_tsx, "export const C = () => <div/>;").unwrap();
        std::fs::write(&foo_spec_tsx, "it('C', () => {});").unwrap();
        assert!(check_adjacent_test(&foo_tsx));
    }

    #[test]
    fn ts_spec_file_itself_returns_false() {
        // Dado arquivo foo.spec.ts (já é ficheiro de teste)
        // Então has_adjacent_test = false — ele é o teste, não o ficheiro testado
        let dir = tempfile::tempdir().unwrap();
        let foo_spec_ts = dir.path().join("foo.spec.ts");
        std::fs::write(&foo_spec_ts, "it('x', () => {});").unwrap();
        assert!(!check_adjacent_test(&foo_spec_ts));
    }

    // ── Critérios ADR-0010: excluded_files ────────────────────────────────────

    #[test]
    fn excluded_files_prevents_specific_file() {
        // Dado excluded_files = { "crate_root": "lib.rs" } e lib.rs na raiz
        // Então lib.rs não aparece no iterator
        let dir = setup_project();
        write_file(dir.path(), "lib.rs", "pub mod foo;");
        write_file(dir.path(), "01_core/foo.rs", "fn foo() {}");
        let mut config = CrystallineConfig::default();
        config.excluded_files.insert("crate_root".to_string(), "lib.rs".to_string());
        let walker = FileWalker::new(dir.path().to_path_buf(), config);
        let files = collect_ok(&walker);
        assert_eq!(files.len(), 1);
        assert!(files[0].path.ends_with("foo.rs"));
    }

    #[test]
    fn excluded_files_does_not_affect_same_name_in_subdir() {
        // Dado excluded_files = { "crate_root": "lib.rs" }
        // E lib.rs na raiz E 01_core/lib.rs num subdirectório
        // Então apenas 01_core/lib.rs aparece — excluded_files é path relativo exacto
        let dir = setup_project();
        write_file(dir.path(), "lib.rs", "pub mod foo;");
        write_file(dir.path(), "01_core/lib.rs", "pub mod bar;");
        let mut config = CrystallineConfig::default();
        config.excluded_files.insert("crate_root".to_string(), "lib.rs".to_string());
        let walker = FileWalker::new(dir.path().to_path_buf(), config);
        let files = collect_ok(&walker);
        assert_eq!(files.len(), 1);
        assert!(files[0].path.to_str().unwrap().contains("01_core"));
    }
}
