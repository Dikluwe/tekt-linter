use crystalline_lint::infra::nucleus::{
    effective_nucleus_hash, effective_prompt_hash, HashDependency,
};
use sha2::{Digest, Sha256};

#[test]
fn prompt_without_nucleus_is_bit_identical_to_legacy() {
    let prompt = b"# Prompt\nHash do C\xC3\xB3digo: deadbeef\nbody\n";
    let legacy = hex::encode(Sha256::digest(b"# Prompt\nbody\n"))[..8].to_owned();
    assert_eq!(effective_prompt_hash(prompt, &[]).unwrap(), legacy);
}

#[test]
fn dependency_order_is_canonical_and_changes_propagate() {
    let a = HashDependency {
        path: "a.toml".into(),
        digest: [1; 32],
    };
    let b = HashDependency {
        path: "b.toml".into(),
        digest: [2; 32],
    };
    assert_eq!(
        effective_prompt_hash(b"p", &[a.clone(), b.clone()]),
        effective_prompt_hash(b"p", &[b.clone(), a.clone()])
    );
    let changed = HashDependency {
        digest: [3; 32],
        ..a.clone()
    };
    assert_ne!(
        effective_prompt_hash(b"p", &[a]),
        effective_prompt_hash(b"p", &[changed])
    );
}

#[test]
fn nucleus_hash_is_deterministic_and_transitive() {
    let leaf = effective_nucleus_hash(b"leaf", &[]);
    let dep = HashDependency {
        path: "leaf.toml".into(),
        digest: leaf,
    };
    assert_eq!(
        effective_nucleus_hash(b"root", &[dep.clone()]),
        effective_nucleus_hash(b"root", &[dep])
    );
    assert_ne!(
        effective_nucleus_hash(b"root", &[]),
        effective_nucleus_hash(b"root!", &[])
    );
}
