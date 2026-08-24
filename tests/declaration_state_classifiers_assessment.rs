use crystalline_lint::entities::layer::Layer;
use crystalline_lint::entities::parsed_file::{
    Declaration, DeclarationKind, StaticDeclaration, WiringConfig,
};
use crystalline_lint::entities::rule_traits::{HasStaticDeclarations, HasWiringPurity};
use crystalline_lint::entities::violation::ViolationLevel;
use crystalline_lint::rules::{mutable_state_core, wiring_logic_leak};
use std::path::Path;

struct DeclarationFixture<'a> {
    layer: Layer,
    declarations: Vec<Declaration<'a>>,
    path: &'a Path,
}

impl<'a> HasWiringPurity<'a> for DeclarationFixture<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn declarations(&self) -> &[Declaration<'a>] {
        &self.declarations
    }
    fn path(&self) -> &'a Path {
        self.path
    }
}

fn kinds() -> [DeclarationKind; 6] {
    [
        DeclarationKind::Struct,
        DeclarationKind::Enum,
        DeclarationKind::Impl,
        DeclarationKind::Class,
        DeclarationKind::Interface,
        DeclarationKind::TypeAlias,
    ]
}

fn layers() -> [Layer; 7] {
    [
        Layer::L0,
        Layer::L1,
        Layer::L2,
        Layer::L3,
        Layer::L4,
        Layer::Lab,
        Layer::Unknown,
    ]
}

#[test]
fn v12_matches_seven_layers_six_kinds_and_both_config_states() {
    for layer in layers() {
        for kind in kinds() {
            for allow_adapter_structs in [false, true] {
                let fixture = DeclarationFixture {
                    layer: layer.clone(),
                    declarations: vec![Declaration {
                        kind: kind.clone(),
                        name: "Thing",
                        line: 5,
                    }],
                    path: Path::new("module.rs"),
                };
                let violations = wiring_logic_leak::check(
                    &fixture,
                    &WiringConfig {
                        allow_adapter_structs,
                    },
                );
                let always_forbidden = matches!(
                    kind,
                    DeclarationKind::Enum
                        | DeclarationKind::Impl
                        | DeclarationKind::Interface
                        | DeclarationKind::TypeAlias
                );
                let adapter = matches!(kind, DeclarationKind::Struct | DeclarationKind::Class);
                let expected =
                    layer == Layer::L4 && (always_forbidden || (adapter && !allow_adapter_structs));
                assert_eq!(
                    violations.len(),
                    usize::from(expected),
                    "V12 mismatch: {layer:?}/{kind:?}/allow={allow_adapter_structs}"
                );
            }
        }
    }
}

#[test]
fn v12_preserves_order_multiplicity_path_line_kind_name_and_unicode_evidence() {
    let path = Path::new("04_wiring/á/源.rs");
    let declarations = vec![
        Declaration {
            kind: DeclarationKind::Enum,
            name: "Árvore",
            line: 9,
        },
        Declaration {
            kind: DeclarationKind::Enum,
            name: "Árvore",
            line: 9,
        },
        Declaration {
            kind: DeclarationKind::Interface,
            name: "A\u{301}rvore",
            line: 2,
        },
    ];
    let violations = wiring_logic_leak::check(
        &DeclarationFixture {
            layer: Layer::L4,
            declarations,
            path,
        },
        &WiringConfig::default(),
    );
    assert_eq!(violations.len(), 3);
    assert_eq!(
        violations
            .iter()
            .map(|v| v.location.line)
            .collect::<Vec<_>>(),
        vec![9, 9, 2]
    );
    for violation in &violations {
        assert_eq!(violation.rule_id, "V12");
        assert_eq!(violation.level, ViolationLevel::Warning);
        assert_eq!(violation.location.path.as_ref(), path);
    }
    let expected = [
        ("Árvore", "enum"),
        ("Árvore", "enum"),
        ("A\u{301}rvore", "interface"),
    ];
    let missing: Vec<_> = violations
        .iter()
        .zip(expected)
        .filter_map(|(violation, (name, kind))| {
            (!(violation.message.contains(name) && violation.message.to_lowercase().contains(kind)))
                .then(|| violation.message.clone())
        })
        .collect();
    assert!(
        missing.is_empty(),
        "V12 messages lost name/kind evidence: {missing:?}"
    );
}

#[test]
fn v12_config_toggle_is_isolated_to_struct_class_and_does_not_mutate_input() {
    let declarations: Vec<_> = kinds()
        .into_iter()
        .enumerate()
        .map(|(line, kind)| Declaration {
            kind,
            name: "Same",
            line: line + 1,
        })
        .collect();
    let before = declarations.clone();
    let fixture = DeclarationFixture {
        layer: Layer::L4,
        declarations,
        path: Path::new("04_wiring/value.rs"),
    };
    let permissive = wiring_logic_leak::check(
        &fixture,
        &WiringConfig {
            allow_adapter_structs: true,
        },
    );
    let strict = wiring_logic_leak::check(
        &fixture,
        &WiringConfig {
            allow_adapter_structs: false,
        },
    );
    assert_eq!(permissive.len(), 4);
    assert_eq!(strict.len(), 6);
    assert_eq!(fixture.declarations, before);
    for line in [2, 3, 5, 6] {
        assert_eq!(
            permissive
                .iter()
                .filter(|v| v.location.line == line)
                .count(),
            1
        );
        assert_eq!(strict.iter().filter(|v| v.location.line == line).count(), 1);
    }
    assert_eq!(
        strict
            .iter()
            .filter(|v| matches!(v.location.line, 1 | 4))
            .count(),
        2
    );
}

