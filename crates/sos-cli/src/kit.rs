//! `sos install | update | check` — vendor the harness into a repo and keep it current.
//!
//! Ownership (docs/plans/V3_TRIAGE_2026-10-08.md §H):
//! - **Kit-owned**: everything under `harness-lite/` that is listed in [`KIT_FILES`]. Recorded
//!   with SHA-256 in `harness-lite/UPSTREAM.json`. `update` replaces a file only when it is
//!   unmodified since the last install/update; a locally edited file is kept and the new
//!   version is written next to it as `<file>.sos-new`.
//! - **Project-owned**: `.claude/settings.json`, `.codex/hooks.json`, `AGENTS.md`, `CLAUDE.md`,
//!   `.sos.toml`, `.claude/agents/*.md`. Created from the templates when missing, never
//!   overwritten. Their templates also ship under `harness-lite/templates/` for reference.
//!
//! Every file the binary installs is embedded at build time, so `sos` needs no kit checkout.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// println! that ignores a closed stdout (`sos update | head`) instead of panicking.
macro_rules! say {
    ($($t:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(std::io::stdout(), $($t)*);
    }};
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const COMMIT: &str = env!("SOS_KIT_COMMIT");
pub const UPSTREAM: &str = "harness-lite/UPSTREAM.json";
pub const HOOKS_PATH: &str = "harness-lite/hooks";

pub struct File {
    pub path: &'static str,
    pub body: &'static str,
    pub exec: bool,
}

macro_rules! kit {
    ($dst:literal, $src:literal) => {
        File { path: $dst, body: include_str!(concat!("../../../", $src)), exec: false }
    };
    ($dst:literal, $src:literal, exec) => {
        File { path: $dst, body: include_str!(concat!("../../../", $src)), exec: true }
    };
}

/// Kit-owned files: destination in the target repo ← source in sos-kit.
pub const KIT_FILES: &[File] = &[
    kit!("harness-lite/README.md", "harness-lite/README.md"),
    kit!("harness-lite/CONTRACT.md", "harness-lite/CONTRACT.md"),
    kit!("harness-lite/roles/orchestrator.md", "harness-lite/roles/orchestrator.md"),
    kit!("harness-lite/roles/architect.md", "harness-lite/roles/architect.md"),
    kit!("harness-lite/roles/worker.md", "harness-lite/roles/worker.md"),
    kit!("harness-lite/roles/reviewer.md", "harness-lite/roles/reviewer.md"),
    kit!("harness-lite/scripts/env-guard.sh", "scripts/env-guard.sh", exec),
    kit!("harness-lite/scripts/status.sh", "scripts/status.sh", exec),
    kit!("harness-lite/scripts/test-watch.py", "scripts/test-watch.py", exec),
    kit!("harness-lite/scripts/advise", "scripts/advise", exec),
    kit!("harness-lite/adapters/README.md", "adapters/README.md"),
    kit!("harness-lite/adapters/claude/hook.py", "adapters/claude/hook.py", exec),
    kit!("harness-lite/adapters/codex/hook.py", "adapters/codex/hook.py", exec),
    kit!("harness-lite/hooks/pre-commit", "templates/app/hooks/pre-commit", exec),
    kit!("harness-lite/hooks/pre-push", "templates/app/hooks/pre-push", exec),
    kit!("harness-lite/templates/AGENTS.md", "templates/app/AGENTS.md"),
    kit!("harness-lite/templates/CLAUDE.md", "templates/app/CLAUDE.md"),
    kit!("harness-lite/templates/sos.toml", "templates/app/sos.toml"),
    kit!("harness-lite/templates/BACKLOG.md", "templates/app/BACKLOG.md"),
    kit!("harness-lite/templates/FEATURES.json", "templates/app/FEATURES.json"),
    kit!("harness-lite/templates/FEATURES.example.json", "templates/app/FEATURES.example.json"),
    kit!("harness-lite/templates/claude-settings.json", "templates/app/claude-settings.json"),
    kit!("harness-lite/templates/codex-hooks.json", "templates/app/codex-hooks.json"),
    kit!("harness-lite/templates/claude-agents/architect.md", "templates/app/claude-agents/architect.md"),
    kit!("harness-lite/templates/claude-agents/worker.md", "templates/app/claude-agents/worker.md"),
    kit!("harness-lite/templates/claude-agents/reviewer.md", "templates/app/claude-agents/reviewer.md"),
];

/// Project-owned files: created from a template when missing, never overwritten.
pub const PROJECT_FILES: &[File] = &[
    kit!("AGENTS.md", "templates/app/AGENTS.md"),
    kit!("CLAUDE.md", "templates/app/CLAUDE.md"),
    kit!(".sos.toml", "templates/app/sos.toml"),
    kit!("docs/BACKLOG.md", "templates/app/BACKLOG.md"),
    kit!("docs/FEATURES.json", "templates/app/FEATURES.json"),
    kit!(".claude/settings.json", "templates/app/claude-settings.json"),
    kit!(".codex/hooks.json", "templates/app/codex-hooks.json"),
    kit!(".claude/agents/architect.md", "templates/app/claude-agents/architect.md"),
    kit!(".claude/agents/worker.md", "templates/app/claude-agents/worker.md"),
    kit!(".claude/agents/reviewer.md", "templates/app/claude-agents/reviewer.md"),
];

pub const GITIGNORE_LINES: &[&str] = &[".advise-state/", "evidence/advice/", "*.sos-new"];

#[derive(Serialize, Deserialize, Debug)]
pub struct Upstream {
    pub kit: String,
    pub version: String,
    pub commit: String,
    /// path → sha256 of the kit version last written (not of local edits).
    pub files: BTreeMap<String, String>,
}

pub fn sha(s: &str) -> String {
    hex(&Sha256::digest(s.as_bytes()))
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn sha_file(p: &Path) -> Result<Option<String>> {
    match std::fs::read(p) {
        Ok(b) => Ok(Some(hex(&Sha256::digest(&b)))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("reading {}", p.display())),
    }
}

/// Resolve a repo-relative path for writing or deleting: no absolute paths or `..`, no
/// symlink at the target, and the parent directory must resolve inside the repository
/// (so a symlinked file or directory cannot redirect a write or delete outside it).
fn safe_path(root: &Path, rel: &str) -> Result<PathBuf> {
    let rp = Path::new(rel);
    if rp.is_absolute() || rp.components().any(|c| !matches!(c, std::path::Component::Normal(_))) {
        bail!("refusing unsafe path {rel:?}");
    }
    let p = root.join(rp);
    if let Ok(m) = std::fs::symlink_metadata(&p) {
        if m.file_type().is_symlink() {
            bail!("{rel} is a symlink; replace it with a regular file (sos does not write through symlinks)");
        }
    }
    let parent = p.parent().context("path has no parent")?;
    std::fs::create_dir_all(parent)?;
    let canon_root = root.canonicalize()?;
    if !parent.canonicalize()?.starts_with(&canon_root) {
        bail!("{rel} resolves outside the repository (symlinked directory)");
    }
    Ok(p)
}

fn write(root: &Path, f: &File, path: &str) -> Result<()> {
    let p = safe_path(root, path)?;
    std::fs::write(&p, f.body).with_context(|| format!("writing {path}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if f.exec { 0o755 } else { 0o644 };
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(mode))?;
    }
    Ok(())
}

fn git_out(root: &Path, args: &[&str]) -> Option<String> {
    let o = Command::new("git").current_dir(root).args(args).output().ok()?;
    // Strip only the trailing newline: paths may legitimately end in spaces.
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).trim_end_matches(['\n', '\r']).to_string())
}

