//! status — a few lines of live repo state for the start of an agent session (port of scripts/status.sh).
//!
//! Prints: repo, branch, uncommitted count, then the first "## " section of the state doc (its open
//! "- [ ]" items, or its first lines when it has none), and a note when that section or the entry
//! files outgrow their budget (same numbers as `sos check`). State doc: $SOS_STATE_FILE, else
//! .sos.toml [state] file, else docs/BACKLOG.md, else STATE.md. Also names the cross-family reviewer
//! the last `sos gate ready` result, and a broken advisor. Outside a git repo it prints nothing.

use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let o = Command::new("git").current_dir(dir).args(args).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).into_owned())
}

/// The table header a line opens, the way status.sh's awk reads it: comment and blanks removed.
fn header(line: &str) -> Option<String> {
    line.trim_start().starts_with('[').then(|| {
        let t = line.split('#').next().unwrap_or("");
        t.chars().filter(|c| !c.is_whitespace()).collect()
    })
}

/// The value of `key = "…"` (or '…') on a line, as the awk does: up to the first quote after `=`.
fn value_of(line: &str, key: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix(key)?;
    let rest = rest.trim_start();
    rest.strip_prefix('=')?;
    let after_eq = &line[line.find('=')? + 1..];
    let trimmed = after_eq.trim_start();
    let v = match trimmed.strip_prefix(['"', '\'']) {
        Some(v) => v,
        None => line,  // no quote: the awk prints the line itself
    };
    Some(v.split(['"', '\'']).next().unwrap_or("").to_string())
}

/// `key` in each listed table of a TOML file, in file order: (table, value).
fn toml_values(text: &str, tables: &[&str], key: &str) -> Vec<(String, String)> {
    let (mut t, mut out) = (String::new(), Vec::new());
    for line in text.split('\n') {
        if let Some(h) = header(line) {
            t = h;
        }
        if tables.contains(&t.as_str()) {
            if let Some(v) = value_of(line, key) {
                out.push((t.clone(), v));
            }
        }
    }
    out
}

/// Whether the first `cmd` of `[ready]` holds a command, in the same reading as status.sh's awk: it
/// has to follow TOML's quoting, since `"" # note` is empty, `"""make"""` is not, and a triple-quoted
/// value may run over the next lines.
fn ready_set(text: &str) -> bool {
    let (mut t, mut at) = (String::new(), 0);
    for line in text.split('\n') {
        if let Some(h) = header(line) {
            t = h;
        }
        if t == "[ready]" && value_of(line, "cmd").is_some() {
            let v = text[at + line.find('=').map_or(0, |i| i + 1)..].trim_start_matches([' ', '\t']);
            let Some(q) = v.chars().next().filter(|c| matches!(c, '"' | '\'')) else {
                return !v.lines().next().unwrap_or("").trim().is_empty();
            };
            let triple = q.to_string().repeat(3);
            let inner = match v.strip_prefix(triple.as_str()) {
                Some(rest) => rest.split(triple.as_str()).next().unwrap_or(""),
                None => v[1..].lines().next().unwrap_or("").split(q).next().unwrap_or(""),
            };
            return !inner.trim().is_empty();
        }
        at += line.len() + 1;
    }
    false
}

fn cut(line: &str) -> String {
    line.chars().take(220).collect()
}

fn truthy(v: Option<&serde_json::Value>, default: bool) -> bool {
    match v {
        None => default,
        Some(serde_json::Value::Null) => false,
        Some(serde_json::Value::Bool(b)) => *b,
        Some(serde_json::Value::Number(n)) => n.as_f64() != Some(0.0),
        Some(serde_json::Value::String(s)) => !s.is_empty(),
        Some(serde_json::Value::Array(a)) => !a.is_empty(),
        Some(serde_json::Value::Object(o)) => !o.is_empty(),
    }
}

