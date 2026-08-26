//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/entities/hash-pair.md
//! @prompt-hash 00000000
//! @layer L1
//! @updated 2026-08-25

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairSnapshot {
    pub source_bytes: Vec<u8>,
    pub prompt_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BijectivePair {
    pub source_path: PathBuf,
    pub prompt_path: String,
    pub old_prompt_hash: String,
    pub new_prompt_hash: String,
    pub new_source_hash: String,
    pub new_source_bytes: Vec<u8>,
    pub new_prompt_bytes: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_preserves_bytes_clone_and_equality() {
        let snapshot = PairSnapshot {
            source_bytes: vec![0, 0xff, b'\n'],
            prompt_bytes: "núcleo".as_bytes().to_vec(),
        };
        assert_eq!(snapshot.clone(), snapshot);
    }

    #[test]
    fn pair_preserves_paths_hashes_and_payloads() {
        let pair = BijectivePair {
            source_path: PathBuf::from("01_core/α.rs"),
            prompt_path: "00_nucleo/prompts/α.md".to_owned(),
            old_prompt_hash: "00000000".to_owned(),
            new_prompt_hash: "11111111".to_owned(),
            new_source_hash: "22222222".to_owned(),
            new_source_bytes: vec![0, 1, 2],
            new_prompt_bytes: vec![0xff, 0],
        };
        assert_eq!(pair.clone(), pair);
        assert_eq!(pair.source_path, PathBuf::from("01_core/α.rs"));
        assert_eq!(pair.new_prompt_bytes, vec![0xff, 0]);
    }
}
