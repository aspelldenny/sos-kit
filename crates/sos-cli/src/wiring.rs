//! The agent hook commands in the project-owned `.claude/settings.json` and `.codex/hooks.json`:
//! what the kit writes, how to recognise the forms older kits wrote, and `sos update --wiring`,
//! which rewrites only those commands, in place (docs/plans/RUST_ARCHITECTURE_2026-11-05.md §3).

use anyhow::{bail, Result};
use serde_json::Value;
use std::path::Path;

pub const EVENTS: [&str; 4] = ["session-start", "pre-edit", "pre-bash", "post-bash"];
pub const FILES: [(&str, &str); 2] = [(".claude/settings.json", "claude"), (".codex/hooks.json", "codex")];
const INSTALL: &str = "https://github.com/aspelldenny/sos-kit#quick-start";

/// Guards (pre-edit, pre-bash) block on anything but "sos ran and returned 0": a missing `sos`
/// (127), an npm shim with no binary (1) and a `sos` too old for `--protocol` (clap's 2) all end
/// as exit 2. No `exec`, so the exit code can still be looked at. The other events run bare.
pub fn command(agent: &str, ev: &str) -> String {
    let call = format!("sos hook {agent} {ev} --protocol {}", crate::hook::PROTOCOL);
    if matches!(ev, "pre-edit" | "pre-bash") {
        format!("{call}; rc=$?; [ \"$rc\" = 0 ] || {{ [ \"$rc\" = 2 ] || echo \"BLOCKED: SOS Kit hook could not run \
                 (sos missing, too old, or failed: exit $rc). Install or upgrade sos: {INSTALL}\" >&2; exit 2; }}")
    } else {
        call
    }
}

/// Codex on Windows: which shell runs the hook is not documented, and a block must be deny JSON
/// with exit 0, which no one-line shell command does in cmd, PowerShell and bash alike. This
/// Python one-liner has no `$`, `%`, `!`, backslash or nested double quote, so all three hand it
/// over unchanged (tested 23/10/2026). Python here only launches `sos`; for pre-edit every failure
/// to run it (not found, nonzero exit, timeout, a crash) is deny JSON. pre-bash only reports: the
/// simulator guard has nothing to stop on Windows.
pub fn codex_windows(ev: &str, py: &str) -> Result<String> {
    one_word(py)?;
    let timeout = if ev == "post-bash" { 315 } else { 60 };
    Ok(format!("{py} -c \"import json,os,shutil,subprocess,sys;a='{ev}';x='sos hook codex {ev} --protocol {p}'.split();\
        d=lambda m:a=='pre-edit' and (print(json.dumps({{'hookSpecificOutput':{{'hookEventName':'PreToolUse','permissionDecision':'deny','permissionDecisionReason':m}}}})),\
        sys.stdout.flush(),os._exit(0)) or (print(ascii(m)[1:-1],file=sys.stderr),os._exit(1));\
        sys.excepthook=lambda t,v,b:d('BLOCKED: SOS Kit hook failed: '+repr(v));\
        w=shutil.which(x[0]);\
        w or d('BLOCKED: SOS Kit hook: sos is not on PATH. Install it: {INSTALL}');\
        c=subprocess.run([w]+x[1:],stdout=subprocess.PIPE,timeout={timeout});\
        c.returncode and d('BLOCKED: SOS Kit hook: sos exited '+str(c.returncode)+' (too old for --protocol {p}, or it failed). Upgrade sos: {INSTALL}');\
        sys.stdout.buffer.write(c.stdout)\"", p = crate::hook::PROTOCOL))
}

fn one_word(py: &str) -> Result<()> {
    if py.is_empty() || !py.chars().all(|c| c.is_ascii_alphanumeric() || "_./:-".contains(c)) {
        bail!("SOS_PYTHON for Codex hooks on Windows must be one word with forward slashes \
               (e.g. C:/Python312/python.exe): {py}");
    }
    Ok(())
}

