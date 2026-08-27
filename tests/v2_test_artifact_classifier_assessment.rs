//! Oráculo independente P0118 — identidade de teste deriva somente do path.

use crystalline_lint::contracts::file_provider::SourceRole;
use crystalline_lint::entities::layer::Language;
use crystalline_lint::infra::walker::classify_source_role;
use std::path::Path;

fn role(path: &str, language: Language) -> SourceRole {
    classify_source_role(&language, Path::new(path))
}

#[test]
fn every_preserved_convention_has_a_positive_control() {
    let cases = [
        ("test_catalogo.py", Language::Python),
        ("catalogo_test.py", Language::Python),
        ("catalogo_test.rs", Language::Rust),
        ("catalogo.test.ts", Language::TypeScript),
        ("catalogo.spec.ts", Language::TypeScript),
        ("catalogo.test.tsx", Language::TypeScript),
        ("catalogo.spec.tsx", Language::TypeScript),
        ("catalogo_test.zig", Language::Zig),
        ("catalogo_test.go", Language::Go),
        ("test_catalogo.c", Language::C),
        ("catalogo_test.c", Language::C),
        ("test_catalogo.cpp", Language::Cpp),
        ("catalogo_test.cpp", Language::Cpp),
        ("test_catalogo.cc", Language::Cpp),
        ("catalogo_test.cc", Language::Cpp),
        ("CatalogoTest.java", Language::Java),
        ("CatalogoTests.java", Language::Java),
        ("TestCatalogo.java", Language::Java),
        ("Catalogo_test.java", Language::Java),
        ("catalogo_test.exs", Language::Elixir),
        ("catalogo_test.ex", Language::Elixir),
    ];

    for (path, language) in cases {
        assert_eq!(role(path, language), SourceRole::Test, "path={path}");
    }
}

#[test]
fn similar_names_case_mismatches_and_empty_stems_are_production() {
    let cases = [
        ("contest_catalogo.py", Language::Python),
        ("testament.py", Language::Python),
        ("catalogo_test_helper.py", Language::Python),
        ("Test_catalogo.py", Language::Python),
        ("test_.py", Language::Python),
        ("_test.py", Language::Python),
        ("latest.rs", Language::Rust),
        ("foo.testing.ts", Language::TypeScript),
        ("Testament.java", Language::Java),
        ("Test.java", Language::Java),
        ("_test.go", Language::Go),
        ("catalogo_test.cxx", Language::Cpp),
        ("catalogo_test.hpp", Language::Cpp),
        ("test_catalogo.h", Language::C),
        ("catalogo_test.h", Language::C),
    ];

    let mismatches: Vec<_> = cases
        .into_iter()
        .filter_map(|(path, language)| {
            let actual = role(path, language);
            (actual != SourceRole::Production).then_some((path, actual))
        })
        .collect();
    assert!(mismatches.is_empty(), "false Test roles: {mismatches:?}");
}

#[cfg(unix)]
#[test]
fn non_utf8_source_name_preserves_unknown_role() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::path::PathBuf;

    let path = PathBuf::from(OsString::from_vec(b"test_catalogo_\xff.py".to_vec()));
    assert_eq!(
        classify_source_role(&Language::Python, &path),
        SourceRole::Unknown
    );
}

#[test]
fn classification_is_independent_of_discovery_order() {
    let inputs = [
        ("z/test_beta.py", Language::Python),
        ("a/alpha.py", Language::Python),
        ("m/gamma_test.py", Language::Python),
        ("b/contest.py", Language::Python),
    ];
    let forward: Vec<_> = inputs
        .iter()
        .map(|(path, language)| (*path, role(path, language.clone())))
        .collect();
    let mut reverse_inputs = inputs;
    reverse_inputs.reverse();
    let mut reverse: Vec<_> = reverse_inputs
        .iter()
        .map(|(path, language)| (*path, role(path, language.clone())))
        .collect();
    reverse.sort_by_key(|(path, _)| *path);
    let mut expected = forward;
    expected.sort_by_key(|(path, _)| *path);
    assert_eq!(reverse, expected);
}
