use crystalline_lint::shell::fix_hashes::{
    execute_bijective, plan_bijective, BijectivePair, FixBatchResult, PairSnapshot,
    TransactionalHashRewriter,
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Default)]
struct Spy {
    state: RefCell<BTreeMap<PathBuf, PairSnapshot>>,
    writes: RefCell<usize>,
}
impl TransactionalHashRewriter for Spy {
    fn preflight(&self, p: &BijectivePair) -> Result<PairSnapshot, String> {
        self.state
            .borrow()
            .get(&p.source_path)
            .cloned()
            .ok_or("missing".into())
    }
    fn apply_pair(&self, p: &BijectivePair) -> Result<(), String> {
        *self.writes.borrow_mut() += 1;
        self.state.borrow_mut().insert(
            p.source_path.clone(),
            PairSnapshot {
                source_bytes: p.new_source_bytes.clone(),
                prompt_bytes: p.new_prompt_bytes.clone(),
            },
        );
        Ok(())
    }
    fn rollback_pair(&self, p: &BijectivePair, s: &PairSnapshot) -> Result<(), String> {
        self.state
            .borrow_mut()
            .insert(p.source_path.clone(), s.clone());
        Ok(())
    }
    fn validate_pair(&self, p: &BijectivePair) -> Result<(), String> {
        let s = self.state.borrow();
        let a = s.get(&p.source_path).ok_or("missing")?;
        (a.source_bytes == p.new_source_bytes && a.prompt_bytes == p.new_prompt_bytes)
            .then_some(())
            .ok_or("stale".into())
    }
}
fn pair() -> BijectivePair {
    BijectivePair {
        source_path: PathBuf::from("code.rs"),
        prompt_path: "prompt.md".into(),
        old_prompt_hash: "old".into(),
        new_prompt_hash: "new".into(),
        new_source_hash: "code".into(),
        new_source_bytes: b"source+nucleus".to_vec(),
        new_prompt_bytes: b"prompt+pin".to_vec(),
    }
}

#[test]
fn dry_run_and_real_share_preflighted_plan() {
    let p = pair();
    let s = Spy::default();
    s.state.borrow_mut().insert(
        p.source_path.clone(),
        PairSnapshot {
            source_bytes: b"old".to_vec(),
            prompt_bytes: b"old".to_vec(),
        },
    );
    let plan = plan_bijective(&[p], &s).unwrap();
    assert!(matches!(
        execute_bijective(&plan, &s, true),
        FixBatchResult::DryRun { .. }
    ));
    assert_eq!(*s.writes.borrow(), 0);
    assert!(matches!(
        execute_bijective(&plan, &s, false),
        FixBatchResult::Applied { .. }
    ));
}

#[test]
fn incomplete_preflight_writes_nothing() {
    let s = Spy::default();
    assert!(plan_bijective(&[pair()], &s).is_err());
    assert_eq!(*s.writes.borrow(), 0);
}