/// The target must be the top level of a git work tree.
pub fn repo_root(dir: &Path) -> Result<PathBuf> {
    let top = git_out(dir, &["rev-parse", "--show-toplevel"])
        .with_context(|| format!("{} is not inside a git repository (run `git init` first)", dir.display()))?;
    let top = PathBuf::from(top).canonicalize()?;
    if top != dir.canonicalize()? {
        bail!("run sos from the repository top level: {}", top.display());
    }
    Ok(top)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Act {
    Add,
    Update,
    Same,
    Conflict,
    Remove,
    KeepModified,
    KeptEdited,
    Create,
    Exists,
}

impl Act {
    fn label(self) -> &'static str {
        match self {
            Act::Add => "add",
            Act::Update => "update",
            Act::Same => "unchanged",
            Act::Conflict => "CONFLICT (kept yours, new version → .sos-new)",
            Act::Remove => "remove (dropped upstream)",
            Act::KeepModified => "keep (dropped upstream, but you edited it)",
            Act::KeptEdited => "kept (edited locally; kit version unchanged)",
            Act::Create => "create",
            Act::Exists => "exists, left as is",
        }
    }
}

pub struct Plan {
    pub kit: Vec<(String, Act)>,
    pub project: Vec<(String, Act)>,
    pub notes: Vec<String>,
}

