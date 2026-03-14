//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/fix-hashes.md
//! @prompt-hash 7ed43b44
//! @layer L3
//! @updated 2026-03-14

use std::path::Path;

/// Replace (or append) the `## Interface Snapshot` section in a prompt file.
/// Atomic write: writes to a temp file then renames.
pub fn write_snapshot(prompt_path: &Path, new_snapshot: &str) -> Result<(), String> {
    let content = std::fs::read_to_string(prompt_path)
        .map_err(|e| format!("Failed to read {}: {}", prompt_path.display(), e))?;

    let new_content = replace_snapshot_section(&content, new_snapshot);

    let tmp_path = prompt_path.with_extension("crystalline-snap-tmp");
    std::fs::write(&tmp_path, &new_content)
        .map_err(|e| format!("Failed to write tmp file: {e}"))?;
    std::fs::rename(&tmp_path, prompt_path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp_path);
        format!("Failed to rename tmp file: {e}")
    })
}

/// Replace the `## Interface Snapshot` section with `new_snapshot`, or insert
/// it before `## Histórico de Revisões` (or append at end if neither exists).
fn replace_snapshot_section(content: &str, new_snapshot: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    // Find existing snapshot section
    let snapshot_start = lines.iter().position(|l| l.trim() == "## Interface Snapshot");

    if let Some(start) = snapshot_start {
        // Find the next `## ` heading after the snapshot section
        let end = lines[start + 1..]
            .iter()
            .position(|l| l.starts_with("## "))
            .map(|pos| start + 1 + pos)
            .unwrap_or(lines.len());

        let mut result = lines[..start].join("\n");
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(new_snapshot);
        result.push('\n');
        if end < lines.len() {
            result.push('\n');
            result.push_str(&lines[end..].join("\n"));
            result.push('\n');
        }
        result
    } else {
        // Insert before ## Histórico if it exists, otherwise append
        let hist = lines.iter().position(|l| l.starts_with("## Histórico"));
        if let Some(pos) = hist {
            let mut result = lines[..pos].join("\n").trim_end().to_string();
            result.push_str("\n\n");
            result.push_str(new_snapshot);
            result.push_str("\n\n---\n\n");
            result.push_str(&lines[pos..].join("\n"));
            result.push('\n');
            result
        } else {
            format!("{}\n\n{}\n", content.trim_end(), new_snapshot)
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const SNAPSHOT: &str = "## Interface Snapshot\n\
                            <!-- GENERATED — não edite manualmente -->\n\
                            <!-- crystalline-snapshot: {\"functions\":[],\"types\":[],\"reexports\":[]} -->";

    #[test]
    fn replaces_existing_snapshot_section() {
        let content = "# Prompt\n\
                       Some text.\n\
                       \n\
                       ## Interface Snapshot\n\
                       <!-- GENERATED — não edite manualmente -->\n\
                       <!-- crystalline-snapshot: {\"functions\":[],\"types\":[],\"reexports\":[]} -->\n\
                       \n\
                       ## Histórico de Revisões\n\
                       | Data | Motivo |\n";

        let new_snap = "## Interface Snapshot\n\
                        <!-- GENERATED — não edite manualmente -->\n\
                        <!-- crystalline-snapshot: {\"functions\":[{\"name\":\"check\",\"params\":[],\"return_type\":null}],\"types\":[],\"reexports\":[]} -->";

        let result = replace_snapshot_section(content, new_snap);
        assert!(result.contains("\"check\""));
        assert!(result.contains("## Histórico de Revisões"));
        assert!(!result.contains("\"functions\":[]"));
    }

    #[test]
    fn inserts_before_historico_when_no_snapshot() {
        let content = "# Prompt\nSome text.\n\n---\n\n## Histórico de Revisões\n| Data |\n";
        let result = replace_snapshot_section(content, SNAPSHOT);
        let snap_pos = result.find("## Interface Snapshot").unwrap();
        let hist_pos = result.find("## Histórico").unwrap();
        assert!(snap_pos < hist_pos, "snapshot should appear before Histórico");
    }

    #[test]
    fn appends_when_no_snapshot_and_no_historico() {
        let content = "# Prompt\nSome text.\n";
        let result = replace_snapshot_section(content, SNAPSHOT);
        assert!(result.ends_with('\n'));
        assert!(result.contains("## Interface Snapshot"));
    }

    #[test]
    fn write_snapshot_round_trips() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("foo.md");
        std::fs::write(&path, "# Prompt\nContent.\n").unwrap();
        write_snapshot(&path, SNAPSHOT).unwrap();
        let out = std::fs::read_to_string(&path).unwrap();
        assert!(out.contains("crystalline-snapshot:"));
    }
}
