//! `sos gate blueprint` — the blueprint has its three parts before anyone reviews or builds on it
//! (plan `docs/plans/QUY-TRINH-LAM-APP-NHAP-2026-11-12.md`, change 1).
//!
//! Observed failure: the architect's drawing was reviewed only once building had started, and what
//! it left out (a journey with no example, no data model, an unprotected delete) was found in the
//! running app. The gate checks presence, not quality: journeys each with a concrete example and
//! one marked as the skeleton, a data model, the irreversible actions. Whether they are right is
//! Người soát's work. Text left from the template (`<…>`) does not count as filled.

use crate::config::BlueprintConfig;
use crate::Report;
use regex::Regex;
use std::path::Path;

/// Words a level-2 heading must contain, and what the part is called in messages.
const PARTS: [(&str, &str); 3] = [("journeys", "Journeys"), ("data model", "Data model"), ("irreversible", "Irreversible actions")];

/// A line's own content: template placeholders, list marks (`-`, `1.`, `2)`) and emphasis removed.
fn content(line: &str, placeholder: &Regex) -> String {
    let text = placeholder.replace_all(line, "");
    let text = text.trim().trim_start_matches(['-', '*', '+', '>']).trim_start();
    let unnumbered = text.trim_start_matches(|c: char| c.is_ascii_digit());
    let text = match unnumbered.strip_prefix(['.', ')']) {
        Some(rest) if unnumbered.len() < text.len() => rest,
        _ => text,
    };
    text.replace(['*', '_', '`'], "").trim().to_string()
}

fn says_something(text: &str) -> bool {
    text.chars().any(char::is_alphanumeric)
}

/// `Example: …` → what follows the label, if the line carries that label.
fn after_label<'a>(text: &'a str, label: &str) -> Option<&'a str> {
    let head = text.get(..label.len())?;
    head.eq_ignore_ascii_case(label).then(|| text[label.len()..].trim_start_matches(|c: char| c == 's' || c == ':' || c.is_whitespace()))
}