/// Decide what install (`prev = None`) or update (`prev = Some`) would do. No writes.
pub fn plan(root: &Path, prev: Option<&Upstream>) -> Result<Plan> {
    // Refuse before writing anything if a target is a symlink (writes never follow them).
    for f in KIT_FILES.iter().chain(if prev.is_none() { PROJECT_FILES } else { &[] }) {
        if std::fs::symlink_metadata(root.join(f.path)).is_ok_and(|m| m.file_type().is_symlink()) {
            bail!("{} is a symlink; sos does not write through symlinks — replace it with a regular file", f.path);
        }
    }
    let mut kit = Vec::new();
    for f in KIT_FILES {
        let local = sha_file(&root.join(f.path))?;
        let new = sha(f.body);
        let recorded = prev.and_then(|u| u.files.get(f.path));
        let act = match (local.as_deref(), recorded) {
            (None, _) => Act::Add,
            (Some(l), _) if l == new => Act::Same,
            (Some(l), Some(r)) if l == r => Act::Update, // unmodified since last write
            (Some(_), Some(r)) if r == &new => Act::KeptEdited, // edited locally, kit unchanged
            _ => Act::Conflict,                          // edited locally and kit changed, or a legacy copy
        };
        kit.push((f.path.to_string(), act));
    }
    if let Some(u) = prev {
        for (path, recorded) in &u.files {
            if KIT_FILES.iter().any(|f| f.path == path) {
                continue;
            }
            let p = Path::new(path);
            if !path.starts_with("harness-lite/") || p.components().any(|c| !matches!(c, std::path::Component::Normal(_))) {
                bail!("{UPSTREAM} lists {path:?}, which is outside harness-lite/ — fix or delete that entry");
            }
            match sha_file(&root.join(path))? {
                None => {}
                Some(l) if &l == recorded => kit.push((path.clone(), Act::Remove)),
                Some(_) => kit.push((path.clone(), Act::KeepModified)),
            }
        }
    }
    let mut project = Vec::new();
    let mut notes = Vec::new();
    if prev.is_none() {
        for f in PROJECT_FILES {
            let exists = root.join(f.path).try_exists().with_context(|| format!("checking {}", f.path))?;
            project.push((f.path.to_string(), if exists { Act::Exists } else { Act::Create }));
        }
        notes.extend(wiring_notes(root, true));
    }
    Ok(Plan { kit, project, notes })
}

/// Advice for project-owned files that exist but are not wired to the kit.
fn wiring_notes(root: &Path, installing: bool) -> Vec<String> {
    let mut n = Vec::new();
    let has = |p: &str, needle: &str| std::fs::read_to_string(root.join(p)).map(|s| s.contains(needle)).unwrap_or(false);
    let exists = |p: &str| root.join(p).exists();
    let verb = if installing { "exists" } else { "is" };
    if installing && exists("AGENTS.md") && !has("AGENTS.md", "harness-lite") {
        n.push(format!("AGENTS.md {verb} not pointing at harness-lite/: add the \"Harness\" section from harness-lite/templates/AGENTS.md"));
    }
    if installing && exists(".claude/settings.json") && !has(".claude/settings.json", "harness-lite/adapters/claude/hook.py") {
        n.push(format!(".claude/settings.json {verb} not wired: merge the hooks from harness-lite/templates/claude-settings.json (replace old claude-hooks entries)"));
    }
    if exists(".quality-gate.toml") && sos_gates::config::load(root).map(|c| c.text.files.is_empty()).unwrap_or(false) {
        n.push(".quality-gate.toml rules are read, but the wording gate checks nothing until .sos.toml [text].files lists the files (globs) your old hook scanned".into());
    }
    if installing && exists(".codex/hooks.json") && !has(".codex/hooks.json", "harness-lite/adapters/codex/hook.py") {
        n.push(format!(".codex/hooks.json {verb} not wired: merge the hooks from harness-lite/templates/codex-hooks.json"));
    }
    n
}

