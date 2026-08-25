use crystalline_lint::rules::nucleus_integrity::{
    check_graph, NucleusGraphEntry, PromptNucleusUsage,
};

fn node(path: &str, deps: &[&str]) -> NucleusGraphEntry {
    NucleusGraphEntry {
        path: path.into(),
        dependencies: deps.iter().map(|v| (*v).into()).collect(),
    }
}

#[test]
fn dag_and_shared_consumption_are_valid() {
    let nodes = vec![node("a.tekt", &[]), node("b.tekt", &["a.tekt"])];
    let uses = vec![
        PromptNucleusUsage {
            prompt: "a.md".into(),
            nucleus: "a.tekt".into(),
        },
        PromptNucleusUsage {
            prompt: "b.md".into(),
            nucleus: "a.tekt".into(),
        },
        PromptNucleusUsage {
            prompt: "b.md".into(),
            nucleus: "b.tekt".into(),
        },
    ];
    assert!(check_graph(&nodes, &uses).is_empty());
}

#[test]
fn missing_cycles_and_orphans_are_deterministic() {
    let forward = vec![
        node("z.tekt", &["missing.tekt"]),
        node("a.tekt", &["b.tekt"]),
        node("b.tekt", &["a.tekt"]),
        node("orphan.tekt", &[]),
    ];
    let mut reverse = forward.clone();
    reverse.reverse();
    let a = check_graph(&forward, &[]);
    let b = check_graph(&reverse, &[]);
    assert_eq!(a, b);
    assert!(a.iter().any(|f| f.message.contains("missing.tekt")));
    assert!(a.iter().any(|f| f.message.contains("cycle")));
    assert!(a.iter().any(|f| f.message.contains("orphan.tekt")));
}

#[test]
fn identities_remain_case_sensitive() {
    let nodes = vec![node("A.tekt", &[]), node("a.tekt", &[])];
    let uses = vec![
        PromptNucleusUsage {
            prompt: "x.md".into(),
            nucleus: "A.tekt".into(),
        },
        PromptNucleusUsage {
            prompt: "y.md".into(),
            nucleus: "a.tekt".into(),
        },
    ];
    assert!(check_graph(&nodes, &uses).is_empty());
}
