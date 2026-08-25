use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST: &str = "00_nucleo/assessments/0034-manifest-individualizacao.tsv";

#[derive(Debug)]
struct Row {
    old_prompt: String,
    consumer: String,
    owner_prompt: String,
    nuclei: String,
    classifier_hash: String,
    state: String,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rows() -> Vec<Row> {
    let text = fs::read_to_string(root().join(MANIFEST)).expect("manifest 0034 must exist");
    let mut lines = text.lines();
    assert_eq!(
        lines.next(),
        Some("old_prompt\tconsumer\towner_prompt\tnuclei_csv\tclassification_sha256\tstate")
    );
    lines
        .map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(fields.len(), 6, "invalid manifest row: {line}");
            Row {
                old_prompt: fields[0].into(),
                consumer: fields[1].into(),
                owner_prompt: fields[2].into(),
                nuclei: fields[3].into(),
                classifier_hash: fields[4].into(),
                state: fields[5].into(),
            }
        })
        .collect()
}

fn classifier_path(old_prompt: &str) -> PathBuf {
    let name = Path::new(old_prompt).file_name().expect("prompt filename");
    root().join("00_nucleo/assessments/0034-groups").join(name)
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[test]
fn b1_manifest_is_total_unique_and_hash_pinned() {
    let rows = rows();
    assert_eq!(rows.len(), 44);

    let old: BTreeSet<_> = rows.iter().map(|row| &row.old_prompt).collect();
    let consumers: BTreeSet<_> = rows.iter().map(|row| &row.consumer).collect();
    let owners: BTreeSet<_> = rows.iter().map(|row| &row.owner_prompt).collect();
    assert_eq!(old.len(), 13);
    assert_eq!(consumers.len(), 44);
    assert_eq!(owners.len(), 44);

    for row in rows {
        assert_eq!(row.state, "CLASSIFIED");
        let bytes = fs::read(classifier_path(&row.old_prompt)).expect("classifier must exist");
        assert_eq!(sha256(&bytes), row.classifier_hash);
    }
}

#[test]
fn b2_every_split_is_authorized_by_its_semantic_classifier() {
    for row in rows() {
        let text = fs::read_to_string(classifier_path(&row.old_prompt)).unwrap();
        assert!(text.contains("**Estado:** CLASSIFIED"));
        assert!(
            text.contains(&format!("`{}`", row.consumer)),
            "consumer absent from classifier: {}",
            row.consumer
        );
    }
}

#[test]
fn b3_projected_graph_is_bijective_and_follows_the_frozen_lots() {
    let rows = rows();
    assert!(rows.iter().all(|row| row.nuclei.is_empty()));

    let lots = [
        [
            "citation-freshness.md",
            "prompt-reader.md",
            "prompt-snapshot-reader.md",
            "external-type-in-contract.md",
            "sarif-formatter.md",
            "segregated-materialization.md",
            "unsourced-constant.md",
        ]
        .as_slice(),
        [
            "violation-types.md",
            "file-walker.md",
            "fix-hashes.md",
            "wildcard-saturation.md",
        ]
        .as_slice(),
        ["refinement-validator.md", "linter-core.md"].as_slice(),
    ];
    let expected = [6, 2, 0];
    let mut remaining: BTreeMap<String, usize> = BTreeMap::new();
    for row in &rows {
        *remaining.entry(row.old_prompt.clone()).or_default() += 1;
    }
    assert_eq!(remaining.values().filter(|count| **count > 1).count(), 13);

    for (lot, expected_shared) in lots.iter().zip(expected) {
        for name in *lot {
            let key = remaining
                .keys()
                .find(|path| path.ends_with(name))
                .cloned()
                .expect("lot must name one frozen group");
            remaining.insert(key, 1);
        }
        assert_eq!(
            remaining.values().filter(|count| **count > 1).count(),
            expected_shared
        );
    }

    let projected_owners: BTreeSet<_> = rows.iter().map(|row| &row.owner_prompt).collect();
    assert_eq!(projected_owners.len(), rows.len());
}