pub fn gate(root: &Path, cfg: &BlueprintConfig) -> Report {
    let mut r = Report::new("blueprint");
    let file = cfg.file.trim();
    let fix = |what: &str| {
        format!(
            "{what} in {file} (start from harness-lite/templates/BLUEPRINT.md; how to write it: harness-lite/roles/architect.md), \
             then run `sos gate blueprint` again. The blueprint lives elsewhere? Set `.sos.toml` [blueprint] file."
        )
    };
    let Ok(text) = std::fs::read_to_string(root.join(file)) else {
        r.errors.push(format!("{file} does not exist or is not readable text"));
        r.fix = fix("write the blueprint");
        return r;
    };
    let placeholder = Regex::new(r"<[^<>\n]*>").expect("static regex");

    // Level-2 sections, fenced code left out of heading detection.
    let mut sections: Vec<(String, Vec<&str>)> = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        }
        let heading = (!fenced).then(|| line.strip_prefix("## ").or_else(|| line.strip_prefix("# "))).flatten();
        match heading {
            Some(h) => sections.push((if line.starts_with("## ") { h.to_lowercase() } else { String::new() }, Vec::new())),
            None => {
                if let Some(s) = sections.last_mut() {
                    s.1.push(line);
                }
            }
        }
    }

    for (key, name) in PARTS {
        let Some((_, body)) = sections.iter().find(|(h, _)| h.contains(key)) else {
            r.errors.push(format!("no `## {name}` section"));
            continue;
        };
        if key != "journeys" {
            if !body.iter().any(|l| !l.starts_with('#') && says_something(&content(l, &placeholder))) {
                r.errors.push(format!("`## {name}` is empty or still the template text"));
            }
            continue;
        }
        // Journeys: `### ` headings, each with an `Example:` that says something.
        let mut journeys: Vec<(&str, Vec<&str>)> = Vec::new();
        for l in body {
            match l.strip_prefix("### ") {
                Some(h) => journeys.push((h.trim(), Vec::new())),
                None => {
                    if let Some(j) = journeys.last_mut() {
                        j.1.push(l);
                    }
                }
            }
        }
        if journeys.is_empty() {
            r.errors.push("`## Journeys` has no journey (a `### ` heading for each)".into());
            continue;
        }
        for (title, lines) in &journeys {
            if placeholder.is_match(title) || !says_something(title) {
                r.errors.push(format!("journey `{title}` has no name yet"));
            }
            // The example runs from its label to the next label of the template or the journey's end.
            let mut example = String::new();
            let mut inside = false;
            for l in lines {
                let c = content(l, &placeholder);
                if let Some(rest) = after_label(&c, "example") {
                    inside = true;
                    example.push_str(rest);
                } else if after_label(&c, "must survive").is_some() {
                    inside = false;
                } else if inside {
                    example.push_str(&c);
                }
            }
            if !says_something(&example) {
                r.errors.push(format!("journey `{title}` has no concrete example (a line starting `Example:`)"));
            }
        }
        if !journeys.iter().any(|(t, _)| t.to_lowercase().contains("skeleton")) {
            r.errors.push("no journey is marked as the skeleton (put `(skeleton)` in the heading of the one to build first)".into());
        }
    }
    if !r.errors.is_empty() {
        r.fix = fix("fill what is missing");
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEMPLATE: &str = include_str!("../../../templates/app/BLUEPRINT.md");

    const FILLED: &str = "# Blueprint — Ledgerly\n\n## Journeys\n\n### J1 Add an expense (skeleton)\n\n\
        Example: Lan types 45000 and \"phở\", taps Save; the list shows it first; one row is stored.\n\n\
        Must survive: empty amount.\n\n### J2 See the month\n\n**Example:**\n\n1. Lan opens Month.\n2. She sees 45,000 đ.\n\n\
        ## Data model\n\nOne SQLite table `expense(id, amount, note, day)`, owned by the device.\n\n\
        ## Irreversible actions\n\n- Delete an expense: undo for 5 seconds.\n";

    fn check(text: &str) -> Report {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("docs")).unwrap();
        std::fs::write(d.path().join("docs/BLUEPRINT.md"), text).unwrap();
        gate(d.path(), &BlueprintConfig::default())
    }

    #[test]
    fn a_filled_blueprint_passes_and_the_untouched_template_does_not() {
        let r = check(FILLED);
        assert!(r.ok(), "{:?}", r.errors);
        let r = check(TEMPLATE);
        // Every part of the template is a placeholder: both journeys, the data model, the irreversible actions.
        assert_eq!(r.errors.len(), 6, "{:?}", r.errors);
        assert!(r.fix.contains("harness-lite/templates/BLUEPRINT.md") && r.fix.contains("sos gate blueprint"));
    }

    #[test]
    fn a_missing_file_fails_closed_and_the_path_is_configurable() {
        let d = tempfile::tempdir().unwrap();
        let r = gate(d.path(), &BlueprintConfig::default());
        assert!(!r.ok() && r.errors[0].contains("docs/BLUEPRINT.md") && !r.fix.is_empty());
        std::fs::write(d.path().join("PLAN.md"), FILLED).unwrap();
        assert!(gate(d.path(), &BlueprintConfig { file: "PLAN.md".into() }).ok());
    }

    #[test]
    fn each_missing_part_is_named() {
        let errs = |text: String| check(&text).errors.join(" | ");
        assert!(errs(FILLED.replace("## Data model", "## Storage")).contains("no `## Data model` section"));
        assert!(errs(FILLED.replace("One SQLite table `expense(id, amount, note, day)`, owned by the device.", "<what is stored>")).contains("`## Data model` is empty"));
        assert!(errs(FILLED.replace("- Delete an expense: undo for 5 seconds.\n", "")).contains("`## Irreversible actions` is empty"));
        // A journey whose example is a placeholder, or missing; the text under `Must survive:` is not an example.
        assert!(errs(FILLED.replace("Lan types 45000 and \"phở\", taps Save; the list shows it first; one row is stored.", "<…>")).contains("journey `J1 Add an expense (skeleton)` has no concrete example"));
        assert!(errs(FILLED.replace("**Example:**", "Steps:")).contains("journey `J2 See the month` has no concrete example"));
        assert!(errs(FILLED.replace(" (skeleton)", "")).contains("no journey is marked as the skeleton"));
        assert!(errs(FILLED.replace("### J2 See the month", "### J2 <name>")).contains("has no name yet"));
        assert!(errs(FILLED.replace("### J1 Add an expense (skeleton)", "").replace("### J2 See the month", "")).contains("has no journey"));
        // A numbered list of placeholders is still the template; a numbered `Example:` is an example.
        let numbered = "## Journeys\n### Create (skeleton)\nExample:\n1. <input and result>\n## Data model\n1. <entity>\n## Irreversible actions\n2) <action and protection>\n";
        assert_eq!(check(numbered).errors.len(), 3, "{:?}", check(numbered).errors);
        assert!(check(&FILLED.replace("**Example:**\n\n1. Lan opens Month.", "1. **Example:** Lan opens Month.")).ok());
        // A heading inside a code fence does not open a section.
        assert!(errs(FILLED.replace("## Data model\n\n", "```\n## Data model\n```\n")).contains("no `## Data model` section"));
    }
}
