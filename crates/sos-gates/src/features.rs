//! FEATURES.json identity rules (was `claude-hooks features-guard`, a Claude-only PreToolUse
//! hook; now a git-level check, so every agent and the shell are covered).
//!
//! Compares the staged file with HEAD. Rules (union of features-guard and Payquill's hook):
//! valid JSON; `{"features":[...]}` or a top-level array; every entry an object with a
//! non-empty string `id` and `title`, boolean `passes`, and `verify` = array with at least one
//! non-empty string; ids unique; no id present at HEAD may disappear; with
//! `allow_regression = false`, a slice that passed may not go back to false.

use crate::config::FeaturesConfig;
use crate::{git, Report};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

fn entries(v: &Value) -> Option<&Vec<Value>> {
    v.get("features").and_then(Value::as_array).or_else(|| v.as_array())
}

fn id_of(f: &Value) -> Option<String> {
    f.get("id").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned)
}

/// Pure decision: errors for `new` given the previous version `old` (None = file is new).
pub fn decide(old: Option<&str>, new: &str, allow_regression: bool) -> Vec<String> {
    let mut errs = Vec::new();
    let new_v: Value = match serde_json::from_str(new) {
        Ok(v) => v,
        Err(e) => return vec![format!("not valid JSON ({e})")],
    };
    let Some(list) = entries(&new_v) else {
        return vec!["expected {\"features\": [...]} or a top-level array".into()];
    };
    let mut ids = HashSet::new();
    for (i, f) in list.iter().enumerate() {
        let label = id_of(f).unwrap_or_else(|| format!("#{}", i + 1));
        if !f.is_object() {
            errs.push(format!("entry {label} must be an object"));
            continue;
        }
        match id_of(f) {
            None => errs.push(format!("entry {label} has no string id")),
            Some(id) if !ids.insert(id.clone()) => errs.push(format!("duplicate id {id}")),
            _ => {}
        }
        if f.get("title").and_then(Value::as_str).is_none_or(|t| t.trim().is_empty()) {
            errs.push(format!("entry {label} has no title"));
        }
        if !f.get("passes").is_some_and(Value::is_boolean) {
            errs.push(format!("entry {label}: passes must be true or false"));
        }
        let verify_ok = f.get("verify").and_then(Value::as_array)
            .is_some_and(|a| a.iter().any(|s| s.as_str().is_some_and(|s| !s.trim().is_empty())));
        if !verify_ok {
            errs.push(format!("entry {label} has no verify steps (array of non-empty strings)"));
        }
    }
    let old_v = old.and_then(|o| serde_json::from_str::<Value>(o).ok());
    if let Some(old_list) = old_v.as_ref().and_then(entries) {
        for of in old_list {
            let Some(id) = id_of(of) else { continue };
            match list.iter().find(|nf| id_of(nf).as_deref() == Some(id.as_str())) {
                None => errs.push(format!("slice {id} was removed; slices may be added, never deleted")),
                Some(nf) => {
                    let was = of.get("passes").and_then(Value::as_bool) == Some(true);
                    let now = nf.get("passes").and_then(Value::as_bool) == Some(false);
                    if !allow_regression && was && now {
                        errs.push(format!("slice {id} passed and is now false (allow_regression = false)"));
                    }
                }
            }
        }
    }
    errs
}

/// Gate on the staged FEATURES file (no-op when it is not staged).
pub fn gate(root: &Path, cfg: &FeaturesConfig) -> Result<Report> {
    let mut r = Report::new("features");
    r.fix = format!(
        "keep every existing id in {} and give each entry id, title, passes (bool) and verify steps; \
         record a regression as passes=false with the reason in the current-state doc instead of deleting",
        cfg.file
    );
    let Some(change) = git::staged_changes(root)?.into_iter().find(|c| c.path == cfg.file) else {
        r.skipped = Some(format!("{} not staged", cfg.file));
        return Ok(r);
    };
    if change.status == 'D' {
        if git::head_blob(root, &cfg.file)?.is_some() {
            r.errors.push(format!("{}: the file is being deleted; slices may be added, never deleted", cfg.file));
        }
        return Ok(r);
    }
    let new = git::staged_blob(root, &cfg.file)?.unwrap_or_default();
    let old = git::head_blob(root, &cfg.file)?;
    r.errors = decide(old.as_deref(), &new, cfg.allow_regression)
        .into_iter()
        .map(|e| format!("{}: {e}", cfg.file))
        .collect();
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::decide;

    // Same fixture as claude-hooks src/hooks/lite.rs tests.
    const OLD: &str = r#"{"features":[
        {"id":"F01","title":"Engine","passes":true,"verify":["unit test"]},
        {"id":"F02","title":"Input","passes":false,"verify":["screen opens empty"]}
    ]}"#;

    #[test]
    fn flip_passed_back_blocked_only_without_allow_regression() {
        let new = OLD.replacen(r#""passes":true"#, r#""passes":false"#, 1);
        assert!(decide(Some(OLD), &new, false).iter().any(|e| e.contains("F01")));
        assert!(decide(Some(OLD), &new, true).is_empty());
    }

    #[test]
    fn deleting_slice_blocked() {
        let new = r#"{"features":[{"id":"F01","title":"Engine","passes":true,"verify":["unit test"]}]}"#;
        assert!(decide(Some(OLD), new, true).iter().any(|e| e.contains("F02")));
    }

    #[test]
    fn marking_pass_and_appending_allowed() {
        let new = r#"[
            {"id":"F01","title":"Engine","passes":true,"verify":["unit test"]},
            {"id":"F02","title":"Input","passes":true,"verify":["screen opens empty"]},
            {"id":"F03","title":"Table","passes":false,"verify":["360 rows scroll"]}
        ]"#;
        assert!(decide(Some(OLD), new, false).is_empty());
    }

    #[test]
    fn structure_rules() {
        let bad = r#"{"features":[
            {"id":"F01","title":"Engine","passes":true,"verify":[]},
            {"id":"F01","title":"","passes":"yes","verify":["x"]},
            "loose string"
        ]}"#;
        let e = decide(None, bad, true);
        assert!(e.iter().any(|x| x.contains("verify")), "{e:?}");
        assert!(e.iter().any(|x| x.contains("duplicate id F01")), "{e:?}");
        assert!(e.iter().any(|x| x.contains("no title")), "{e:?}");
        assert!(e.iter().any(|x| x.contains("passes must be")), "{e:?}");
        assert!(e.iter().any(|x| x.contains("must be an object")), "{e:?}");
    }

    #[test]
    fn invalid_json_and_wrong_shape_blocked() {
        assert!(!decide(Some(OLD), "{not json", true).is_empty());
        assert!(!decide(Some(OLD), r#"{"items":[]}"#, true).is_empty());
    }

    #[test]
    fn whitespace_in_id_does_not_count_as_new_id() {
        let new = OLD.replace(r#""id":"F02""#, r#""id":" F02 ""#);
        assert!(decide(Some(OLD), &new, true).is_empty());
    }
}