struct StaticFixture<'a> {
    layer: Layer,
    declarations: Vec<StaticDeclaration<'a>>,
    path: &'a Path,
}

impl<'a> HasStaticDeclarations<'a> for StaticFixture<'a> {
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

// SPEC-GAP(A4/A5): o Assessment 0011 exige 18 tokens, mas não os enumera e a API
// pública não exporta a tabela. Estes 16 são observáveis pelo contrato nominal;
// os dois nomes restantes não podem ser congelados sem ler o alvo proibido.
const PUBLICLY_IDENTIFIABLE_TOKENS: [&str; 16] = [
    "Mutex",
    "RwLock",
    "RefCell",
    "UnsafeCell",
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
];

#[test]
fn v13_matches_seven_layers_is_mut_and_all_publicly_identifiable_tokens() {
    let mut missed = Vec::new();
    for layer in layers() {
        for is_mut in [false, true] {
            for token in PUBLICLY_IDENTIFIABLE_TOKENS {
                let fixture = StaticFixture {
                    layer: layer.clone(),
                    declarations: vec![StaticDeclaration {
                        name: "STATE",
                        type_text: token,
                        is_mut,
                        line: 4,
                    }],
                    path: Path::new("module.rs"),
                };
                let actual = mutable_state_core::check(&fixture).len();
                let expected = usize::from(layer == Layer::L1);
                if actual != expected {
                    missed.push(format!(
                        "{layer:?}/{is_mut}/{token}: {actual} != {expected}"
                    ));
                }
            }
        }
    }
    assert!(
        missed.is_empty(),
        "V13 candidate token mismatches: {missed:?}"
    );
}

#[test]
fn v13_nearby_immutable_types_are_exempt_and_mut_has_message_precedence() {
    let nearby = [
        "mutex",
        "RWLock",
        "AtomicU128",
        "AtomU8",
        "SafeCell",
        "Once_Cell",
        "Vec<CellValue>",
        "Mútex",
        "普通型",
    ];
    for type_text in nearby {
        let immutable = StaticFixture {
            layer: Layer::L1,
            declarations: vec![StaticDeclaration {
                name: "VALUE",
                type_text,
                is_mut: false,
                line: 1,
            }],
            path: Path::new("01_core/value.rs"),
        };
        assert!(
            mutable_state_core::check(&immutable).is_empty(),
            "substring false positive: {type_text}"
        );
    }

    let mutable = StaticFixture {
        layer: Layer::L1,
        declarations: vec![StaticDeclaration {
            name: "MUTÁVEL",
            type_text: "Mutex<AtomicU8>",
            is_mut: true,
            line: 3,
        }],
        path: Path::new("01_core/value.rs"),
    };
    let violations = mutable_state_core::check(&mutable);
    assert_eq!(violations.len(), 1);
    let immutable_token = StaticFixture {
        layer: Layer::L1,
        declarations: vec![StaticDeclaration {
            name: "MUTÁVEL",
            type_text: "Mutex<AtomicU8>",
            is_mut: false,
            line: 3,
        }],
        path: Path::new("01_core/value.rs"),
    };
    let token_message = mutable_state_core::check(&immutable_token)[0]
        .message
        .clone();
    assert_ne!(
        violations[0].message, token_message,
        "is_mut and token causes collapsed to the same message: {:?}",
        violations[0].message
    );
}

#[test]
fn v13_preserves_order_multiplicity_path_line_name_evidence_and_is_deterministic() {
    let path = Path::new("01_core/estado/源.rs");
    let fixture = StaticFixture {
        layer: Layer::L1,
        declarations: vec![
            StaticDeclaration {
                name: "Á",
                type_text: "Mutex<u8>",
                is_mut: false,
                line: 8,
            },
            StaticDeclaration {
                name: "Á",
                type_text: "Mutex<u8>",
                is_mut: false,
                line: 8,
            },
            StaticDeclaration {
                name: "A\u{301}",
                type_text: "AtomicPtr<普通型>",
                is_mut: false,
                line: 2,
            },
        ],
        path,
    };
    let first = mutable_state_core::check(&fixture);
    let second = mutable_state_core::check(&fixture);
    assert_eq!(first, second);
    assert_eq!(first.len(), 3);
    assert_eq!(
        first.iter().map(|v| v.location.line).collect::<Vec<_>>(),
        vec![8, 8, 2]
    );
    for violation in &first {
        assert_eq!(violation.rule_id, "V13");
        assert_eq!(violation.level, ViolationLevel::Error);
        assert_eq!(violation.location.path.as_ref(), path);
    }
    assert!(first[0].message.contains("Á") && first[0].message.contains("Mutex"));
    assert!(first[1].message.contains("Á") && first[1].message.contains("Mutex"));
    assert!(first[2].message.contains("A\u{301}") && first[2].message.contains("AtomicPtr"));
}
