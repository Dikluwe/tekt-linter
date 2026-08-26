//! Gate B3/P0104: contrato transacional e biunívoco de `fix-hashes`.
//!
//! Este gate é deliberadamente escrito contra a seam normativa de P0104, não contra
//! a implementação histórica orientada a consumers. Antes de C, a ausência destes
//! símbolos deve produzir compile-RED causal.

use crystalline_lint::shell::fix_hashes::{
    execute_bijective, plan_bijective, validate_bijective, ApplyFailure, BijectivePair,
    FixBatchError, FixBatchPlan, FixBatchResult, PairSnapshot, TransactionalHashRewriter,
};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct Spy {
    state: RefCell<BTreeMap<PathBuf, PairSnapshot>>,
    calls: RefCell<Vec<String>>,
    fail_preflight: RefCell<Option<PathBuf>>,
    fail_apply_at: Cell<Option<usize>>,
    fail_rollback: Cell<bool>,
}

impl Spy {
    fn seeded(pairs: &[BijectivePair]) -> Self {
        let mut state = BTreeMap::new();
        for pair in pairs {
            state.insert(
                pair.source_path.clone(),
                PairSnapshot {
                    source_bytes: format!("source:{}", pair.source_path.display()).into_bytes(),
                    prompt_bytes: format!("prompt:{}", pair.prompt_path).into_bytes(),
                },
            );
        }
        Self {
            state: RefCell::new(state),
            ..Self::default()
        }
    }

    fn writes(&self) -> Vec<String> {
        self.calls
            .borrow()
            .iter()
            .filter(|call| call.starts_with("apply:") || call.starts_with("rollback:"))
            .cloned()
            .collect()
    }
}

impl TransactionalHashRewriter for Spy {
    fn preflight(&self, pair: &BijectivePair) -> Result<PairSnapshot, String> {
        self.calls
            .borrow_mut()
            .push(format!("preflight:{}", pair.source_path.display()));
        if self.fail_preflight.borrow().as_ref() == Some(&pair.source_path) {
            return Err("metadata ausente".into());
        }
        self.state
            .borrow()
            .get(&pair.source_path)
            .cloned()
            .ok_or_else(|| "par desconhecido".into())
    }

    fn apply_pair(&self, pair: &BijectivePair) -> Result<(), String> {
        let index = self
            .calls
            .borrow()
            .iter()
            .filter(|call| call.starts_with("apply:"))
            .count();
        self.calls
            .borrow_mut()
            .push(format!("apply:{}", pair.source_path.display()));
        if self.fail_apply_at.get() == Some(index) {
            return Err("falha injetada".into());
        }
        self.state.borrow_mut().insert(
            pair.source_path.clone(),
            PairSnapshot {
                source_bytes: pair.new_source_bytes.clone(),
                prompt_bytes: pair.new_prompt_bytes.clone(),
            },
        );
        Ok(())
    }

    fn rollback_pair(&self, pair: &BijectivePair, snapshot: &PairSnapshot) -> Result<(), String> {
        self.calls
            .borrow_mut()
            .push(format!("rollback:{}", pair.source_path.display()));
        if self.fail_rollback.get() {
            return Err("rollback rejeitado".into());
        }
        self.state
            .borrow_mut()
            .insert(pair.source_path.clone(), snapshot.clone());
        Ok(())
    }

    fn validate_pair(&self, pair: &BijectivePair) -> Result<(), String> {
        self.calls
            .borrow_mut()
            .push(format!("validate:{}", pair.source_path.display()));
        let state = self.state.borrow();
        let actual = state.get(&pair.source_path).ok_or("par ausente")?;
        if actual.source_bytes == pair.new_source_bytes
            && actual.prompt_bytes == pair.new_prompt_bytes
        {
            Ok(())
        } else {
            Err("paridade bidirecional ausente".into())
        }
    }
}

fn pair(source: &str, prompt: &str) -> BijectivePair {
    BijectivePair {
        source_path: PathBuf::from(source),
        prompt_path: prompt.into(),
        old_prompt_hash: "old-a".into(),
        new_prompt_hash: "new-a".into(),
        new_source_hash: "new-b".into(),
        new_source_bytes: format!("new-source:{source}").into_bytes(),
        new_prompt_bytes: format!("new-prompt:{prompt}:{source}").into_bytes(),
    }
}

#[test]
fn shared_prompt_blocks_the_entire_batch_before_first_write() {
    let pairs = vec![
        pair("01_core/a.rs", "00_nucleo/prompts/shared.md"),
        pair("01_core/b.rs", "00_nucleo/prompts/shared.md"),
        pair("01_core/ok.rs", "00_nucleo/prompts/ok.md"),
    ];
    let spy = Spy::seeded(&pairs);

    let error = plan_bijective(&pairs, &spy).unwrap_err();

    assert!(matches!(error, FixBatchError::OwnershipCollisions { .. }));
    assert!(spy.writes().is_empty());
}

#[test]
fn all_collisions_are_reported_in_deterministic_byte_order() {
    let inputs = [
        vec![
            pair("z.rs", "p/b.md"),
            pair("b.rs", "p/a.md"),
            pair("a.rs", "p/a.md"),
            pair("y.rs", "p/b.md"),
        ],
        vec![
            pair("y.rs", "p/b.md"),
            pair("a.rs", "p/a.md"),
            pair("z.rs", "p/b.md"),
            pair("b.rs", "p/a.md"),
        ],
    ];
    let rendered: Vec<String> = inputs
        .iter()
        .map(|pairs| {
            let spy = Spy::seeded(pairs);
            format!("{}", plan_bijective(pairs, &spy).unwrap_err())
        })
        .collect();

    assert_eq!(rendered[0], rendered[1]);
    assert!(rendered[0].find("p/a.md").unwrap() < rendered[0].find("p/b.md").unwrap());
    assert!(rendered[0].find("a.rs").unwrap() < rendered[0].find("b.rs").unwrap());
    assert!(rendered[0].contains("y.rs") && rendered[0].contains("z.rs"));
    assert_eq!(
        rendered[0],
        "ownership collisions: p/a.md=[a.rs, b.rs] p/b.md=[y.rs, z.rs]"
    );
}