fn as_text(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// The advisor's last failed call (not a role run), when it carries a fix.
fn advisor_note(root: &Path) -> Option<String> {
    let text = std::fs::read(root.join(".advise-state/calls.jsonl")).ok()?;
    let text = String::from_utf8_lossy(&text);
    let mut last: Option<serde_json::Map<String, serde_json::Value>> = None;
    for line in text.split('\n') {
        if let Ok(serde_json::Value::Object(d)) = serde_json::from_str(line) {
            if d.get("kind").and_then(|k| k.as_str()) != Some("role") {
                last = Some(d);
            }
        }
    }
    let last = last.filter(|d| !d.is_empty())?;
    if truthy(last.get("ok"), true) || !truthy(last.get("fix"), false) {
        return None;
    }
    let ts = last.get("ts").map(as_text).unwrap_or_else(|| "?".into());
    let err = last.get("error").filter(|e| truthy(Some(e), false)).map(as_text).unwrap_or_default();
    Some(format!(
        "Note: the advisor's last call failed ({ts}): {}. Fix: {}, then `sos advise --check`.",
        err.chars().take(120).collect::<String>(),
        as_text(&last["fix"])
    ))
}

/// The status lines for the repo containing `dir` (empty outside a repo).
pub fn render(dir: &Path) -> String {
    let Some(top) = git(dir, &["rev-parse", "--show-toplevel"]) else { return String::new() };
    let root = Path::new(top.trim_end_matches(['\n', '\r']));
    let mut out: Vec<String> = Vec::new();

    let branch = git(root, &["branch", "--show-current"]).map(|b| b.trim().to_string()).unwrap_or_else(|| "?".into());
    let dirty = git(root, &["status", "--porcelain"]).map(|s| s.matches('\n').count()).unwrap_or(0);
    let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    out.push(format!("{name} — branch: {} · uncommitted files: {dirty}",
                     if branch.is_empty() { "detached" } else { &branch }));

    let sos_toml = std::fs::read_to_string(root.join(".sos.toml")).ok();
    let mut state = std::env::var("SOS_STATE_FILE").unwrap_or_default();
    if state.is_empty() {
        if let Some(t) = &sos_toml {
            if let Some((_, v)) = toml_values(t, &["[state]"], "file").into_iter().next() {
                state = v;
            }
        }
    }
    if state.is_empty() {
        state = ["docs/BACKLOG.md", "STATE.md"].into_iter().find(|f| root.join(f).is_file()).unwrap_or("").to_string();
    }
    if !state.is_empty() && root.join(&state).is_file() {
        let body = String::from_utf8_lossy(&std::fs::read(root.join(&state)).unwrap_or_default()).into_owned();
        let mut lines: Vec<&str> = body.split('\n').collect();
        if body.ends_with('\n') {
            lines.pop();
        }
        let (mut n, mut k, mut more, mut other) = (0, 0, 0, Vec::new());
        for line in &lines {
            if line.starts_with("## ") {
                n += 1;
                if n == 1 {
                    out.push(cut(line));
                    continue;
                }
                break;
            }
            if n == 1 && line.starts_with("- [ ]") {
                k += 1;
                if k <= 10 { out.push(cut(line)) } else { more += 1 }
                continue;
            }
            if n == 1 && line.chars().any(|c| c != ' ' && c != '\t') && !line.starts_with("- [x]") && other.len() < 8 {
                other.push(cut(line));
            }
        }
        if more > 0 {
            out.push(format!("  … +{more} more open items"));
        }
        if k == 0 {
            out.extend(other);
        }
        out.push(format!("Detail: {state}"));
        let mut s = 0;
        let section = lines.iter().filter(|l| {
            if l.starts_with("## ") {
                s += 1;
            }
            s == 1
        }).count();
        if section > 80 {
            out.push(format!("Note: that section is {section} lines (keep under 80): open items only; done ones go to the changelog."));
        }
    }
    // A reviewer from the other family runs through `sos role`, not the session's own subagent: say so
    // where Quản đốc looks each session (0.3.6–0.3.7: `sos role` was never called in seven apps).
    if let Some(t) = &sos_toml {
        let route: Vec<String> = toml_values(t, &["[roles.reviewer.claude]", "[roles.reviewer.codex]"], "backend")
            .into_iter()
            .filter_map(|(table, v)| {
                let from = table.trim_start_matches("[roles.reviewer.").trim_end_matches(']').to_string();
                (v != from).then(|| format!("{from}-built work → {v}"))
            })
            .collect();
        if !route.is_empty() {
            out.push(format!("Người soát: {} — run it with `sos role reviewer --from <builder> --brief <file>`.", route.join("; ")));
        }
    }
    // Green slices do not prove the combined product works (Payquill F35 and F31): name the last
    // end-to-end result, or that nothing runs one.
    if let Some(t) = &sos_toml {
        if !ready_set(t) {
            out.push("Note: .sos.toml [ready] cmd is empty: nothing checks the whole product before handover.".into());
        } else {
            out.extend(crate::ready::last_note(root));
            // A product that has never run end to end is still a skeleton being built: say so each
            // session, so a second journey is not started on top of it.
            if !crate::ready::ever_passed(root) {
                out.push(crate::ready::NO_PASS.into());
            }
        }
    }
    let entry: usize = ["AGENTS.md", "CLAUDE.md"].iter()
        .filter_map(|f| std::fs::read(root.join(f)).ok())
        .map(|b| b.iter().filter(|&&c| c == b'\n').count())
        .sum();
    if entry > 200 {
        out.push(format!("Note: AGENTS.md + CLAUDE.md are {entry} lines (keep under 200): route to docs, move reference into them."));
    }
    // The advisor fails silently from a hook (a lost login, 15–17/10/2026): say so where Quản đốc looks.
    if let Some(note) = advisor_note(root) {
        out.push(note);
    }
    let mut text = out.join("\n");
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_toml_values_like_the_awk() {
        let t = "[state]\nfile = \"docs/PROGRESS.md\" # x\n[roles.reviewer.claude] # c\nbackend = 'codex'\n";
        assert_eq!(toml_values(t, &["[state]"], "file"), vec![("[state]".into(), "docs/PROGRESS.md".into())]);
        assert_eq!(toml_values(t, &["[roles.reviewer.claude]"], "backend")[0].1, "codex");
        assert!(toml_values("[state]\nfiles = \"x\"\n", &["[state]"], "file").is_empty());
    }

    #[test]
    fn reads_the_ready_command_in_each_toml_quoting() {
        let set = |v: &str| ready_set(&format!("[ready] # c\ncmd = {v}\n"));
        for v in ["\"make\"", "'make \"e2e\"'", "\"\"\"make ready\"\"\"", "'''make'''", "\"\"\"\nmake\n\"\"\""] {
            assert!(set(v), "{v}");
        }
        for v in ["\"\"", "''", "\"  \"", "\"\" # later", "\"\"\" \"\"\"", "\"\"\"\n\"\"\"", "\"\"\"\n  \n\"\"\""] {
            assert!(!set(v), "{v}");
        }
        assert!(!ready_set("[other]\ncmd = \"x\"\n"));
        assert!(!ready_set("[ready]\nscmd = \"x\"\n"));
    }
}
