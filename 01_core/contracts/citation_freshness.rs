//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/citation-freshness.md
//! @prompt-hash 84133c6d
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_modalities_and_reasons_preserve_identity() {
        let stale = [
            CitationStaleReason::MissingFile,
            CitationStaleReason::InvalidLine,
            CitationStaleReason::EmptyLine,
        ];
        let unknown = [
            CitationUnknownReason::OutsideRoot,
            CitationUnknownReason::Symlink,
            CitationUnknownReason::InvalidRoot,
            CitationUnknownReason::Io,
            CitationUnknownReason::InvalidUtf8,
            CitationUnknownReason::BudgetExceeded,
            CitationUnknownReason::ConcurrentMutation,
        ];

        assert_eq!(CitationFreshness::Valid.clone(), CitationFreshness::Valid);
        for reason in stale {
            let value = CitationFreshness::Stale(reason);
            assert_eq!(value.clone(), value);
            assert_ne!(value, CitationFreshness::Valid);
        }
        for reason in unknown {
            let value = CitationFreshness::Unknown(reason);
            assert_eq!(value.clone(), value);
            assert_ne!(value, CitationFreshness::Valid);
        }
    }

    #[test]
    fn default_resolver_is_total_and_fail_closed_without_io() {
        let resolver = UnknownCitationFreshness;
        for (path, line) in [("", 0), ("../hostile", usize::MAX), ("núcleo/α.md", 7)] {
            assert_eq!(
                resolver.resolve(path, line),
                CitationFreshness::Unknown(CitationUnknownReason::Io)
            );
        }
    }
}
