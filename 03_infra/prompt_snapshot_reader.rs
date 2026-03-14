//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/prompt-snapshot-reader.md
//! @prompt-hash 00000000
//! @layer L3
//! @updated 2026-03-14

use std::path::PathBuf;

use crate::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crate::entities::parsed_file::PublicInterface;

// ── FsPromptSnapshotReader ────────────────────────────────────────────────────

pub struct FsPromptSnapshotReader {
    pub nucleo_root: PathBuf,
}

impl PromptSnapshotReader for FsPromptSnapshotReader {
    fn read_snapshot(&self, prompt_path: &str) -> Option<PublicInterface> {
        let full_path = self.nucleo_root.join(prompt_path);
        let content = std::fs::read_to_string(&full_path).ok()?;
        extract_snapshot_json(&content)
            .and_then(|json| serde_json::from_str(&json).ok())
    }

    fn serialize_snapshot(&self, interface: &PublicInterface) -> String {
        let json = serde_json::to_string(interface).unwrap_or_default();
        format!(
            "## Interface Snapshot\n\
             <!-- GENERATED — não edite manualmente -->\n\
             <!-- crystalline-snapshot: {} -->",
            json
        )
    }
}

/// Extract the JSON payload from the `<!-- crystalline-snapshot: {...} -->` line.
pub fn extract_snapshot_json(content: &str) -> Option<String> {
    content
        .lines()
        .find(|line| line.contains("crystalline-snapshot:"))
        .and_then(|line| {
            let start = line.find('{')?;
            let end = line.rfind('}')? + 1;
            Some(line[start..end].to_string())
        })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::parsed_file::{FunctionSignature, PublicInterface};
    use tempfile::TempDir;

    fn make_reader(dir: &TempDir) -> FsPromptSnapshotReader {
        FsPromptSnapshotReader { nucleo_root: dir.path().to_path_buf() }
    }

    #[test]
    fn reads_valid_snapshot() {
        let dir = TempDir::new().unwrap();
        let prompt = r#"## Interface Snapshot
<!-- GENERATED — não edite manualmente -->
<!-- crystalline-snapshot: {"functions":[{"name":"check","params":[],"return_type":null}],"types":[],"reexports":[]} -->
"#;
        std::fs::write(dir.path().join("foo.md"), prompt).unwrap();
        let reader = make_reader(&dir);
        let result = reader.read_snapshot("foo.md");
        assert!(result.is_some());
        let iface = result.unwrap();
        assert_eq!(iface.functions.len(), 1);
        assert_eq!(iface.functions[0].name, "check");
    }

    #[test]
    fn returns_none_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let reader = make_reader(&dir);
        assert!(reader.read_snapshot("nonexistent.md").is_none());
    }

    #[test]
    fn returns_none_when_no_snapshot_section() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("foo.md"), "# Some prompt\nno snapshot here\n").unwrap();
        let reader = make_reader(&dir);
        assert!(reader.read_snapshot("foo.md").is_none());
    }

    #[test]
    fn returns_none_when_json_malformed() {
        let dir = TempDir::new().unwrap();
        let prompt = "## Interface Snapshot\n<!-- crystalline-snapshot: {bad json} -->\n";
        std::fs::write(dir.path().join("foo.md"), prompt).unwrap();
        let reader = make_reader(&dir);
        assert!(reader.read_snapshot("foo.md").is_none());
    }

    #[test]
    fn serialize_snapshot_contains_json_and_marker() {
        let dir = TempDir::new().unwrap();
        let reader = make_reader(&dir);
        let iface = PublicInterface {
            functions: vec![FunctionSignature {
                name: "check".to_string(),
                params: vec![],
                return_type: None,
            }],
            types: vec![],
            reexports: vec![],
        };
        let out = reader.serialize_snapshot(&iface);
        assert!(out.contains("crystalline-snapshot:"));
        assert!(out.contains("\"check\""));
        assert!(out.contains("## Interface Snapshot"));
    }

    #[test]
    fn extract_snapshot_json_extracts_correctly() {
        let content = "## Interface Snapshot\n\
                       <!-- crystalline-snapshot: {\"functions\":[],\"types\":[],\"reexports\":[]} -->\n";
        let json = extract_snapshot_json(content);
        assert!(json.is_some());
        assert!(json.unwrap().contains("functions"));
    }
}