/// The Windows Codex form the kit wrote up to 0.3.13 (calls the Python adapter). Only to recognise it.
fn old_codex_windows(ev: &str, py: &str) -> String {
    format!("{py} -c \"import json,os,subprocess,sys;a='{ev}';\
        d=lambda m:a=='pre-edit' and (print(json.dumps({{'hookSpecificOutput':{{'hookEventName':'PreToolUse','permissionDecision':'deny','permissionDecisionReason':m}}}})),\
        sys.stdout.flush(),os._exit(0)) or (print(ascii(m)[1:-1],file=sys.stderr),os._exit(1));\
        sys.excepthook=lambda t,v,b:d('BLOCKED: SOS Kit hook failed: '+repr(v));\
        r=subprocess.run('git rev-parse --show-toplevel',shell=True,capture_output=True,encoding='utf-8');\
        p=r.stdout.strip()+'/harness-lite/adapters/codex/hook.py';\
        r.returncode and d('BLOCKED: SOS Kit hook: not inside a git repository (or git is missing)');\
        os.path.isfile(p) or d('BLOCKED: SOS Kit hook: '+p+' not found');\
        c=subprocess.run([sys.executable,p,a],stdout=subprocess.PIPE,env=dict(os.environ,PYTHONUTF8='1'));\
        c.returncode and d('BLOCKED: SOS Kit hook: '+p+' exited '+str(c.returncode));\
        sys.stdout.buffer.write(c.stdout)\"")
}

/// The template's commands rewritten into the Windows Codex form.
pub fn codex_windows_template(body: &str, py: &str) -> Result<String> {
    let mut out = body.to_string();
    for ev in EVENTS {
        let old = serde_json::to_string(&command("codex", ev))?;
        if !out.contains(&old) {
            bail!("internal: .codex/hooks.json template has no {ev} command to rewrite");
        }
        out = out.replace(&old, &serde_json::to_string(&codex_windows(ev, py)?)?);
    }
    Ok(out)
}

#[derive(Debug, PartialEq)]
pub enum Form {
    /// What this kit writes now: `sos hook …`, shell form.
    Current,
    /// What this kit writes now on Windows for Codex; carries the interpreter.
    CurrentWindows(String),
    /// A kit adapter command from before `sos hook` (shell form); carries the interpreter word.
    Old(String),
    /// The pre-`sos hook` Windows Codex launcher; carries the interpreter.
    OldWindows(String),
}

/// Which kit form `cmd` is, for which event; `None` for anything else (left alone).
pub fn recognise(agent: &str, cmd: &str) -> Option<(&'static str, Form)> {
    for ev in EVENTS {
        if cmd == command(agent, ev) {
            return Some((ev, Form::Current));
        }
        let path = match agent {
            "claude" => "\"$CLAUDE_PROJECT_DIR/harness-lite/adapters/claude/hook.py\"",
            _ => "\"$(git rev-parse --show-toplevel)/harness-lite/adapters/codex/hook.py\"",
        };
        if let Some(word) = cmd.strip_suffix(&format!(" {path} {ev}")) {
            // Exactly what older installs wrote (`shell_word`): a plain word of [A-Za-z0-9_./:-], or
            // single quotes around anything without a quote. `check&&python3` is a custom command.
            let plain = |w: &str| !w.is_empty() && w.chars().all(|c| c.is_ascii_alphanumeric() || "_./:-".contains(c));
            let interp = match word.strip_prefix('\'').and_then(|w| w.strip_suffix('\'')) {
                Some(q) if !q.is_empty() && !q.contains('\'') && !plain(q) => Some(q),
                Some(_) => None,
                None => plain(word).then_some(word),
            };
            if let Some(i) = interp {
                return Some((ev, Form::Old(i.to_string())));
            }
        }
        if agent == "codex" {
            if let Some((py, _)) = cmd.split_once(" -c \"") {
                if cmd == old_codex_windows(ev, py) {
                    return Some((ev, Form::OldWindows(py.to_string())));
                }
                if codex_windows(ev, py).is_ok_and(|c| c == cmd) {
                    return Some((ev, Form::CurrentWindows(py.to_string())));
                }
            }
        }
    }
    None
}

