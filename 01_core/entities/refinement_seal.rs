//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/segregated-materialization.md
//! @prompt-hash 4f6bc4f5
//! @layer L1
//! @updated 2026-08-24
//!
//! Pure policy for the segregated refinement materialization pilot.

use crate::entities::refinement::RefinementVerdict;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OracleKind {
    Positive,
    Negative,
    Unknown,
}

impl OracleKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerdictName {
    Preserved,
    Violated,
    Unknown,
}

impl VerdictName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Preserved => "PRESERVED",
            Self::Violated => "VIOLATED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

impl From<&RefinementVerdict> for VerdictName {
    fn from(value: &RefinementVerdict) -> Self {
        match value {
            RefinementVerdict::Preserved => Self::Preserved,
            RefinementVerdict::Violated { .. } => Self::Violated,
            RefinementVerdict::Unknown { .. } => Self::Unknown,
        }
    }
}

pub fn accepts(kind: OracleKind, verdict: &RefinementVerdict) -> bool {
    match (kind, verdict) {
        (OracleKind::Positive, RefinementVerdict::Preserved) => true,
        (OracleKind::Negative, RefinementVerdict::Violated { inconclusive, .. }) => {
            inconclusive.is_empty()
        }
        (OracleKind::Unknown, RefinementVerdict::Unknown { .. }) => true,
        (OracleKind::Positive, RefinementVerdict::Violated { .. })
        | (OracleKind::Positive, RefinementVerdict::Unknown { .. })
        | (OracleKind::Negative, RefinementVerdict::Preserved)
        | (OracleKind::Negative, RefinementVerdict::Unknown { .. })
        | (OracleKind::Unknown, RefinementVerdict::Preserved)
        | (OracleKind::Unknown, RefinementVerdict::Violated { .. }) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::refinement::{Inconclusive, UnknownReason};

    #[test]
    fn negative_rejects_violation_with_inconclusive_results() {
        let clean = RefinementVerdict::Violated {
            witnesses: Vec::new(),
            inconclusive: Vec::new(),
        };
        let mixed = RefinementVerdict::Violated {
            witnesses: Vec::new(),
            inconclusive: vec![Inconclusive {
                contract_id: "contract".to_string(),
                relation: "preserve".to_string(),
                observable: "field".to_string(),
                reason: UnknownReason::MissingObservable,
            }],
        };
        assert!(accepts(OracleKind::Negative, &clean));
        assert!(!accepts(OracleKind::Negative, &mixed));
    }
}
