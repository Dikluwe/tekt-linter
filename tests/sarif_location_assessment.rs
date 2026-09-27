use crystalline_lint::{
    entities::violation::{Location, Violation, ViolationLevel},
    shell::cli::format_sarif,
};
use std::{borrow::Cow, path::Path};
fn finding(line: usize, column: usize) -> Violation<'static> {
    Violation {
        rule_id: "V0".into(),
        level: ViolationLevel::Fatal,
        message: "Diagnostic".into(),
        location: Location {
            path: Cow::Borrowed(Path::new("01_core/file.rs")),
            line,
            column,
        },
    }
}
#[test]
fn unknown_lines_preserve_artifact_without_inventing_a_region() {
    let doc: serde_json::Value =
        serde_json::from_str(&format_sarif(&[finding(0, 0), finding(0, usize::MAX)])).unwrap();
    for result in doc["runs"][0]["results"].as_array().unwrap() {
        let physical = &result["locations"][0]["physicalLocation"];
        assert_eq!(physical["artifactLocation"]["uri"], "01_core/file.rs");
        assert!(physical.get("region").is_none());
        assert_eq!(result["ruleId"], "V0");
        assert_eq!(result["level"], "error");
    }
}
#[test]
fn known_positions_are_preserved_in_mixed_reports() {
    let doc: serde_json::Value = serde_json::from_str(&format_sarif(&[
        finding(0, 0),
        finding(7, 0),
        finding(11, 9),
    ]))
    .unwrap();
    let results = &doc["runs"][0]["results"];
    assert_eq!(
        results[1]["locations"][0]["physicalLocation"]["region"],
        serde_json::json!({"startLine":7,"startColumn":1})
    );
    assert_eq!(
        results[2]["locations"][0]["physicalLocation"]["region"],
        serde_json::json!({"startLine":11,"startColumn":10})
    );
}
#[test]
fn unrepresentable_column_retains_known_line_without_overflow() {
    let doc: serde_json::Value =
        serde_json::from_str(&format_sarif(&[finding(3, usize::MAX)])).unwrap();
    let region = &doc["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"];
    assert_eq!(region["startLine"], 3);
    assert!(region.get("startColumn").is_none());
}