/// Effective core.hooksPath (local, global or system): a global hooks dir counts as in use.
fn hooks_path(root: &Path) -> Option<String> {
    git_out(root, &["config", "--get", "core.hooksPath"])
}

pub struct Opts {
    pub dry_run: bool,
    pub force_hooks: bool,
}

fn print_plan(p: &Plan) {
    for (path, act) in p.kit.iter().filter(|(_, a)| *a != Act::Same) {
        say!("  {:<52} {}", path, act.label());
    }
    let same = p.kit.iter().filter(|(_, a)| *a == Act::Same).count();
    if same > 0 {
        say!("  ({same} kit file(s) already up to date)");
    }
    for (path, act) in &p.project {
        say!("  {:<52} {}", path, act.label());
    }
}

fn apply_kit(root: &Path, p: &Plan) -> Result<BTreeMap<String, String>> {
    let mut recorded = BTreeMap::new();
    for (path, act) in &p.kit {
        let file = KIT_FILES.iter().find(|f| f.path == path);
        match (act, file) {
            (Act::Add | Act::Update, Some(f)) => write(root, f, f.path)?,
            (Act::Conflict, Some(f)) => {
                let new = format!("{}.sos-new", f.path);
                if root.join(&new).exists() {
                    say!("  {new} already exists (unfinished merge?): left untouched");
                } else {
                    write(root, f, &new)?;
                }
            }
            (Act::Remove, _) => std::fs::remove_file(safe_path(root, path)?)?,
            _ => {}
        }
        if let Some(f) = file {
            recorded.insert(f.path.to_string(), sha(f.body));
        }
    }
    Ok(recorded)
}

fn write_upstream(root: &Path, files: BTreeMap<String, String>) -> Result<()> {
    let u = Upstream { kit: "sos-kit".into(), version: VERSION.into(), commit: COMMIT.into(), files };
    std::fs::write(root.join(UPSTREAM), serde_json::to_string_pretty(&u)? + "\n")?;
    Ok(())
}

fn ensure_gitignore(root: &Path) -> Result<Vec<&'static str>> {
    let p = root.join(".gitignore");
    let cur = read_gitignore(root)?;
    let missing: Vec<&str> = GITIGNORE_LINES.iter().copied().filter(|l| !cur.lines().any(|x| x.trim() == *l)).collect();
    if !missing.is_empty() {
        let mut s = cur.clone();
        if !s.is_empty() && !s.ends_with('\n') {
            s.push('\n');
        }
        s.push_str("\n# SOS Kit (advisor state, logged advice, update conflicts)\n");
        for l in &missing {
            s.push_str(l);
            s.push('\n');
        }
        std::fs::write(&p, s)?;
    }
    Ok(missing)
}

/// Git's hook names (githooks(5)); other files in a hooks dir are not hooks.
const GIT_HOOKS: &[&str] = &[
    "applypatch-msg", "pre-applypatch", "post-applypatch", "pre-commit", "pre-merge-commit",
    "prepare-commit-msg", "commit-msg", "post-commit", "pre-rebase", "post-checkout", "post-merge",
    "pre-push", "pre-receive", "update", "proc-receive", "post-receive", "post-update",
    "reference-transaction", "push-to-checkout", "pre-auto-gc", "post-rewrite", "sendemail-validate",
    "fsmonitor-watchman", "post-index-change",
];

/// What install does with git hooks.
enum HookPlan {
    /// Nothing active to lose: point core.hooksPath at harness-lite/hooks.
    Set,
    /// Already pointing at harness-lite/hooks.
    Already,
    /// Active hooks elsewhere and no --force-hooks: leave git config alone.
    Keep { dir: PathBuf, hooks: Vec<String> },
    /// --force-hooks: copy active hooks next to ours (pre-commit/pre-push become *.local,
    /// which our hooks run after the gates), then switch core.hooksPath.
    Migrate { dir: PathBuf, copies: Vec<(PathBuf, String)> },
}

fn is_executable(p: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(p).map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0).unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        p.is_file()
    }
}

