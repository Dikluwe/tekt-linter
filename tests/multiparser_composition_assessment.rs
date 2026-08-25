use std::borrow::Cow;
use std::cell::Cell;
use std::path::{Path, PathBuf};

use crystalline_lint::contracts::file_provider::SourceFile;
use crystalline_lint::contracts::language_parser::{LanguageParser, ParserSet};
use crystalline_lint::contracts::parse_error::ParseError;
use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::parsed_file::{
    Declaration, DeclarationKind, FunctionSignature, Import, ImportKind, ModuleDecl, ParsedFile,
    PromptHeader, PublicInterface, StaticDeclaration, Token, TokenKind, TypeKind, TypeSignature,
};
use crystalline_lint::entities::rule_traits::{
    BodyForm, Citation, CitationKind, ConstantKind, DecisionArm, DecisionExpr, ScrutineeForm,
    SemanticObservation, SemanticObservationKind, SourceConstant,
};

#[derive(Clone, Copy)]
enum Reply {
    Ok,
    Err,
}

struct Spy {
    reply: Reply,
    calls: Cell<usize>,
    address: Cell<*const SourceFile>,
}

impl Spy {
    fn new(reply: Reply) -> Self {
        Self {
            reply,
            calls: Cell::new(0),
            address: Cell::new(std::ptr::null()),
        }
    }
}

impl LanguageParser for Spy {
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        self.calls.set(self.calls.get() + 1);
        self.address.set(file as *const SourceFile);
        match self.reply {
            Reply::Ok => Ok(sentinel_parsed(file)),
            Reply::Err => Err(sentinel_error(file)),
        }
    }
}

fn sentinel_error(file: &SourceFile) -> ParseError {
    ParseError::SyntaxError {
        path: file.path.clone(),
        line: 701,
        column: 709,
        message: "sentinel exact parser error".to_owned(),
    }
}

fn sentinel_interface<'a>(prefix: &'a str) -> PublicInterface<'a> {
    PublicInterface {
        functions: vec![FunctionSignature {
            name: prefix,
            params: vec!["sentinel-param"],
            return_type: Some("sentinel-return"),
        }],
        types: vec![TypeSignature {
            name: "sentinel-type",
            kind: TypeKind::Interface,
            members: vec!["sentinel-member"],
        }],
        reexports: vec!["sentinel-reexport"],
    }
}

fn sentinel_parsed(file: &SourceFile) -> ParsedFile<'_> {
    ParsedFile {
        path: file.path.as_path(),
        layer: Layer::L3,
        language: file.language.clone(),
        prompt_header: Some(PromptHeader {
            prompt_path: "sentinel-prompt-path",
            prompt_hash: Some("sentinel-prompt-hash"),
            current_hash: Some("sentinel-current-hash".to_owned()),
            layer: Layer::L2,
            updated: Some("sentinel-updated"),
        }),
        prompt_file_exists: true,
        prompt_refs: vec!["sentinel-prompt-ref"],
        has_test_coverage: true,
        imports: vec![Import {
            path: "sentinel-import",
            line: 101,
            kind: ImportKind::Alias,
            target_layer: Layer::L1,
            target_subdir: Some("sentinel-target-subdir"),
            is_test_origin: true,
        }],
        tokens: vec![Token {
            symbol: Cow::Borrowed("sentinel-token"),
            line: 103,
            column: 107,
            kind: TokenKind::MacroInvocation,
        }],
        public_interface: sentinel_interface("sentinel-function-current"),
        prompt_snapshot: Some(sentinel_interface("sentinel-function-snapshot")),
        declarations: vec![Declaration {
            kind: DeclarationKind::TypeAlias,
            name: "sentinel-declaration",
            line: 109,
        }],
        declared_traits: vec!["sentinel-declared-trait"],
        implemented_traits: vec!["sentinel-implemented-trait"],
        blanket_impl_traits: vec!["sentinel-blanket-impl-trait"],
        static_declarations: vec![StaticDeclaration {
            name: "sentinel-static",
            type_text: "sentinel-static-type",
            line: 113,
            is_mut: true,
        }],
        module_decls: vec![ModuleDecl {
            name: "sentinel-module",
            line: 127,
            target_layer: Layer::L4,
        }],
        decision_exprs: vec![DecisionExpr {
            snippet_scrutinee: "sentinel-scrutinee",
            scrutinee_form: ScrutineeForm::MethodCall,
            arms: vec![DecisionArm {
                pattern_snippet: "sentinel-pattern",
                is_catchall: true,
                bound_ident_used_in_body: true,
                qualified_prefixes: vec!["sentinel-prefix"],
                has_guard: true,
                guard_is_compound: true,
                pattern_is_range: true,
                pattern_depth: 3,
                or_alternatives: 5,
                body_form: BodyForm::ErrorBarrier,
                body_snippet: "sentinel-body",
                line: 131,
                column: 137,
            }],
            line: 139,
            column: 149,
        }],
        constants: vec![SourceConstant {
            kind: ConstantKind::NegativeLiteral,
            snippet: "sentinel-constant",
            line: 151,
            column: 157,
            citation: Some(Citation {
                kind: CitationKind::Spec("sentinel-spec"),
                raw: "sentinel-citation",
                line: 163,
            }),
            is_test_origin: true,
            function_return_type: Some("sentinel-function-return"),
            is_in_binary_scaling: true,
            context_var: Some("sentinel-context-var".to_owned()),
            geometric_sink: Some("sentinel-geometric-sink".to_owned()),
            is_in_data_table: true,
        }],
        semantic_observations: vec![SemanticObservation {
            contract_id: "sentinel-contract-id".to_owned(),
            kind: SemanticObservationKind::DirectDecisionReimplementation,
            detail: "sentinel-observation-detail".to_owned(),
            line: 167,
            column: 173,
        }],
    }
}

