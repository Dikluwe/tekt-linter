use std::cell::Cell;
use std::path::PathBuf;

use crystalline_lint::contracts::file_provider::SourceFile;
use crystalline_lint::contracts::language_parser::{
    parser_slot, LanguageParser, ParserSet, ParserSlot,
};
use crystalline_lint::contracts::parse_error::ParseError;
use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::parsed_file::{ParsedFile, PublicInterface};

fn source(language: Language) -> SourceFile {
    SourceFile {
        path: PathBuf::from("sentinela/entrada.ext"),
        content: "conteúdo sentinela \0 🦀".to_owned(),
        language,
        layer: Layer::Lab,
        has_adjacent_test: true,
    }
}

fn expected_slot(language: &Language) -> Option<ParserSlot> {
    match language {
        Language::Rust => Some(ParserSlot::Rust),
        Language::TypeScript => Some(ParserSlot::TypeScript),
        Language::Python => Some(ParserSlot::Python),
        Language::C => Some(ParserSlot::C),
        Language::Cpp => Some(ParserSlot::Cpp),
        Language::Zig => Some(ParserSlot::Zig),
        Language::Go => Some(ParserSlot::Go),
        Language::Java => Some(ParserSlot::Java),
        Language::Elixir => Some(ParserSlot::Elixir),
        Language::Unknown => None,
    }
}

#[test]
fn b1_maps_all_variants_and_ignores_every_other_source_field() {
    let languages = [
        Language::Rust,
        Language::TypeScript,
        Language::Python,
        Language::C,
        Language::Cpp,
        Language::Zig,
        Language::Go,
        Language::Java,
        Language::Elixir,
        Language::Unknown,
    ];

    for language in languages {
        let expected = expected_slot(&language);
        let mut file = source(language);

        assert_eq!(parser_slot(&file.language), expected);
        file.path = PathBuf::from("../../outro/路径/arquivo");
        file.content = "mutado\0\r\nλ".to_owned();
        file.layer = Layer::Unknown;
        file.has_adjacent_test = false;
        assert_eq!(parser_slot(&file.language), expected);
        assert_eq!(parser_slot(&file.language), expected);
    }
}

#[derive(Clone, Copy)]
enum Reply {
    Ok,
    Err(&'static str),
}

struct Spy {
    calls: Cell<usize>,
    received: Cell<*const SourceFile>,
    reply: Cell<Reply>,
}

impl Spy {
    fn new(id: &'static str) -> Self {
        Self {
            calls: Cell::new(0),
            received: Cell::new(std::ptr::null()),
            reply: Cell::new(Reply::Err(id)),
        }
    }

    fn reset(&self, reply: Reply) {
        self.calls.set(0);
        self.received.set(std::ptr::null());
        self.reply.set(reply);
    }
}

impl LanguageParser for Spy {
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        self.calls.set(self.calls.get() + 1);
        self.received.set(file as *const SourceFile);
        match self.reply.get() {
            Reply::Ok => Ok(ParsedFile {
                path: &file.path,
                layer: file.layer.clone(),
                language: file.language.clone(),
                prompt_header: None,
                prompt_file_exists: false,
                has_test_coverage: file.has_adjacent_test,
                imports: vec![],
                tokens: vec![],
                public_interface: PublicInterface::empty(),
                prompt_snapshot: None,
                declarations: vec![],
                declared_traits: vec![],
                implemented_traits: vec![],
            }),
            Reply::Err(message) => Err(ParseError::SyntaxError {
                path: PathBuf::from(format!("spy/{message}")),
                line: 91,
                column: 7,
                message: message.to_owned(),
            }),
        }
    }
}

struct Spies {
    rust: Spy,
    typescript: Spy,
    python: Spy,
    c: Spy,
    cpp: Spy,
    zig: Spy,
    go: Spy,
    java: Spy,
    elixir: Spy,
}

impl Spies {
    fn new() -> Self {
        Self {
            rust: Spy::new("rust"),
            typescript: Spy::new("typescript"),
            python: Spy::new("python"),
            c: Spy::new("c"),
            cpp: Spy::new("cpp"),
            zig: Spy::new("zig"),
            go: Spy::new("go"),
            java: Spy::new("java"),
            elixir: Spy::new("elixir"),
        }
    }

    fn set(&self) -> ParserSet<'_> {
        ParserSet {
            rust: &self.rust,
            typescript: &self.typescript,
            python: &self.python,
            c: &self.c,
            cpp: &self.cpp,
            zig: &self.zig,
            go: &self.go,
            java: &self.java,
            elixir: &self.elixir,
        }
    }

    fn all(&self) -> [&Spy; 9] {
        [
            &self.rust,
            &self.typescript,
            &self.python,
            &self.c,
            &self.cpp,
            &self.zig,
            &self.go,
            &self.java,
            &self.elixir,
        ]
    }

    fn reset(&self, reply: Reply) {
        for spy in self.all() {
            spy.reset(reply);
        }
    }
}

#[test]
fn b2_calls_only_the_selected_independent_spy_once_with_the_same_source_and_exact_err() {
    let cases = [
        (Language::Rust, 0, "rust"),
        (Language::TypeScript, 1, "typescript"),
        (Language::Python, 2, "python"),
        (Language::C, 3, "c"),
        (Language::Cpp, 4, "cpp"),
        (Language::Zig, 5, "zig"),
        (Language::Go, 6, "go"),
        (Language::Java, 7, "java"),
        (Language::Elixir, 8, "elixir"),
    ];
    let spies = Spies::new();

    for (language, selected, id) in cases {
        spies.reset(Reply::Err("unselected"));
        spies.all()[selected].reply.set(Reply::Err(id));
        let file = source(language);
        let original_address = &file as *const SourceFile;

        let result = spies.set().parse(&file);

        assert_eq!(
            result.unwrap_err(),
            ParseError::SyntaxError {
                path: PathBuf::from(format!("spy/{id}")),
                line: 91,
                column: 7,
                message: id.to_owned(),
            }
        );
        for (index, spy) in spies.all().into_iter().enumerate() {
            assert_eq!(spy.calls.get(), usize::from(index == selected));
        }
        assert_eq!(spies.all()[selected].received.get(), original_address);
    }
}

#[test]
fn b2_propagates_ok_and_unknown_is_exact_without_any_spy_call() {
    let spies = Spies::new();
    spies.reset(Reply::Err("must-not-run"));
    spies.java.reply.set(Reply::Ok);
    let file = source(Language::Java);
    let original_address = &file as *const SourceFile;

    let parsed = spies.set().parse(&file).unwrap();

    assert_eq!(parsed.path, file.path.as_path());
    assert_eq!(parsed.language, Language::Java);
    assert_eq!(parsed.layer, Layer::Lab);
    assert!(parsed.has_test_coverage);
    assert_eq!(spies.java.calls.get(), 1);
    assert_eq!(spies.java.received.get(), original_address);
    assert_eq!(
        spies.all().iter().map(|spy| spy.calls.get()).sum::<usize>(),
        1
    );

    spies.reset(Reply::Err("must-not-run"));
    let unknown = source(Language::Unknown);
    let expected_path = unknown.path.clone();
    assert_eq!(
        spies.set().parse(&unknown).unwrap_err(),
        ParseError::UnsupportedLanguage {
            path: expected_path,
            language: Language::Unknown,
        }
    );
    assert_eq!(
        spies.all().iter().map(|spy| spy.calls.get()).sum::<usize>(),
        0
    );
}
