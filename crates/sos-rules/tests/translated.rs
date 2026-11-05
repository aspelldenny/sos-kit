//! Every translated rule under `tests/rules/<name>/` (rules.toml + cases.jsonl, with the old
//! script's result recorded per case by tests/rules/record_old.py) must still pass its cases.

use std::path::Path;

#[test]
fn translated_rules_pass_their_cases() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/rules");
    let mut dirs: Vec<_> = std::fs::read_dir(&root)
        .expect("tests/rules")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.join("rules.toml").is_file())
        .collect();
    dirs.sort();
    assert!(dirs.len() >= 5, "expected the five translated rules under {}", root.display());
    let mut failed = Vec::new();
    for d in &dirs {
        let cases = d.join("cases.jsonl");
        assert!(cases.is_file(), "{} has rules.toml but no cases.jsonl", d.display());
        if sos_rules::cases::run(&[d.join("rules.toml")], &cases) > 0 {
            failed.push(d.file_name().unwrap().to_string_lossy().into_owned());
        }
    }
    assert!(failed.is_empty(), "cases failed in: {failed:?} (see the FAIL lines above)");
}
