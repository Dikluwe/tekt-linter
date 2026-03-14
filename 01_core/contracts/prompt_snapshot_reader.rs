//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/prompt-snapshot-reader.md
//! @prompt-hash 00000000
//! @layer L1
//! @updated 2026-03-14

use crate::entities::parsed_file::PublicInterface;

/// L1 contract for reading and serializing the Interface Snapshot stored in
/// a L0 prompt file. L3 provides the concrete I/O implementation.
/// L1 never calls serde_json or std::fs directly.
pub trait PromptSnapshotReader {
    /// Read and deserialize the interface snapshot from the prompt file.
    /// Returns `PublicInterface<'static>` via Box::leak — snapshot data is
    /// parsed from the prompt file on disk and cannot borrow from SourceFile.
    /// Returns None if:
    /// - the file does not exist
    /// - the file has no `## Interface Snapshot` section
    /// - the snapshot JSON is malformed
    fn read_snapshot(&self, prompt_path: &str) -> Option<PublicInterface<'static>>;

    /// Serialize a PublicInterface to the canonical snapshot section format.
    /// Used by `--update-snapshot` to write back to the prompt file.
    fn serialize_snapshot(&self, interface: &PublicInterface<'_>) -> String;
}
