//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/prompt-reader.md
//! @prompt-hash 53520eff
//! @layer L1
//! @updated 2026-03-13

/// Contract for reading prompt files from 00_nucleo/.
/// L3 implements (FsPromptReader with std::fs + sha2).
/// L1 declares — zero I/O, zero sha2.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptPresence {
    Present,
    Missing,
    InfrastructureFailure { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptHashResolution {
    Hash(String),
    Missing,
    InfrastructureFailure { reason: String },
}

/// Used by V1 and V5 without collapsing an infrastructure failure into absence.
pub trait PromptReader {
    fn presence(&self, prompt_path: &str) -> PromptPresence;
    fn read_hash(&self, prompt_path: &str) -> PromptHashResolution;
}

/// Resolves one header in the required order and promotes hash I/O failures to
/// prompt access failures so they cannot be mistaken for V5 drift.
pub fn resolve_prompt(
    reader: &impl PromptReader,
    prompt_path: &str,
) -> (PromptPresence, Option<String>) {
    let presence = reader.presence(prompt_path);
    if presence != PromptPresence::Present {
        return (presence, None);
    }
    match reader.read_hash(prompt_path) {
        PromptHashResolution::Hash(hash) => (PromptPresence::Present, Some(hash)),
        PromptHashResolution::Missing => (PromptPresence::Missing, None),
        PromptHashResolution::InfrastructureFailure { reason } => {
            (PromptPresence::InfrastructureFailure { reason }, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPromptReader {
        hash: PromptHashResolution,
        presence: PromptPresence,
    }

    impl PromptReader for MockPromptReader {
        fn read_hash(&self, _prompt_path: &str) -> PromptHashResolution {
            self.hash.clone()
        }
        fn presence(&self, _prompt_path: &str) -> PromptPresence {
            self.presence.clone()
        }
    }

    #[test]
    fn read_hash_returns_eight_char_hex() {
        let reader = MockPromptReader {
            hash: PromptHashResolution::Hash("a3f8c2d1".to_string()),
            presence: PromptPresence::Present,
        };
        let hash = reader.read_hash("prompts/auth.md");
        assert_eq!(hash, PromptHashResolution::Hash("a3f8c2d1".to_string()));
    }

    #[test]
    fn read_hash_returns_none_when_file_missing() {
        let reader = MockPromptReader {
            hash: PromptHashResolution::Missing,
            presence: PromptPresence::Missing,
        };
        assert_eq!(
            reader.read_hash("prompts/missing.md"),
            PromptHashResolution::Missing
        );
    }

    #[test]
    fn exists_returns_true_when_present() {
        let reader = MockPromptReader {
            hash: PromptHashResolution::Hash("00000000".to_string()),
            presence: PromptPresence::Present,
        };
        assert_eq!(reader.presence("prompts/auth.md"), PromptPresence::Present);
    }

    #[test]
    fn exists_returns_false_when_absent() {
        let reader = MockPromptReader {
            hash: PromptHashResolution::Missing,
            presence: PromptPresence::Missing,
        };
        assert_eq!(
            reader.presence("prompts/missing.md"),
            PromptPresence::Missing
        );
    }

    #[test]
    fn v5_uses_mock_without_disk_access() {
        // V5 operates purely on hashes — no I/O when using mock
        let reader = MockPromptReader {
            hash: PromptHashResolution::Hash("b9e4f7a2".to_string()),
            presence: PromptPresence::Present,
        };
        let declared = "a3f8c2d1";
        let PromptHashResolution::Hash(current) = reader.read_hash("prompts/linter-core.md") else {
            panic!("hash expected")
        };
        assert_ne!(declared, current); // drift detected
    }

    #[test]
    fn resolution_reads_hash_only_for_present_prompt() {
        let present = MockPromptReader {
            hash: PromptHashResolution::Hash("a3f8c2d1".into()),
            presence: PromptPresence::Present,
        };
        assert_eq!(
            resolve_prompt(&present, "prompts/core.md"),
            (PromptPresence::Present, Some("a3f8c2d1".into()))
        );

        let failed = MockPromptReader {
            hash: PromptHashResolution::Hash("must-not-escape".into()),
            presence: PromptPresence::InfrastructureFailure {
                reason: "root".into(),
            },
        };
        assert_eq!(
            resolve_prompt(&failed, "prompts/core.md"),
            (
                PromptPresence::InfrastructureFailure {
                    reason: "root".into()
                },
                None
            )
        );
    }
}