fn hook_plan(root: &Path, force: bool) -> Result<HookPlan> {
    let configured = hooks_path(root).filter(|h| !h.is_empty());
    if configured.as_deref() == Some(HOOKS_PATH) {
        return Ok(HookPlan::Already);
    }
    let dir = match &configured {
        Some(h) if h.starts_with("~/") => PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(&h[2..]),
        Some(h) if Path::new(h).is_absolute() => PathBuf::from(h),
        Some(h) => root.join(h),
        None => {
            let gp = git_out(root, &["rev-parse", "--git-path", "hooks"]).context("locating .git/hooks")?;
            let gp = PathBuf::from(gp);
            if gp.is_absolute() { gp } else { root.join(gp) }
        }
    };
    let hooks: Vec<String> = GIT_HOOKS.iter().filter(|h| is_executable(&dir.join(h))).map(|h| h.to_string()).collect();
    if configured.is_none() && hooks.is_empty() {
        return Ok(HookPlan::Set);
    }
    if !force {
        return Ok(HookPlan::Keep { dir, hooks });
    }
    let mut copies = Vec::new();
    for h in &hooks {
        let dst = match h.as_str() {
            "pre-commit" => format!("{HOOKS_PATH}/pre-commit.local"),
            "pre-push" => format!("{HOOKS_PATH}/pre-push.local"),
            other => format!("{HOOKS_PATH}/{other}"),
        };
        let src = dir.join(h);
        if let Ok(existing) = std::fs::read(root.join(&dst)) {
            if existing != std::fs::read(&src)? {
                bail!("{dst} already exists and differs from {} — merge them by hand, then rerun", src.display());
            }
        }
        copies.push((src, dst));
    }
    Ok(HookPlan::Migrate { dir, copies })
}

fn describe(hp: &HookPlan) -> String {
    match hp {
        HookPlan::Set => format!("git core.hooksPath → {HOOKS_PATH}"),
        HookPlan::Already => "git core.hooksPath already set".into(),
        HookPlan::Keep { dir, hooks } if hooks.is_empty() => format!(
            "git core.hooksPath stays {} (set by you): call `sos gate all` from its pre-commit and `sos gate local-secrets` from its pre-push, or rerun with --force-hooks",
            dir.display()
        ),
        HookPlan::Keep { dir, hooks } => format!(
            "existing hooks in {} stay active ({}); sos gates are NOT wired yet: rerun with --force-hooks to keep them as harness-lite/hooks/*.local and switch, or call `sos gate all` from your pre-commit and `sos gate local-secrets` from your pre-push",
            dir.display(),
            hooks.join(", ")
        ),
        HookPlan::Migrate { dir, copies } if copies.is_empty() => format!("git core.hooksPath {} → {HOOKS_PATH} (--force-hooks)", dir.display()),
        HookPlan::Migrate { dir, copies } => format!(
            "git core.hooksPath {} → {HOOKS_PATH} (--force-hooks); your hooks keep running: {}",
            dir.display(),
            copies.iter().map(|(s, d)| format!("{} → {d}", s.file_name().unwrap_or_default().to_string_lossy())).collect::<Vec<_>>().join(", ")
        ),
    }
}

fn read_gitignore(root: &Path) -> Result<String> {
    match std::fs::read_to_string(root.join(".gitignore")) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e).context("reading .gitignore (left unchanged; fix or remove the unreadable bytes)"),
    }
}

pub fn install(dir: &Path, o: &Opts) -> Result<i32> {
    let root = repo_root(dir)?;
    if root.join(UPSTREAM).exists() {
        bail!("{UPSTREAM} exists: the kit is already installed here — run `sos update`");
    }
    // Preflight: every check that can fail runs before the first write.
    let p = plan(&root, None)?;
    let hp = hook_plan(&root, o.force_hooks)?;
    read_gitignore(&root)?;
    say!("sos install {VERSION} ({COMMIT}) → {}", root.display());
    print_plan(&p);
    say!("  {}", describe(&hp));
    let conflicts = p.kit.iter().filter(|(_, a)| *a == Act::Conflict).count();
    if o.dry_run {
        for n in &p.notes {
            say!("  note: {n}");
        }
        say!("dry run: nothing written.");
        return Ok(0);
    }
    let recorded = apply_kit(&root, &p)?;
    for (path, act) in &p.project {
        if *act == Act::Create {
            let f = PROJECT_FILES.iter().find(|f| f.path == path).expect("project file");
            write(&root, f, f.path)?;
        }
    }
    let added = ensure_gitignore(&root)?;
    if !added.is_empty() {
        say!("  .gitignore += {}", added.join(", "));
    }
    if let HookPlan::Migrate { copies, .. } = &hp {
        for (src, dst) in copies {
            let target = safe_path(&root, dst)?;
            std::fs::copy(src, &target).with_context(|| format!("copying {} to {dst}", src.display()))?;
        }
    }
    if matches!(hp, HookPlan::Set | HookPlan::Migrate { .. }) {
        let ok = Command::new("git").current_dir(&root).args(["config", "--local", "core.hooksPath", HOOKS_PATH]).status()?.success();
        if !ok {
            bail!("could not set core.hooksPath (rerun `sos install`; nothing is recorded yet)");
        }
    }
    // Written last: if anything above failed, `sos install` can simply run again.
    write_upstream(&root, recorded)?;
    finish(&root, &p.notes, conflicts)
}

