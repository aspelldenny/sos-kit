//! `sos check-links` on a small folder: which links count, which are broken, and the exit codes
//! (wave 3.2). Parity with the product-description skill's `check-links.py` is checked separately.
use std::fs;
use std::path::Path;
use std::process::Command;

fn write(root: &Path, rel: &str, body: &[u8]) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, body).unwrap();
}

fn sos(dir: &Path) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_sos")).arg("check-links").arg(dir).output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned())
}

const A: &str = "# Tiêu đề

[ok](b.md) [missing](nope.md) [anchor](b.md#ket-qua) [upper](./b.md#Ket-Qua) [bad](b.md#nope)
![image](gone.png) [web](https://example.com/x) [mail](mailto:a@b.c) [self](#tieu-đe) [dup](b.md#a-1)
[titled](sub/c.md \"t\") [folder](sub/) [txt](notes.txt#anything)

```
[in a fence](fenced-away.md)
```
";

const B: &str = "## Kết quả
## A
## A
";

#[test]
fn reports_missing_files_and_anchors() {
    let t = tempfile::tempdir().unwrap();
    let r = t.path();
    write(r, "a.md", A.as_bytes());
    write(r, "b.md", B.as_bytes());
    write(r, "sub/c.md", b"plain\n");
    write(r, "notes.txt", b"x\n");
    // Not walked: hidden folders and node_modules.
    write(r, ".hidden/x.md", b"[x](nope.md)\n");
    write(r, "node_modules/p/x.md", b"[x](nope.md)\n");
    let (rc, out) = sos(r);
    assert_eq!(rc, 1, "{out}");
    assert_eq!(out, "a.md:3: missing file: nope.md\na.md:3: missing anchor: b.md#nope\n\
3 files, 10 relative links, 2 broken\n");
}

#[test]
fn clean_folder_passes_and_bad_input_is_reported() {
    let t = tempfile::tempdir().unwrap();
    let r = t.path();
    write(r, "a.md", b"# One\n[b](b.md#one)\n");
    write(r, "b.md", b"# One\n");
    assert_eq!(sos(r), (0, "2 files, 1 relative links, 0 broken\n".into()));

    // A file that is not UTF-8 is broken, not a crash.
    write(r, "c.md", b"\xff\xfe[x](a.md)\n");
    let (rc, out) = sos(r);
    assert_eq!(rc, 1);
    assert!(out.starts_with("c.md: not UTF-8\n"), "{out}");

    // A folder that does not exist is an error, not "0 broken".
    let (rc, out) = sos(&r.join("missing"));
    assert_eq!((rc, out.as_str()), (2, ""));
}
