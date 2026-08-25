//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/nucleus-integrity.md
//! @prompt-hash 7f3c396b
//! @layer L1

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NucleusGraphEntry {
    pub path: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptNucleusUsage {
    pub prompt: String,
    pub nucleus: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NucleusFinding {
    pub path: String,
    pub message: String,
}

pub fn check_graph(
    nodes: &[NucleusGraphEntry],
    usages: &[PromptNucleusUsage],
) -> Vec<NucleusFinding> {
    let graph: BTreeMap<&str, BTreeSet<&str>> = nodes
        .iter()
        .map(|node| {
            (
                node.path.as_str(),
                node.dependencies.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    let used: BTreeSet<&str> = usages.iter().map(|usage| usage.nucleus.as_str()).collect();
    let mut findings = Vec::new();

    for (path, dependencies) in &graph {
        if !used.contains(path)
            && !nodes
                .iter()
                .any(|node| node.dependencies.iter().any(|d| d == path))
        {
            findings.push(NucleusFinding {
                path: (*path).into(),
                message: format!("orphan nucleus {path}"),
            });
        }
        for dependency in dependencies {
            if !graph.contains_key(dependency) {
                findings.push(NucleusFinding {
                    path: (*path).into(),
                    message: format!("missing nucleus dependency {dependency}"),
                });
            }
        }
    }

    fn visit<'a>(
        node: &'a str,
        graph: &BTreeMap<&'a str, BTreeSet<&'a str>>,
        visiting: &mut BTreeSet<&'a str>,
        visited: &mut BTreeSet<&'a str>,
        cycles: &mut BTreeSet<String>,
    ) {
        if visiting.contains(node) {
            cycles.insert(node.to_owned());
            return;
        }
        if !visited.insert(node) {
            return;
        }
        visiting.insert(node);
        if let Some(dependencies) = graph.get(node) {
            for dependency in dependencies.iter().filter(|d| graph.contains_key(*d)) {
                visit(dependency, graph, visiting, visited, cycles);
            }
        }
        visiting.remove(node);
    }

    let mut visited = BTreeSet::new();
    let mut cycles = BTreeSet::new();
    for node in graph.keys() {
        visit(
            node,
            &graph,
            &mut BTreeSet::new(),
            &mut visited,
            &mut cycles,
        );
    }
    for path in cycles {
        findings.push(NucleusFinding {
            path: path.clone(),
            message: format!("nucleus dependency cycle includes {path}"),
        });
    }
    findings.sort();
    findings.dedup();
    findings
}
