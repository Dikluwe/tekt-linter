use std::path::Path;

use crystalline_lint::entities::layer::Layer;
use crystalline_lint::entities::parsed_file::StaticDeclaration;
use crystalline_lint::entities::rule_traits::HasStaticDeclarations;
use crystalline_lint::entities::violation::ViolationLevel;
use crystalline_lint::rules::mutable_state_core::check;

const NORMATIVE_TOKENS: [&str; 18] = [
    "Mutex",
    "RwLock",
    "OnceLock",
    "LazyLock",
    "AtomicBool",
    "AtomicI8",
    "AtomicI16",
    "AtomicI32",
    "AtomicI64",
    "AtomicIsize",
    "AtomicU8",
    "AtomicU16",
    "AtomicU32",
    "AtomicU64",
    "AtomicUsize",
    "AtomicPtr",
    "RefCell",
    "UnsafeCell",
];

struct StaticFile<'a> {
    layer: Layer,
    declarations: Vec<StaticDeclaration<'a>>,
    path: &'a Path,
}

impl<'a> HasStaticDeclarations<'a> for StaticFile<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }

    fn static_declarations(&self) -> &[StaticDeclaration<'a>] {
        &self.declarations
    }

    fn path(&self) -> &'a Path {
        self.path
    }
}

fn file<'a>(layer: Layer, declarations: Vec<StaticDeclaration<'a>>) -> StaticFile<'a> {
    StaticFile {
        layer,
        declarations,
        path: Path::new("01_core/rules/example.rs"),
    }
}

fn declaration<'a>(
    name: &'a str,
    type_text: &'a str,
    is_mut: bool,
    line: usize,
) -> StaticDeclaration<'a> {
    StaticDeclaration {
        name,
        type_text,
        is_mut,
        line,
    }
}

#[test]
fn all_18_normative_tokens_are_rejected_nominally_in_l1() {
    assert_eq!(NORMATIVE_TOKENS.len(), 18);

    for (index, token) in NORMATIVE_TOKENS.iter().enumerate() {
        let type_text = format!("wrapper::{token}<Payload>");
        let name = format!("STATIC_{index}");
        let input = file(
            Layer::L1,
            vec![declaration(&name, &type_text, false, index + 10)],
        );

        let violations = check(&input);
        assert_eq!(violations.len(), 1, "token nominal ausente: {token}");
        let violation = &violations[0];
        assert_eq!(violation.rule_id, "V13", "token: {token}");
        assert_eq!(violation.level, ViolationLevel::Error, "token: {token}");
        assert_eq!(
            violation.location.path.as_ref(),
            input.path,
            "token: {token}"
        );
        assert_eq!(violation.location.line, index + 10, "token: {token}");
        assert_eq!(violation.location.column, 0, "token: {token}");
        assert!(violation.message.contains(&name), "token: {token}");
        assert!(violation.message.contains(token), "token: {token}");
    }
}

#[test]
fn v13_scope_is_exactly_l1_for_every_normative_token() {
    for layer in [
        Layer::L0,
        Layer::L2,
        Layer::L3,
        Layer::L4,
        Layer::Lab,
        Layer::Unknown,
    ] {
        let declarations = NORMATIVE_TOKENS
            .iter()
            .enumerate()
            .map(|(index, token)| declaration(token, token, false, index + 1))
            .collect();
        assert!(check(&file(layer, declarations)).is_empty());
    }
}

#[test]
fn static_mut_is_rejected_without_a_forbidden_token_and_mut_takes_precedence() {
    let input = file(
        Layer::L1,
        vec![
            declaration("COUNTER", "u32", true, 21),
            declaration("CACHE", "Mutex<u8>", true, 22),
        ],
    );

    let violations = check(&input);
    assert_eq!(violations.len(), 2);
    assert!(violations[0].message.contains("COUNTER"));
    assert!(violations[0].message.contains("'mut'"));
    assert!(violations[1].message.contains("CACHE"));
    assert!(violations[1].message.contains("'mut'"));
    assert!(!violations[1].message.contains("usa 'Mutex'"));
}

#[test]
fn immutable_near_misses_remain_allowed_without_normalization() {
    let near_misses = [
        "mutex<T>",
        "RWLock<T>",
        "Once_Lock<T>",
        "Lazy<T>",
        "AtomicU128",
        "AtomicI128",
        "Cell<T>",
        "SafeCell<T>",
        "&'static str",
        "&'static [&'static str]",
    ];
    let declarations = near_misses
        .iter()
        .enumerate()
        .map(|(index, type_text)| declaration("IMMUTABLE", type_text, false, index + 1))
        .collect();

    assert!(check(&file(Layer::L1, declarations)).is_empty());
}

#[test]
fn violations_preserve_input_order_and_cardinality_including_duplicates() {
    let input = file(
        Layer::L1,
        vec![
            declaration("THIRD", "AtomicPtr<u8>", false, 30),
            declaration("FIRST", "Mutex<u8>", false, 10),
            declaration("FIRST_AGAIN", "Mutex<u16>", false, 10),
            declaration("OK", "usize", false, 99),
            declaration("SECOND", "RefCell<String>", false, 20),
        ],
    );

    let violations = check(&input);
    assert_eq!(violations.len(), 4);
    assert_eq!(
        violations
            .iter()
            .map(|v| v.location.line)
            .collect::<Vec<_>>(),
        vec![30, 10, 10, 20]
    );
    for (violation, name) in violations
        .iter()
        .zip(["THIRD", "FIRST", "FIRST_AGAIN", "SECOND"])
    {
        assert!(violation.message.contains(name));
    }
}