fn source(language: Language) -> SourceFile {
    SourceFile {
        path: PathBuf::from("sentinel/path/source.unit"),
        content: "sentinel source content".to_owned(),
        language,
        layer: Layer::Lab,
        has_adjacent_test: false,
    }
}

fn supported_languages() -> [Language; 9] {
    [
        Language::Rust,
        Language::TypeScript,
        Language::Python,
        Language::C,
        Language::Cpp,
        Language::Zig,
        Language::Go,
        Language::Java,
        Language::Elixir,
    ]
}

fn exercise(reply: Reply) {
    for (selected, language) in supported_languages().into_iter().enumerate() {
        let spies: Vec<Spy> = (0..9).map(|_| Spy::new(reply)).collect();
        let set = ParserSet {
            rust: &spies[0],
            typescript: &spies[1],
            python: &spies[2],
            c: &spies[3],
            cpp: &spies[4],
            zig: &spies[5],
            go: &spies[6],
            java: &spies[7],
            elixir: &spies[8],
        };
        let file = source(language);
        let expected = match reply {
            Reply::Ok => Ok(sentinel_parsed(&file)),
            Reply::Err => Err(sentinel_error(&file)),
        };
        let actual = set.parse(&file);

        assert_eq!(
            actual, expected,
            "result changed for selected slot {selected}"
        );
        for (index, spy) in spies.iter().enumerate() {
            assert_eq!(spy.calls.get(), usize::from(index == selected));
        }
        assert_eq!(spies[selected].address.get(), &file as *const SourceFile);
    }
}

#[test]
fn each_supported_language_calls_only_its_port_and_propagates_complete_ok() {
    exercise(Reply::Ok);
}

#[test]
fn each_supported_language_calls_only_its_port_and_propagates_exact_err() {
    exercise(Reply::Err);
}

#[test]
fn unknown_calls_no_port_and_returns_exact_unsupported_language() {
    let spies: Vec<Spy> = (0..9).map(|_| Spy::new(Reply::Ok)).collect();
    let set = ParserSet {
        rust: &spies[0],
        typescript: &spies[1],
        python: &spies[2],
        c: &spies[3],
        cpp: &spies[4],
        zig: &spies[5],
        go: &spies[6],
        java: &spies[7],
        elixir: &spies[8],
    };
    let file = source(Language::Unknown);

    assert_eq!(
        set.parse(&file),
        Err(ParseError::UnsupportedLanguage {
            path: file.path.clone(),
            language: Language::Unknown,
        })
    );
    assert!(spies.iter().all(|spy| spy.calls.get() == 0));
    assert!(spies.iter().all(|spy| spy.address.get().is_null()));
    assert_eq!(file.path.as_path(), Path::new("sentinel/path/source.unit"));
}