pub fn update(dir: &Path, o: &Opts) -> Result<i32> {
    let root = repo_root(dir)?;
    let src = std::fs::read_to_string(root.join(UPSTREAM))
        .with_context(|| format!("{UPSTREAM} not found — run `sos install` first"))?;
    let prev: Upstream = serde_json::from_str(&src).with_context(|| format!("parsing {UPSTREAM}"))?;
    let p = plan(&root, Some(&prev))?;
    say!("sos update {} ({}) → {VERSION} ({COMMIT})", prev.version, prev.commit);
    print_plan(&p);
    if o.dry_run {
        say!("dry run: nothing written.");
        return Ok(0);
    }
    let recorded = apply_kit(&root, &p)?;
    write_upstream(&root, recorded)?;
    ensure_gitignore(&root)?;
    // Unresolved merges from earlier updates count too, so `update` stays non-zero until done.
    let pending = KIT_FILES.iter().filter(|f| root.join(format!("{}.sos-new", f.path)).exists()).count();
    finish(&root, &wiring_notes(&root, false), pending)
}

fn finish(root: &Path, notes: &[String], conflicts: usize) -> Result<i32> {
    for n in notes {
        say!("  note: {n}");
    }
    if conflicts > 0 {
        say!("{conflicts} conflict(s): compare each <file> with <file>.sos-new, keep what you want, delete the .sos-new.");
    }
    say!("Next: `sos check` in {}, review the diff, commit. Reload running agent sessions so they pick up the hooks.", root.display());
    Ok(if conflicts > 0 { 1 } else { 0 })
}

fn on_path(bin: &str) -> bool {
    Command::new(bin).arg("--version").output().is_ok()
}

