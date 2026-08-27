//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/entities/config-init.md
//! @prompt-hash 224171f6
//! @layer L1
//! @updated 2026-08-26

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigDestinationState {
    Absent,
    Present,
    Unsafe(UnsafeReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsafeReason(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationFailure(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalEntryKind {
    Directory,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryEntry {
    pub name_bytes: Vec<u8>,
    pub kind: PhysicalEntryKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedLayer {
    L0,
    L1,
    L2,
    L3,
    L4,
    Lab,
}

impl DetectedLayer {
    pub fn key(self) -> &'static str {
        match self {
            Self::L0 => "L0",
            Self::L1 => "L1",
            Self::L2 => "L2",
            Self::L3 => "L3",
            Self::L4 => "L4",
            Self::Lab => "lab",
        }
    }

    pub fn directory(self) -> &'static str {
        match self {
            Self::L0 => "00_nucleo",
            Self::L1 => "01_core",
            Self::L2 => "02_shell",
            Self::L3 => "03_infra",
            Self::L4 => "04_wiring",
            Self::Lab => "lab",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedTopology {
    pub layers: Vec<DetectedLayer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyAmbiguity {
    CaseAmbiguous(Vec<Vec<u8>>),
    RequiredMissing(Vec<DetectedLayer>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyDecision {
    Recognized(DetectedTopology),
    Unknown(TopologyAmbiguity),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitPlan {
    Create(DetectedTopology),
    RefusePresent,
    RefuseUnsafe(UnsafeReason),
    RefuseUnknown(TopologyAmbiguity),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitReceipt {
    Created,
    RefusedPresent,
    RefusedUnsafe(UnsafeReason),
    RefusedUnknown(TopologyAmbiguity),
    PublicationFailed(PublicationFailure),
}

const CANONICAL: [DetectedLayer; 6] = [
    DetectedLayer::L0,
    DetectedLayer::L1,
    DetectedLayer::L2,
    DetectedLayer::L3,
    DetectedLayer::L4,
    DetectedLayer::Lab,
];

pub fn detect_topology(entries: &[InventoryEntry]) -> TopologyDecision {
    let mut entries = entries.to_vec();
    entries.sort_by(|left, right| left.name_bytes.cmp(&right.name_bytes));

    let mut ambiguous = Vec::new();
    for entry in &entries {
        for layer in CANONICAL {
            let canonical = layer.directory().as_bytes();
            if entry.name_bytes != canonical && entry.name_bytes.eq_ignore_ascii_case(canonical) {
                ambiguous.push(entry.name_bytes.clone());
            }
        }
    }
    ambiguous.sort();
    ambiguous.dedup();
    if !ambiguous.is_empty() {
        return TopologyDecision::Unknown(TopologyAmbiguity::CaseAmbiguous(ambiguous));
    }

    let layers: Vec<_> = CANONICAL
        .into_iter()
        .filter(|layer| {
            entries.iter().any(|entry| {
                entry.kind == PhysicalEntryKind::Directory
                    && entry.name_bytes == layer.directory().as_bytes()
            })
        })
        .collect();
    let missing: Vec<_> = [DetectedLayer::L0, DetectedLayer::L1]
        .into_iter()
        .filter(|required| !layers.contains(required))
        .collect();
    if missing.is_empty() {
        TopologyDecision::Recognized(DetectedTopology { layers })
    } else {
        TopologyDecision::Unknown(TopologyAmbiguity::RequiredMissing(missing))
    }
}

pub fn decide_init(
    destination: ConfigDestinationState,
    topology: Option<TopologyDecision>,
) -> InitPlan {
    match destination {
        ConfigDestinationState::Present => InitPlan::RefusePresent,
        ConfigDestinationState::Unsafe(reason) => InitPlan::RefuseUnsafe(reason),
        ConfigDestinationState::Absent => {
            let Some(topology) = topology else {
                return InitPlan::RefuseUnsafe(UnsafeReason(
                    "project topology was not inventoried".to_string(),
                ));
            };
            match topology {
                TopologyDecision::Recognized(topology) => InitPlan::Create(topology),
                TopologyDecision::Unknown(reason) => InitPlan::RefuseUnknown(reason),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory(name: &[u8]) -> InventoryEntry {
        InventoryEntry {
            name_bytes: name.to_vec(),
            kind: PhysicalEntryKind::Directory,
        }
    }

    #[test]
    fn topology_is_total_deterministic_and_fail_closed() {
        let recognized = detect_topology(&[
            directory(b"lab"),
            directory(b"01_core"),
            directory(b"irrelevant"),
            directory(b"00_nucleo"),
        ]);
        assert_eq!(
            recognized,
            TopologyDecision::Recognized(DetectedTopology {
                layers: vec![DetectedLayer::L0, DetectedLayer::L1, DetectedLayer::Lab]
            })
        );

        let ambiguous = detect_topology(&[directory(b"01_CORE"), directory(b"00_NUCLEO")]);
        assert_eq!(
            ambiguous,
            TopologyDecision::Unknown(TopologyAmbiguity::CaseAmbiguous(vec![
                b"00_NUCLEO".to_vec(),
                b"01_CORE".to_vec(),
            ]))
        );

        assert_eq!(
            detect_topology(&[directory(b"00_nucleo")]),
            TopologyDecision::Unknown(TopologyAmbiguity::RequiredMissing(vec![DetectedLayer::L1]))
        );
    }

    #[test]
    fn destination_precedence_does_not_consult_topology() {
        assert_eq!(
            decide_init(ConfigDestinationState::Present, None),
            InitPlan::RefusePresent
        );
        assert_eq!(
            decide_init(
                ConfigDestinationState::Unsafe(UnsafeReason("unsafe".to_string())),
                None
            ),
            InitPlan::RefuseUnsafe(UnsafeReason("unsafe".to_string()))
        );
    }
}
