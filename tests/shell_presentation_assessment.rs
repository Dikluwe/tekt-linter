use crystalline_lint::entities::violation::{Location, Violation, ViolationLevel};
use crystalline_lint::shell::cli::{
    format_sarif, format_text, should_fail, sort_violations, FailLevel,
};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::path::Path;

fn violation(
    rule: &str,
    level: ViolationLevel,
    message: &str,
    path: &'static Path,
    line: usize,
    column: usize,
) -> Violation<'static> {
    Violation {
        rule_id: rule.to_owned(),
        level,
        message: message.to_owned(),
        location: Location {
            path: Cow::Borrowed(path),
            line,
            column,
        },
    }
}

fn permutations(values: &[usize]) -> Vec<Vec<usize>> {
    fn visit(prefix: &mut Vec<usize>, rest: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if rest.is_empty() {
            out.push(prefix.clone());
            return;
        }
        for index in 0..rest.len() {
            let value = rest.remove(index);
            prefix.push(value);
            visit(prefix, rest, out);
            prefix.pop();
            rest.insert(index, value);
        }
    }
    let mut out = Vec::new();
    visit(&mut Vec::new(), &mut values.to_vec(), &mut out);
    out
}

#[test]
fn total_sort_is_independent_of_input_permutation_under_all_ties() {
    let source = vec![
        violation("V2", ViolationLevel::Error, "z", Path::new("same.rs"), 4, 2),
        violation("V2", ViolationLevel::Error, "a", Path::new("same.rs"), 4, 2),
        violation("V1", ViolationLevel::Error, "m", Path::new("same.rs"), 4, 2),
        violation("V9", ViolationLevel::Error, "m", Path::new("same.rs"), 4, 1),
    ];
    let expected_keys = vec![
        (1, "V9", "m"),
        (2, "V1", "m"),
        (2, "V2", "a"),
        (2, "V2", "z"),
    ];
    let mut expected_text = None;
    let mut expected_sarif = None;

    for order in permutations(&[0, 1, 2, 3]) {
        let mut values: Vec<_> = order.iter().map(|index| source[*index].clone()).collect();
        sort_violations(&mut values);
        let keys: Vec<_> = values
            .iter()
            .map(|value| {
                (
                    value.location.column,
                    value.rule_id.as_str(),
                    value.message.as_str(),
                )
            })
            .collect();
        assert_eq!(keys, expected_keys);
        let text = format_text(&values);
        let sarif = format_sarif(&values);
        assert_eq!(expected_text.get_or_insert_with(|| text.clone()), &text);
        assert_eq!(expected_sarif.get_or_insert_with(|| sarif.clone()), &sarif);
    }
}