/// Every hook command string in a settings/hooks JSON value: (event key, command).
pub fn commands(v: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(events) = v.get("hooks").and_then(Value::as_object) {
        for (key, entries) in events {
            for e in entries.as_array().into_iter().flatten() {
                for h in e.get("hooks").and_then(Value::as_array).into_iter().flatten() {
                    if let Some(c) = h.get("command").and_then(Value::as_str) {
                        out.push((key.clone(), c.to_string()));
                    }
                }
            }
        }
    }
    out
}

/// Some reason the agent wiring may still call `path` (a kit file `sos update` wants to remove):
/// a command naming it, or a hook file that cannot be read as JSON, so nobody can tell.
pub fn references(root: &Path, path: &str) -> Option<String> {
    // Compared loosely, to err on keeping: `\` as `/`, any case (Windows paths, review 1b point 5).
    let norm = |s: &str| s.replace('\\', "/").to_lowercase();
    let want = norm(path);
    for (file, _) in FILES {
        let src = match std::fs::read_to_string(root.join(file)) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Some(format!("{file} cannot be read ({e}), so whether it calls it is unknown")),
        };
        match serde_json::from_str::<Value>(&src) {
            Err(_) => return Some(format!("{file} is not valid JSON, so whether it calls it is unknown")),
            Ok(v) if commands(&v).iter().any(|(_, c)| norm(c).contains(&want)) => return Some(format!("{file} still calls it")),
            Ok(_) => {}
        }
    }
    None
}

fn short(c: &str) -> String {
    let one: String = c.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() > 70 { format!("{}…", one.chars().take(69).collect::<String>()) } else { one }
}

