//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/infra/config-init.md
//! @prompt-hash fae97064
//! @layer L3
//! @updated 2026-08-26

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::entities::config_init::{
    ConfigDestinationState, DetectedTopology, InventoryEntry, PhysicalEntryKind, UnsafeReason,
};
use crate::infra::config::CrystallineConfig;

pub struct RootInventory {
    pub destination: ConfigDestinationState,
    pub entries: Vec<InventoryEntry>,
}

pub fn inspect_root(root: &Path) -> Result<RootInventory, String> {
    let root_meta = match fs::symlink_metadata(root) {
        Ok(meta) => meta,
        Err(error) => return Err(root_metadata_error(&error)),
    };
    if root_meta.file_type().is_symlink() || !root_meta.is_dir() {
        return Err("project root is not a physical directory".to_string());
    }

    let destination_path = root.join("crystalline.toml");
    let destination = match fs::symlink_metadata(&destination_path) {
        Ok(meta) if meta.is_file() => ConfigDestinationState::Present,
        Ok(_) => ConfigDestinationState::Unsafe(UnsafeReason(
            "config destination is not a regular file".to_string(),
        )),
        Err(error) => destination_error(&error)?,
    };
    if destination != ConfigDestinationState::Absent {
        return Ok(RootInventory {
            destination,
            entries: Vec::new(),
        });
    }

    let iterator = fs::read_dir(root).map_err(|error| inventory_error(&error))?;
    let mut entries = Vec::new();
    for item in iterator {
        let item = item.map_err(|error| format!("cannot inventory project root: {error}"))?;
        let kind = item
            .file_type()
            .map_err(|error| format!("cannot inventory project root: {error}"))?;
        entries.push(InventoryEntry {
            name_bytes: os_bytes(item.file_name()),
            kind: if kind.is_dir() {
                PhysicalEntryKind::Directory
            } else {
                PhysicalEntryKind::Other
            },
        });
    }
    entries.sort_by(|left, right| left.name_bytes.cmp(&right.name_bytes));
    Ok(RootInventory {
        destination,
        entries,
    })
}

fn root_metadata_error(error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::NotFound => "project root does not exist".to_string(),
        std::io::ErrorKind::PermissionDenied => "project root is not accessible".to_string(),
        _ => format!("cannot inventory project root: {error}"),
    }
}

fn destination_error(error: &std::io::Error) -> Result<ConfigDestinationState, String> {
    match error.kind() {
        std::io::ErrorKind::NotFound => Ok(ConfigDestinationState::Absent),
        std::io::ErrorKind::PermissionDenied => Err("project root is not accessible".to_string()),
        _ => Err(format!("cannot inventory project root: {error}")),
    }
}

fn inventory_error(error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::PermissionDenied => "project root is not accessible".to_string(),
        _ => format!("cannot inventory project root: {error}"),
    }
}

#[cfg(unix)]
fn os_bytes(value: std::ffi::OsString) -> Vec<u8> {
    use std::os::unix::ffi::OsStringExt;
    value.into_vec()
}

#[cfg(not(unix))]
fn os_bytes(value: std::ffi::OsString) -> Vec<u8> {
    value.to_string_lossy().into_owned().into_bytes()
}

pub fn serialize_config(topology: &DetectedTopology) -> Vec<u8> {
    let mut output = String::from("[project]\nroot = \".\"\n\n[layers]\n");
    for layer in &topology.layers {
        output.push_str(&format!("{} = \"{}\"\n", layer.key(), layer.directory()));
    }
    output.push_str(
        "\n[excluded]\nbuild = \"target\"\ndeps = \"node_modules\"\nvcs = \".git\"\nvenv = \".venv\"\npycache = \"__pycache__\"\n\n[analysis.lineage]\nstrict_directories = [",
    );
    for (index, layer) in topology.layers.iter().enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        output.push('"');
        output.push_str(layer.directory());
        output.push('"');
    }
    output.push_str("]\n");
    output.into_bytes()
}

pub fn publish_config(root: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    let temporary = temporary_path(root)?;
    publish_config_at(root, bytes, &temporary)
}