#[test]
fn hostile_strings_round_trip_and_unix_paths_preserve_original_bytes() {
    let hostile = "quote=\" slash=/ backslash=\\ newline=\n carriage=\r tab=\t nul=\0 esc=\u{1b} separators=\u{2028}\u{2029} emoji=🧪 combining=e\u{301} bidi=\u{202e} NFC=é NFD=e\u{301}";
    let value = violation(
        hostile,
        ViolationLevel::Warning,
        hostile,
        Path::new("safe/path.rs"),
        7,
        3,
    );
    let text = format_text(std::slice::from_ref(&value));
    assert!(text.contains(hostile));
    assert!(text.contains("safe/path.rs"));
    let sarif: serde_json::Value = serde_json::from_str(&format_sarif(&[value])).unwrap();
    assert_eq!(sarif["runs"][0]["results"][0]["ruleId"], hostile);
    assert_eq!(sarif["runs"][0]["results"][0]["message"]["text"], hostile);

    #[cfg(unix)]
    {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let original = b"a\xffb.rs";
        let path = Path::new(OsStr::from_bytes(original));
        let value = Violation {
            rule_id: "V1".to_owned(),
            level: ViolationLevel::Error,
            message: "non-utf8".to_owned(),
            location: Location {
                path: Cow::Owned(path.to_owned()),
                line: 1,
                column: 0,
            },
        };
        let text = format_text(std::slice::from_ref(&value));
        assert!(text.contains(r"a\xFFb.rs"), "text output: {text:?}");
        let sarif: serde_json::Value = serde_json::from_str(&format_sarif(&[value])).unwrap();
        assert_eq!(
            sarif["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["artifactLocation"]
                ["uri"],
            "a%FFb.rs"
        );

        let percent = Violation {
            location: Location {
                path: Cow::Owned(Path::new("literal%name.rs").to_owned()),
                line: 1,
                column: 0,
            },
            ..violation(
                "V1",
                ViolationLevel::Error,
                "percent",
                Path::new("unused.rs"),
                1,
                0,
            )
        };
        let sarif: serde_json::Value = serde_json::from_str(&format_sarif(&[percent])).unwrap();
        assert_eq!(
            sarif["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["artifactLocation"]
                ["uri"],
            "literal%25name.rs"
        );
    }
}

#[test]
fn sarif_catalog_levels_positions_and_result_order_are_coherent() {
    let levels = [
        ViolationLevel::Fatal,
        ViolationLevel::Error,
        ViolationLevel::Warning,
        ViolationLevel::Info,
    ];
    let violations: Vec<_> = (0..=27)
        .map(|index| {
            violation(
                &format!("V{index}"),
                levels[index % 4].clone(),
                &format!("message-{index}"),
                Path::new("catalog.rs"),
                index + 10,
                index + 2,
            )
        })
        .collect();
    let document: serde_json::Value = serde_json::from_str(&format_sarif(&violations)).unwrap();
    assert_eq!(document["version"], "2.1.0");
    assert_eq!(document["runs"].as_array().unwrap().len(), 1);
    let run = &document["runs"][0];
    let rules = run["tool"]["driver"]["rules"].as_array().unwrap();
    let ids: Vec<_> = rules
        .iter()
        .map(|rule| rule["id"].as_str().unwrap())
        .collect();
    let unique: BTreeSet<_> = ids.iter().map(|id| (*id).to_owned()).collect();
    assert_eq!(ids.len(), 28);
    assert_eq!(
        unique,
        (0..=27)
            .map(|index| format!("V{index}"))
            .collect::<BTreeSet<_>>()
    );

    let results = run["results"].as_array().unwrap();
    assert_eq!(results.len(), violations.len());
    for (index, result) in results.iter().enumerate() {
        let expected_id = format!("V{index}");
        assert_eq!(result["ruleId"], expected_id);
        assert!(unique.contains(expected_id.as_str()));
        let expected_level = match levels[index % 4] {
            ViolationLevel::Fatal | ViolationLevel::Error => "error",
            ViolationLevel::Warning => "warning",
            ViolationLevel::Info => "note",
        };
        assert_eq!(result["level"], expected_level);
        let region = &result["locations"][0]["physicalLocation"]["region"];
        assert_eq!(region["startLine"], index + 10);
        assert_eq!(region["startColumn"], index + 3);
    }
}

#[test]
fn should_fail_matches_the_complete_table_and_set_laws() {
    let levels = [
        ViolationLevel::Fatal,
        ViolationLevel::Error,
        ViolationLevel::Warning,
        ViolationLevel::Info,
    ];
    let unit = [[true, true], [true, true], [false, true], [false, false]];
    for (index, level) in levels.iter().enumerate() {
        let values = vec![violation("V", level.clone(), "m", Path::new("p.rs"), 1, 0)];
        assert_eq!(should_fail(&values, &FailLevel::Error), unit[index][0]);
        assert_eq!(should_fail(&values, &FailLevel::Warning), unit[index][1]);
    }
    for mask in 0_u8..16 {
        let values: Vec<_> = levels
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .map(|(_, level)| violation("V", level.clone(), "m", Path::new("p.rs"), 1, 0))
            .collect();
        for mode in [FailLevel::Error, FailLevel::Warning] {
            let decision = should_fail(&values, &mode);
            let reversed: Vec<_> = values.iter().cloned().rev().collect();
            let duplicated: Vec<_> = values
                .iter()
                .cloned()
                .chain(values.iter().cloned())
                .collect();
            assert_eq!(should_fail(&reversed, &mode), decision);
            assert_eq!(should_fail(&duplicated, &mode), decision);
        }
        assert!(
            !should_fail(&values, &FailLevel::Error) || should_fail(&values, &FailLevel::Warning)
        );
    }
}