/// What `--wiring` would change in one file: the new JSON text and the lines describing it.
pub fn rewrite(agent: &str, src: &str, template: &str) -> Result<Option<(String, Vec<String>)>> {
    let mut v: Value = serde_json::from_str(src)?;
    let tpl: Value = serde_json::from_str(template)?;
    let mut changes = Vec::new();
    // Windows form when the file already uses it (a repo committed from Windows keeps it on any machine).
    let mut windows: Option<String> = None;
    let mut any_kit = false;
    // (a) Old kit commands → the current ones, same family.
    if let Some(events) = v.get_mut("hooks").and_then(Value::as_object_mut) {
        for (key, entries) in events.iter_mut() {
            for e in entries.as_array_mut().into_iter().flatten() {
                for h in e.get_mut("hooks").and_then(Value::as_array_mut).into_iter().flatten() {
                    let Some(cmd) = h.get("command").and_then(Value::as_str).map(str::to_string) else { continue };
                    let Some((ev, form)) = recognise(agent, &cmd) else { continue };
                    any_kit = true;
                    let new = match form {
                        Form::Current => continue,
                        Form::CurrentWindows(py) => {
                            windows = Some(py);
                            continue;
                        }
                        Form::Old(_) => command(agent, ev),
                        Form::OldWindows(py) => {
                            let c = codex_windows(ev, &py)?;
                            windows = Some(py);
                            c
                        }
                    };
                    changes.push(format!("{key}: {} → {}", short(&cmd), short(&new)));
                    h["command"] = Value::String(new);
                }
            }
        }
    }
    let windows = windows.or_else(|| (agent == "codex" && !any_kit && cfg!(windows)).then(crate::kit::hook_python));
    // (b) Kit events the file lacks (0.3.13 wiring had no pre-bash): the template's entry, appended.
    // A command that still calls the adapter in a way not recognised above counts as present.
    let present = |v: &Value, key: &str, ev: &str| commands(v).iter().any(|(k, c)| {
        k == key && (recognise(agent, c).is_some_and(|(e, _)| e == ev)
            || (c.contains(&format!("harness-lite/adapters/{agent}/hook.py")) && c.contains(ev)))
    });
    let mut custom = Vec::new();
    for (key, entries) in tpl.get("hooks").and_then(Value::as_object).into_iter().flatten() {
        for entry in entries.as_array().into_iter().flatten() {
            let mut entry = entry.clone();
            let Some(ev) = commands(&serde_json::json!({ "hooks": { key: [entry.clone()] } }))
                .first().and_then(|(_, c)| recognise(agent, c)).map(|(e, _)| e) else { continue };
            if present(&v, key, ev) {
                continue;
            }
            if let (Some(py), Some(hs)) = (&windows, entry.get_mut("hooks").and_then(Value::as_array_mut)) {
                for h in hs {
                    h["command"] = Value::String(codex_windows(ev, py)?);
                }
            }
            if !v.is_object() {
                bail!("the top level is not a JSON object");
            }
            let hooks = v.as_object_mut().unwrap().entry("hooks").or_insert_with(|| serde_json::json!({}));
            let Some(hooks) = hooks.as_object_mut() else { bail!("\"hooks\" is not a JSON object") };
            let arr = hooks.entry(key.clone()).or_insert_with(|| serde_json::json!([]));
            let Some(arr) = arr.as_array_mut() else { bail!("\"hooks\".{key} is not a JSON array") };
            arr.push(entry);
            changes.push(format!("{key}: + {ev} (missing)"));
        }
    }
    for (key, c) in commands(&v) {
        if c.contains(&format!("harness-lite/adapters/{agent}/hook.py")) && recognise(agent, &c).is_none() {
            custom.push(format!("{key}: left as is (not a kit command, still calls the adapter; edit it by hand): {}", short(&c)));
        }
    }
    let rewritten = !changes.is_empty();
    changes.extend(custom);
    if changes.is_empty() {
        return Ok(None);
    }
    let mut text = serde_json::to_string_pretty(&v)? + "\n";
    if src.contains("\r\n") {
        text = text.replace('\n', "\r\n");
    }
    Ok(Some((if rewritten { text } else { src.to_string() }, changes)))
}

/// `sos update --wiring`: rewrite the kit's own hook commands in place, shown first. Returns the
/// number of files that could not be handled.
pub fn update(root: &Path, dry_run: bool, templates: &[(&str, String)]) -> Result<usize> {
    let mut problems = 0;
    for (file, agent) in FILES {
        let path = root.join(file);
        let Ok(src) = std::fs::read_to_string(&path) else {
            crate::kit::say(&format!("  {file}: not present, left alone (`sos install` creates it in a new repo)"));
            continue;
        };
        if std::fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
            crate::kit::say(&format!("  {file}: is a symlink; sos does not write through symlinks — left alone"));
            problems += 1;
            continue;
        }
        let template = templates.iter().find(|(f, _)| f == &file).map(|(_, t)| t.as_str()).expect("template");
        match rewrite(agent, &src, template) {
            Err(e) => {
                crate::kit::say(&format!("  {file}: cannot edit ({e}); left as is — fix it by hand from harness-lite/templates/"));
                problems += 1;
            }
            Ok(None) => crate::kit::say(&format!("  {file}: hook wiring up to date")),
            Ok(Some((text, changes))) => {
                crate::kit::say(&format!("  {file}:"));
                for c in &changes {
                    crate::kit::say(&format!("    {c}"));
                }
                if !dry_run && text != src {
                    if let Err(e) = write_inside(root, &path, &text) {
                        crate::kit::say(&format!("  {file}: not written ({e:#})"));
                        problems += 1;
                    }
                }
            }
        }
    }
    Ok(problems)
}