/// Health of an installed repo. Exit 1 on any error.
pub fn check(dir: &Path) -> Result<i32> {
    let root = repo_root(dir)?;
    let mut errors: Vec<String> = Vec::new();
    let mut warns: Vec<String> = Vec::new();
    match std::fs::read_to_string(root.join(UPSTREAM)) {
        Err(_) => errors.push(format!("{UPSTREAM} missing — run `sos install`")),
        Ok(src) => {
            let u: Upstream = serde_json::from_str(&src).with_context(|| format!("parsing {UPSTREAM}"))?;
            if u.version != VERSION || u.commit != COMMIT {
                warns.push(format!("installed kit {} ({}) differs from this sos {VERSION} ({COMMIT}) — `sos update`", u.version, u.commit));
            }
            for (path, recorded) in &u.files {
                match sha_file(&root.join(path))? {
                    None => errors.push(format!("{path} missing — `sos update` restores it")),
                    Some(l) if &l != recorded => warns.push(format!("{path} edited locally (update will not overwrite it)")),
                    _ => {}
                }
                if root.join(format!("{path}.sos-new")).exists() {
                    warns.push(format!("{path}.sos-new pending: merge it, then delete it"));
                }
            }
        }
    }
    let hp = hooks_path(&root).unwrap_or_default();
    let hook_dir = if hp.is_empty() { ".git/hooks".to_string() } else { hp.clone() };
    let pre_commit = std::fs::read_to_string(root.join(&hook_dir).join("pre-commit")).unwrap_or_default();
    let runs_gates = pre_commit.lines().map(str::trim).any(|l| !l.starts_with('#') && l.contains("sos gate all"));
    if !runs_gates {
        errors.push(format!("{hook_dir}/pre-commit does not run `sos gate all` — `git config core.hooksPath {HOOKS_PATH}`"));
    }
    let pre_push = std::fs::read_to_string(root.join(&hook_dir).join("pre-push")).unwrap_or_default();
    if !pre_push.lines().map(str::trim).any(|l| !l.starts_with('#') && l.contains("sos gate local-secrets")) {
        errors.push(format!("{hook_dir}/pre-push does not run `sos gate local-secrets`"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for h in ["pre-commit", "pre-push"] {
            let p = root.join(&hook_dir).join(h);
            if let Ok(m) = std::fs::metadata(&p) {
                if m.permissions().mode() & 0o111 == 0 {
                    errors.push(format!("{hook_dir}/{h} is not executable — chmod +x it (git skips it silently)"));
                }
            }
        }
    }
    for (bin, why) in [("gitleaks", "secret gates"), ("python3", "agent adapters and advisor")] {
        if !on_path(bin) {
            errors.push(format!("{bin} not found on PATH (needed by {why})"));
        }
    }
    match sos_gates::config::load(&root) {
        Err(e) => errors.push(format!(".sos.toml: {e:#}")),
        Ok(cfg) => {
            if !root.join(&cfg.features.file).exists() {
                warns.push(format!("{} missing (template: harness-lite/templates/FEATURES.example.json)", cfg.features.file));
            }
        }
    }
    for (file, agent) in [(".claude/settings.json", "claude"), (".codex/hooks.json", "codex")] {
        let Ok(src) = std::fs::read_to_string(root.join(file)) else { continue };
        match serde_json::from_str::<serde_json::Value>(&src) {
            Err(e) => errors.push(format!("{file} is not valid JSON ({e}); the agent will ignore its hooks")),
            Ok(v) => {
                let needle = format!("harness-lite/adapters/{agent}/hook.py");
                let wired = v.get("hooks").and_then(|h| h.as_object()).is_some_and(|events| {
                    events.values().flat_map(|e| e.as_array().into_iter().flatten())
                        .flat_map(|m| m.get("hooks").and_then(|h| h.as_array()).into_iter().flatten())
                        .any(|h| h.get("command").and_then(|c| c.as_str()).is_some_and(|c| c.contains(&needle)))
                });
                if !wired {
                    warns.push(format!("{file} has no hook calling {needle} (template: harness-lite/templates/)"));
                }
            }
        }
    }
    // Role model and effort are decisions (worker sonnet/medium, checks high); unset, a subagent
    // silently runs the main session's model, as TurnSigil's first worker did.
    for role in ["worker", "reviewer", "architect"] {
        let Ok(src) = std::fs::read_to_string(root.join(format!(".claude/agents/{role}.md"))) else { continue };
        let front = src.strip_prefix("---\n").and_then(|r| r.split("\n---").next()).unwrap_or("");
        let missing: Vec<&str> = ["model", "effort"].into_iter()
            .filter(|k| !front.lines().any(|l| l.trim_start().starts_with(&format!("{k}:")))).collect();
        if !missing.is_empty() {
            warns.push(format!(".claude/agents/{role}.md sets no {}: it runs the main session's (template: harness-lite/templates/claude-agents/{role}.md)", missing.join(" or ")));
        }
    }
    if let Ok(c) = std::fs::read_to_string(root.join("CLAUDE.md")) {
        if !c.contains("@AGENTS.md") {
            warns.push("CLAUDE.md does not import AGENTS.md (add a line `@AGENTS.md`)".into());
        }
    }
    if let Ok(a) = std::fs::read_to_string(root.join("AGENTS.md")) {
        let open: Vec<&str> = ["<Project name>", "<One paragraph", "<e.g.", "<paths>", "<design, wording"]
            .into_iter().filter(|p| a.contains(p)).collect();
        if !open.is_empty() {
            warns.push(format!("AGENTS.md still has template placeholders ({}): fill them in", open.join(", ")));
        }
        if !a.contains("harness-lite") {
            warns.push("AGENTS.md does not point at harness-lite/ (see harness-lite/templates/AGENTS.md)".into());
        }
    }
    if !root.join("docs/BACKLOG.md").exists() {
        warns.push("docs/BACKLOG.md missing: the session-start status reads its first section".into());
    }
    for w in &warns {
        say!("warning: {w}");
    }
    for e in &errors {
        say!("error: {e}");
    }
    if errors.is_empty() {
        say!("sos check: ok ({} warning(s))", warns.len());
        Ok(0)
    } else {
        say!("sos check: {} error(s)", errors.len());
        Ok(1)
    }
}