fn publish_config_at(root: &Path, bytes: &[u8], temporary: &Path) -> Result<PathBuf, String> {
    let destination = root.join("crystalline.toml");
    let mut owns_temporary = false;

    let operation = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| format!("cannot create temporary config: {error}"))?;
        owns_temporary = true;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|error| format!("cannot write temporary config: {error}"))?;
        drop(file);
        CrystallineConfig::load(&temporary)
            .map_err(|error| format!("generated config failed validation: {error}"))?;
        if let Err(error) = fs::hard_link(&temporary, &destination) {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                return Err("config destination appeared concurrently".to_string());
            }
            return Err(format!("cannot publish config: {error}"));
        }
        fs::remove_file(&temporary)
            .map_err(|error| format!("cannot clean temporary config: {error}"))?;
        Ok(destination.clone())
    })();

    if operation.is_err() && owns_temporary {
        if let Err(error) = fs::remove_file(&temporary) {
            if cleanup_failure_is_reportable(error.kind()) {
                return Err(format!("cannot clean temporary config: {error}"));
            }
        }
    }
    operation
}

fn temporary_path(root: &Path) -> Result<PathBuf, String> {
    let mut random = [0u8; 16];
    getrandom::fill(&mut random)
        .map_err(|error| format!("cannot create temporary config: {error}"))?;
    let token: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(root.join(format!(".crystalline.toml.{token}.tmp")))
}

fn cleanup_failure_is_reportable(kind: std::io::ErrorKind) -> bool {
    kind != std::io::ErrorKind::NotFound
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::config_init::DetectedLayer;

    fn minimal() -> DetectedTopology {
        DetectedTopology {
            layers: vec![DetectedLayer::L0, DetectedLayer::L1],
        }
    }

    #[test]
    fn concurrent_publish_never_replaces_the_winner() {
        let root = tempfile::tempdir().unwrap();
        let bytes = serialize_config(&minimal());
        std::thread::scope(|scope| {
            let first = scope.spawn(|| publish_config(root.path(), &bytes));
            let second = scope.spawn(|| publish_config(root.path(), &bytes));
            let results = [first.join().unwrap(), second.join().unwrap()];
            assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
            assert_eq!(
                results
                    .iter()
                    .filter(|result| {
                        result.as_ref().err().map(String::as_str)
                            == Some("config destination appeared concurrently")
                    })
                    .count(),
                1
            );
        });
        assert_eq!(
            fs::read(root.path().join("crystalline.toml")).unwrap(),
            bytes
        );
        assert!(fs::read_dir(root.path()).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")));
    }

    #[test]
    fn temporary_name_collision_never_removes_the_preexisting_entry() {
        let root = tempfile::tempdir().unwrap();
        let temporary = root.path().join(".crystalline.toml.collision.tmp");
        fs::write(&temporary, b"belongs to another operation").unwrap();

        let error =
            publish_config_at(root.path(), &serialize_config(&minimal()), &temporary).unwrap_err();

        assert!(error.starts_with("cannot create temporary config: "));
        assert_eq!(
            fs::read(&temporary).unwrap(),
            b"belongs to another operation"
        );
        assert!(!root.path().join("crystalline.toml").exists());
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_unrelated_entry_is_preserved_without_lossy_collision() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("00_nucleo")).unwrap();
        fs::create_dir(root.path().join("01_core")).unwrap();
        fs::create_dir(root.path().join(OsString::from_vec(vec![0xff]))).unwrap();
        let inventory = inspect_root(root.path()).unwrap();
        assert!(inventory
            .entries
            .iter()
            .any(|entry| entry.name_bytes == vec![0xff]));
    }

    #[test]
    fn io_error_projection_is_total_and_context_sensitive() {
        let missing = std::io::Error::from(std::io::ErrorKind::NotFound);
        let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        let other = std::io::Error::from(std::io::ErrorKind::InvalidData);

        assert_eq!(root_metadata_error(&missing), "project root does not exist");
        assert_eq!(
            root_metadata_error(&denied),
            "project root is not accessible"
        );
        assert!(root_metadata_error(&other).starts_with("cannot inventory project root: "));
        assert_eq!(
            destination_error(&missing),
            Ok(ConfigDestinationState::Absent)
        );
        assert_eq!(
            destination_error(&denied),
            Err("project root is not accessible".to_string())
        );
        assert!(destination_error(&other)
            .unwrap_err()
            .starts_with("cannot inventory project root: "));
        assert_eq!(inventory_error(&denied), "project root is not accessible");
        assert!(inventory_error(&other).starts_with("cannot inventory project root: "));
        assert!(!cleanup_failure_is_reportable(std::io::ErrorKind::NotFound));
        assert!(cleanup_failure_is_reportable(
            std::io::ErrorKind::PermissionDenied
        ));
    }
}