/// Replace `path` (inside `root`, its folder not a symlink out of it) through a temp file created
/// fresh next to it: never through a symlink, never half-written (review 1b point 3).
fn write_inside(root: &Path, path: &Path, text: &str) -> Result<()> {
    use std::io::Write;
    let dir = path.parent().expect("file in a folder");
    let (croot, cdir) = (root.canonicalize()?, dir.canonicalize()?);
    if !cdir.starts_with(&croot) {
        bail!("{} resolves outside the repository; sos does not write through symlinks", dir.display());
    }
    let name = path.file_name().expect("file name").to_string_lossy();
    let tmp = cdir.join(format!(".{name}.sos-tmp-{}", std::process::id()));
    // create_new: fails on anything already there, a symlink included, instead of following it.
    let mut f = std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp)
        .map_err(|e| anyhow::anyhow!("creating {}: {e}", tmp.display()))?;
    let res = f.write_all(text.as_bytes()).and_then(|_| f.sync_all());
    drop(f);
    if let Err(e) = res.and_then(|_| std::fs::rename(&tmp, cdir.join(&*name))) {
        let _ = std::fs::remove_file(&tmp);
        bail!("writing {}: {e}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_every_form_and_nothing_else() {
        let old = "python3 \"$CLAUDE_PROJECT_DIR/harness-lite/adapters/claude/hook.py\" pre-edit";
        assert_eq!(recognise("claude", old), Some(("pre-edit", Form::Old("python3".into()))));
        let q = "'/opt/My Python/python' \"$CLAUDE_PROJECT_DIR/harness-lite/adapters/claude/hook.py\" post-bash";
        assert_eq!(recognise("claude", q), Some(("post-bash", Form::Old("/opt/My Python/python".into()))));
        assert_eq!(recognise("claude", &command("claude", "pre-bash")), Some(("pre-bash", Form::Current)));
        let w = old_codex_windows("pre-edit", "python");
        assert_eq!(recognise("codex", &w), Some(("pre-edit", Form::OldWindows("python".into()))));
        let n = codex_windows("pre-bash", "C:/Py/python.exe").unwrap();
        assert_eq!(recognise("codex", &n), Some(("pre-bash", Form::CurrentWindows("C:/Py/python.exe".into()))));
        for other in ["python3 money-guard.py", "echo hi; python3 \"$CLAUDE_PROJECT_DIR/harness-lite/adapters/claude/hook.py\" pre-edit",
                      "policy_check&&python3 \"$CLAUDE_PROJECT_DIR/harness-lite/adapters/claude/hook.py\" pre-edit",
                      "'' \"$CLAUDE_PROJECT_DIR/harness-lite/adapters/claude/hook.py\" pre-edit",
                      &command("codex", "pre-edit")] {
            assert_eq!(recognise("claude", other), None, "{other}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn write_inside_never_follows_a_planted_temp_symlink() {
        let root = std::env::temp_dir().join(format!("sos-wiring-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".claude")).unwrap();
        let root = root.canonicalize().unwrap();
        let victim = root.join("victim");
        std::fs::write(&victim, "keep me").unwrap();
        std::os::unix::fs::symlink(&victim, root.join(format!(".claude/.settings.json.sos-tmp-{}", std::process::id()))).unwrap();
        std::fs::write(root.join(".claude/settings.json"), "{}").unwrap();
        assert!(write_inside(&root, &root.join(".claude/settings.json"), "{\"x\": 1}").is_err());
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "keep me");
        assert_eq!(std::fs::read_to_string(root.join(".claude/settings.json")).unwrap(), "{}");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn windows_launcher_survives_cmd_and_powershell() {
        for ev in EVENTS {
            let c = codex_windows(ev, "python").unwrap();
            let body = c.strip_prefix("python -c \"").and_then(|b| b.strip_suffix('"')).unwrap();
            assert!(!body.contains(['$', '%', '!', '\\', '"', '&', '|', '<', '>', '^']), "{ev}: {body}");
        }
        assert!(codex_windows("pre-edit", "C:\\Py\\python.exe").is_err());
    }
}
