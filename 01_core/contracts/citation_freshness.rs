//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/citation-freshness.md
//! @prompt-hash PENDING
//! @layer L1
//! @updated 2026-08-24

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationStaleReason {
    MissingFile,
    InvalidLine,
    EmptyLine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationUnknownReason {
    OutsideRoot,
    Symlink,
    InvalidRoot,
    Io,
    InvalidUtf8,
    BudgetExceeded,
    ConcurrentMutation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationFreshness {
    Valid,
    Stale(CitationStaleReason),
    Unknown(CitationUnknownReason),
}

pub trait CitationFreshnessResolver {
    fn resolve(&self, path: &str, line: usize) -> CitationFreshness;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UnknownCitationFreshness;

impl CitationFreshnessResolver for UnknownCitationFreshness {
    fn resolve(&self, _path: &str, _line: usize) -> CitationFreshness {
        CitationFreshness::Unknown(CitationUnknownReason::Io)
    }
}
