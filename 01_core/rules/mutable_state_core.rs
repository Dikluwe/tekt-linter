//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/mutable-state-core.md
//! @prompt-hash 5e372e8d
//! @layer L1
//! @updated 2026-03-22

use std::borrow::Cow;

use crate::entities::layer::Layer;
use crate::entities::parsed_file::StaticDeclaration;
use crate::entities::rule_traits::HasStaticDeclarations;
use crate::entities::violation::{Location, Violation, ViolationLevel};

const MUTABLE_STATE_TOKENS: &[&str] = &[
    "Mutex", "RwLock", "OnceLock", "LazyLock",
    "AtomicBool", "AtomicI8", "AtomicI16", "AtomicI32", "AtomicI64",
    "AtomicIsize", "AtomicU8", "AtomicU16", "AtomicU32", "AtomicU64",
    "AtomicUsize", "AtomicPtr", "RefCell", "UnsafeCell",
];

/// V13 — Mutable State In Core.
///
/// Estado global mutável viola a pureza de L1 — funções que lêem de
/// `static Mutex<T>` não são determinísticas. V13 proíbe qualquer
/// `static_item` em L1 cujo tipo contém tokens de mutabilidade interior,
/// bem como `static mut T` directamente.
///
/// Error — aplica-se apenas a arquivos com `layer == L1`.
pub fn check<'a, T: HasStaticDeclarations<'a>>(file: &T) -> Vec<Violation<'a>> {
    if *file.layer() != Layer::L1 {
        return vec![];
    }

    file.static_declarations()
        .iter()
        .filter(|s| is_mutable_static(s))
        .map(|s| Violation {
            rule_id: "V13".to_string(),
            level: ViolationLevel::Error,
            message: format!(
                "Estado global mutável em L1: '{}' usa '{}'. \
                 Estado deve ser injectado por parâmetro, não partilhado globalmente.",
                s.name,
                offending_token(s),
            ),
            location: Location {
                path: Cow::Borrowed(file.path()),
                line: s.line,
                column: 0,
            },
        })
        .collect()
}

fn is_mutable_static(s: &StaticDeclaration<'_>) -> bool {
    if s.is_mut {
        return true;
    }
    MUTABLE_STATE_TOKENS.iter().any(|token| s.type_text.contains(token))
}

fn offending_token(s: &StaticDeclaration<'_>) -> &'static str {
    if s.is_mut {
        return "mut";
    }
    MUTABLE_STATE_TOKENS
        .iter()
        .find(|token| s.type_text.contains(*token))
        .copied()
        .unwrap_or("estado mutável")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use crate::entities::layer::Layer;
    use crate::entities::parsed_file::StaticDeclaration;

    struct MockFile {
        layer: Layer,
        statics: Vec<StaticDeclaration<'static>>,
        path: &'static Path,
    }

    impl HasStaticDeclarations<'static> for MockFile {
        fn layer(&self) -> &Layer { &self.layer }
        fn static_declarations(&self) -> &[StaticDeclaration<'static>] { &self.statics }
        fn path(&self) -> &'static Path { self.path }
    }

    fn l1_file(statics: Vec<StaticDeclaration<'static>>) -> MockFile {
        MockFile { layer: Layer::L1, statics, path: Path::new("01_core/foo.rs") }
    }

    fn l3_file(statics: Vec<StaticDeclaration<'static>>) -> MockFile {
        MockFile { layer: Layer::L3, statics, path: Path::new("03_infra/foo.rs") }
    }

    #[test]
    fn static_mut_triggers_v13() {
        let file = l1_file(vec![
            StaticDeclaration { name: "COUNTER", type_text: "u32", is_mut: true, line: 5 },
        ]);
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V13");
        assert_eq!(violations[0].level, ViolationLevel::Error);
        assert_eq!(violations[0].location.line, 5);
        assert!(violations[0].message.contains("COUNTER"));
        assert!(violations[0].message.contains("mut"));
    }

    #[test]
    fn mutex_static_triggers_v13() {
        let file = l1_file(vec![
            StaticDeclaration {
                name: "CACHE",
                type_text: "Mutex<HashMap<String, Vec<u8>>>",
                is_mut: false,
                line: 10,
            },
        ]);
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V13");
        assert!(violations[0].message.contains("Mutex"));
    }

    #[test]
    fn once_lock_static_triggers_v13() {
        let file = l1_file(vec![
            StaticDeclaration {
                name: "INSTANCE",
                type_text: "OnceLock<Config>",
                is_mut: false,
                line: 3,
            },
        ]);
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("OnceLock"));
    }

    #[test]
    fn lazy_lock_static_triggers_v13() {
        let file = l1_file(vec![
            StaticDeclaration {
                name: "TABLE",
                type_text: "LazyLock<HashMap<&str, Layer>>",
                is_mut: false,
                line: 7,
            },
        ]);
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("LazyLock"));
    }

    #[test]
    fn atomic_usize_triggers_v13() {
        let file = l1_file(vec![
            StaticDeclaration {
                name: "ATOMIC",
                type_text: "AtomicUsize",
                is_mut: false,
                line: 2,
            },
        ]);
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("AtomicUsize"));
    }

    #[test]
    fn immutable_str_static_is_allowed() {
        let file = l1_file(vec![
            StaticDeclaration {
                name: "RULE_ID",
                type_text: "&str",
                is_mut: false,
                line: 1,
            },
        ]);
        let violations = check(&file);
        assert!(violations.is_empty());
    }

    #[test]
    fn immutable_slice_static_is_allowed_even_with_mutex_in_name() {
        // The text "Mutex" appears as a string literal, not as a type.
        // &[&str] is the type — not a Mutex.
        let file = l1_file(vec![
            StaticDeclaration {
                name: "FORBIDDEN_TOKENS",
                type_text: "&[&str]",
                is_mut: false,
                line: 1,
            },
        ]);
        let violations = check(&file);
        assert!(violations.is_empty());
    }

    #[test]
    fn mutable_static_in_l3_is_ignored() {
        let file = l3_file(vec![
            StaticDeclaration {
                name: "CACHE",
                type_text: "Mutex<HashMap<String, u32>>",
                is_mut: false,
                line: 10,
            },
        ]);
        let violations = check(&file);
        assert!(violations.is_empty());
    }

    #[test]
    fn two_forbidden_statics_produce_two_violations() {
        let file = l1_file(vec![
            StaticDeclaration { name: "COUNTER", type_text: "u32", is_mut: true, line: 5 },
            StaticDeclaration {
                name: "CACHE",
                type_text: "Mutex<HashMap<String, u32>>",
                is_mut: false,
                line: 10,
            },
        ]);
        let violations = check(&file);
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn empty_statics_returns_empty() {
        let file = l1_file(vec![]);
        assert!(check(&file).is_empty());
    }
}
