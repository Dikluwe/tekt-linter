//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/prompt-snapshot-reader.md
//! @prompt-hash 7238fc56
//! @layer L3
//! @updated 2026-03-14

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crate::entities::parsed_file::{
    FunctionSignature, PublicInterface, TypeKind, TypeSignature,
};

// ── Owned intermediates for serde ─────────────────────────────────────────────
//
// PublicInterface<'a> uses &'a str — cannot derive Deserialize.
// L3 deserializes into owned types, then converts to PublicInterface<'static>
// via Box::leak (ADR-0004 + ADR-0005: snapshot data lives for the duration
// of the process; Box::leak is the documented pattern for prompt snapshots).

#[derive(Debug, Serialize, Deserialize)]
struct OwnedFunctionSignature {
    name: String,
    params: Vec<String>,
    return_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum OwnedTypeKind {
    Struct,
    Enum,
    Trait,
    // ADR-0009: linguagens OO
    Class,
    Interface,
    #[serde(rename = "type")]
    TypeAlias,
}

#[derive(Debug, Serialize, Deserialize)]
struct OwnedTypeSignature {
    name: String,
    kind: OwnedTypeKind,
    members: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OwnedPublicInterface {
    functions: Vec<OwnedFunctionSignature>,
    types: Vec<OwnedTypeSignature>,
    reexports: Vec<String>,
}

fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn owned_to_static(owned: OwnedPublicInterface) -> PublicInterface<'static> {
    PublicInterface {
        functions: owned
            .functions
            .into_iter()
            .map(|f| FunctionSignature {
                name: leak_str(f.name),
                params: f.params.into_iter().map(leak_str).collect(),
                return_type: f.return_type.map(leak_str),
            })
            .collect(),
        types: owned
            .types
            .into_iter()
            .map(|t| TypeSignature {
                name: leak_str(t.name),
                kind: match t.kind {
                    OwnedTypeKind::Struct    => TypeKind::Struct,
                    OwnedTypeKind::Enum      => TypeKind::Enum,
                    OwnedTypeKind::Trait     => TypeKind::Trait,
                    OwnedTypeKind::Class     => TypeKind::Class,
                    OwnedTypeKind::Interface => TypeKind::Interface,
                    OwnedTypeKind::TypeAlias => TypeKind::TypeAlias,
                },
                members: t.members.into_iter().map(leak_str).collect(),
            })
            .collect(),
        reexports: owned.reexports.into_iter().map(leak_str).collect(),
    }
}

fn interface_to_owned(iface: &PublicInterface<'_>) -> OwnedPublicInterface {
    OwnedPublicInterface {
        functions: iface
            .functions
            .iter()
            .map(|f| OwnedFunctionSignature {
                name: f.name.to_string(),
                params: f.params.iter().map(|p| p.to_string()).collect(),
                return_type: f.return_type.map(|r| r.to_string()),
            })
            .collect(),
        types: iface
            .types
            .iter()
            .map(|t| OwnedTypeSignature {
                name: t.name.to_string(),
                kind: match t.kind {
                    TypeKind::Struct    => OwnedTypeKind::Struct,
                    TypeKind::Enum      => OwnedTypeKind::Enum,
                    TypeKind::Trait     => OwnedTypeKind::Trait,
                    TypeKind::Class     => OwnedTypeKind::Class,
                    TypeKind::Interface => OwnedTypeKind::Interface,
                    TypeKind::TypeAlias => OwnedTypeKind::TypeAlias,
                },
                members: t.members.iter().map(|m| m.to_string()).collect(),
            })
            .collect(),
        reexports: iface.reexports.iter().map(|r| r.to_string()).collect(),
    }
}

// ── FsPromptSnapshotReader ────────────────────────────────────────────────────

#[derive(Clone)]
pub struct FsPromptSnapshotReader {
    pub nucleo_root: PathBuf,
}

impl PromptSnapshotReader for FsPromptSnapshotReader {
    fn read_snapshot(&self, prompt_path: &str) -> Option<PublicInterface<'static>> {
        let full_path = self.nucleo_root.join(prompt_path);
        let content = std::fs::read_to_string(&full_path).ok()?;
        let json = extract_snapshot_json(&content)?;
        let owned: OwnedPublicInterface = serde_json::from_str(&json).ok()?;
        Some(owned_to_static(owned))
    }

    fn serialize_snapshot(&self, interface: &PublicInterface<'_>) -> String {
        let owned = interface_to_owned(interface);
        let json = serde_json::to_string(&owned).unwrap_or_default();
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
            functions: vec![FunctionSignature { name: "check", params: vec![], return_type: None }],
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
