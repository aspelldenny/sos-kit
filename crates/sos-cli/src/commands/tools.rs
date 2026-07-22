// commands/tools.rs — P077d3 (OA-07): `sos tools status`. NEW command,
// additive alongside `install.sh` (zero-touch); no Bash counterpart.
//
// Thin CLI surface over `sos_install::tools::{check_tools, required_drift}`
// — the SAME core the install-engine step-5 seam
// (`sos_install::engine::resolve_tools`) uses. One mechanism, two surfaces
// (Task 3/4 CHỐT: "một cơ chế, hai/ba surface").

use anyhow::Result;
use sos_install::tools::{self, Verdict};

pub fn run() -> Result<()> {
    let manifest = tools::embedded_manifest_parsed()?;
    let statuses = tools::check_tools(&manifest, None);

    println!(
        "{:<16} {:<9} {:<10} {:<12} VERDICT",
        "TOOL", "REQUIRED", "EXPECTED", "INSTALLED"
    );
    for s in &statuses {
        let installed = s.found.clone().unwrap_or_else(|| "-".to_string());
        let verdict_str = match s.verdict {
            Verdict::Ok => "OK",
            Verdict::Newer => "NEWER",
            Verdict::Drift => "DRIFT",
            Verdict::Missing => "MISSING",
            Verdict::Unparseable => "UNPARSEABLE",
        };
        println!(
            "{:<16} {:<9} {:<10} {:<12} {}",
            s.name,
            if s.required { "yes" } else { "no" },
            s.expected,
            installed,
            verdict_str
        );
        if !s.required && matches!(s.verdict, Verdict::Drift | Verdict::Missing | Verdict::Unparseable) {
            println!("  ⚠ optional, not at pinned version — warn only, exit code unaffected");
        }
    }

    if tools::required_drift(&statuses) {
        eprintln!();
        for s in statuses
            .iter()
            .filter(|s| s.required && matches!(s.verdict, Verdict::Drift | Verdict::Missing | Verdict::Unparseable))
        {
            eprintln!("✗ {}", tools::describe_failure(s));
        }
        eprintln!("(run the installer to fix — see install.sh / docs/SETUP.md)");
        std::process::exit(1);
    }

    Ok(())
}
