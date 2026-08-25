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
    let nodes = vec![node("a.toml", &[]), node("b.toml", &["a.toml"])];
    let uses = vec![
        PromptNucleusUsage {
            prompt: "a.md".into(),
            nucleus: "a.toml".into(),
        },
        PromptNucleusUsage {
            prompt: "b.md".into(),
            nucleus: "a.toml".into(),
        },
        PromptNucleusUsage {
            prompt: "b.md".into(),
            nucleus: "b.toml".into(),
        },
    ];
    assert!(check_graph(&nodes, &uses).is_empty());
}

#[test]
fn missing_cycles_and_orphans_are_deterministic() {
    let forward = vec![
        node("z.toml", &["missing.toml"]),
        node("a.toml", &["b.toml"]),
        node("b.toml", &["a.toml"]),
        node("orphan.toml", &[]),
    ];
    let mut reverse = forward.clone();
    reverse.reverse();
    let a = check_graph(&forward, &[]);
    let b = check_graph(&reverse, &[]);
    assert_eq!(a, b);
    assert!(a.iter().any(|f| f.message.contains("missing.toml")));
    assert!(a.iter().any(|f| f.message.contains("cycle")));
    assert!(a.iter().any(|f| f.message.contains("orphan.toml")));
}

#[test]
fn identities_remain_case_sensitive() {
    let nodes = vec![node("A.toml", &[]), node("a.toml", &[])];
    let uses = vec![
        PromptNucleusUsage {
            prompt: "x.md".into(),
            nucleus: "A.toml".into(),
        },
        PromptNucleusUsage {
            prompt: "y.md".into(),
            nucleus: "a.toml".into(),
        },
    ];
    assert!(check_graph(&nodes, &uses).is_empty());
}