#[test]
fn plan_deduplicates_only_the_exact_same_bijective_identity() {
    let inputs = vec![
        pair("same.rs", "a.md"),
        pair("same.rs", "a.md"),
        pair("same.rs", "b.md"),
    ];
    let spy = Spy::seeded(&inputs);
    let plan = plan_bijective(&inputs, &spy).expect("distinct prompt identity remains distinct");

    assert_eq!(plan.pairs.len(), 2);
    assert_eq!(plan.pairs[0].prompt_path, "a.md");
    assert_eq!(plan.pairs[1].prompt_path, "b.md");
}

#[test]
fn missing_metadata_and_any_other_preflight_failure_block_all_writes() {
    let pairs = vec![pair("a.rs", "a.md"), pair("b.rs", "b.md")];
    let spy = Spy::seeded(&pairs);
    *spy.fail_preflight.borrow_mut() = Some(PathBuf::from("b.rs"));

    let error = plan_bijective(&pairs, &spy).unwrap_err();

    assert!(matches!(error, FixBatchError::Preflight { .. }));
    assert!(spy.writes().is_empty());
    assert_eq!(
        spy.calls.borrow().as_slice(),
        ["preflight:a.rs", "preflight:b.rs"]
    );
}

#[test]
fn application_failure_rolls_back_every_applied_pair() {
    let pairs = vec![pair("a.rs", "a.md"), pair("b.rs", "b.md")];
    let spy = Spy::seeded(&pairs);
    let before = spy.state.borrow().clone();
    let plan = plan_bijective(&pairs, &spy).unwrap();
    spy.fail_apply_at.set(Some(1));

    let result = execute_bijective(&plan, &spy, false);

    assert!(matches!(result, FixBatchResult::RolledBack { .. }));
    assert_eq!(*spy.state.borrow(), before);
    assert_eq!(spy.writes(), ["apply:a.rs", "apply:b.rs", "rollback:a.rs"]);
}

#[test]
fn rollback_failure_is_fatal_and_never_reported_as_complete() {
    let pairs = vec![pair("a.rs", "a.md"), pair("b.rs", "b.md")];
    let spy = Spy::seeded(&pairs);
    let plan = plan_bijective(&pairs, &spy).unwrap();
    spy.fail_apply_at.set(Some(1));
    spy.fail_rollback.set(true);

    let result = execute_bijective(&plan, &spy, false);

    assert!(matches!(
        result,
        FixBatchResult::Fatal(ApplyFailure::RollbackFailed { .. })
    ));
    assert!(!result.is_complete());
}

#[test]
fn second_pass_validates_both_bytes_of_every_pair() {
    let pairs = vec![pair("a.rs", "a.md")];
    let spy = Spy::seeded(&pairs);
    let plan = plan_bijective(&pairs, &spy).unwrap();
    assert!(matches!(
        execute_bijective(&plan, &spy, false),
        FixBatchResult::Applied { .. }
    ));

    spy.state
        .borrow_mut()
        .get_mut(Path::new("a.rs"))
        .unwrap()
        .prompt_bytes = b"Hash do Codigo ausente ou obsoleto".to_vec();

    let error = validate_bijective(&plan, &spy).unwrap_err();
    assert!(error.to_string().contains("paridade bidirecional"));
}

#[test]
fn dry_run_and_real_execution_share_the_exact_same_integral_plan() {
    let pairs = vec![pair("b.rs", "b.md"), pair("a.rs", "a.md")];
    let dry_spy = Spy::seeded(&pairs);
    let real_spy = Spy::seeded(&pairs);
    let dry_plan: FixBatchPlan = plan_bijective(&pairs, &dry_spy).unwrap();
    let real_plan: FixBatchPlan = plan_bijective(&pairs, &real_spy).unwrap();

    assert_eq!(dry_plan, real_plan);
    assert!(matches!(
        execute_bijective(&dry_plan, &dry_spy, true),
        FixBatchResult::DryRun { .. }
    ));
    assert!(dry_spy.writes().is_empty());
    assert!(matches!(
        execute_bijective(&real_plan, &real_spy, false),
        FixBatchResult::Applied { .. }
    ));
}

#[test]
fn p1179_false_closure_is_rejected_when_only_direct_v5_is_green() {
    let pairs = vec![pair("typst.rs", "typst.md")];
    let spy = Spy::seeded(&pairs);
    let plan = plan_bijective(&pairs, &spy).unwrap();
    assert!(matches!(
        execute_bijective(&plan, &spy, false),
        FixBatchResult::Applied { .. }
    ));

    // Reproduz o estado P1179: o @prompt-hash direto parece correto, mas a metadata
    // reversa foi removida. Uma validação somente V5 declararia "Nothing to fix".
    let state = spy.state.borrow_mut();
    let mut broken = state.get(Path::new("typst.rs")).unwrap().clone();
    drop(state);
    broken.prompt_bytes.clear();
    spy.state
        .borrow_mut()
        .insert(PathBuf::from("typst.rs"), broken);

    assert!(validate_bijective(&plan, &spy).is_err());
}
