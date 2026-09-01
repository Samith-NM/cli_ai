use crate::context::Contextpayload;
use crate::process::ProcessResult;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn write_report(
    binary: &str,
    result: &ProcessResult,
    payload: &Contextpayload,
    explanation_md: &str,
    out_dir: &Path,
) -> Result<PathBuf> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("couldn't create report directory {}", out_dir.display()))?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let binary_stem = Path::new(binary)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("program");
    let out_path = out_dir.join(format!("crash-{binary_stem}-{ts}.md"));

    let mut doc = String::new();

    doc.push_str(&format!("# Crash Report: `{binary}`\n\n"));
    doc.push_str(&format!("- **Generated:** unix time `{ts}`\n"));
    doc.push_str(&format!(
        "- **Signal:** {}\n",
        payload.signal_name.as_deref().unwrap_or("unknown")
    ));
    if let Some(code) = result.exit_code {
        doc.push_str(&format!("- **Exit code:** {code}\n"));
    }
    if let (Some(f), Some(l)) = (&payload.file, payload.line) {
        doc.push_str(&format!("- **Primary crash site:** `{f}:{l}`\n"));
    }
    doc.push('\n');

    doc.push_str("## AI Analysis\n\n");
    doc.push_str(explanation_md.trim());
    doc.push_str("\n\n");

    if !payload.references.is_empty() {
        doc.push_str("## Referenced Files\n\n");
        doc.push_str(&format!(
            "{} distinct file:line location(s) were found in the crash output:\n\n",
            payload.references.len()
        ));
        for (i, r) in payload.references.iter().enumerate() {
            doc.push_str(&format!("### {}. `{}:{}`\n\n", i + 1, r.file, r.line));
            match &r.snippet {
                Some(snippet) => {
                    doc.push_str("```\n");
                    doc.push_str(snippet);
                    doc.push_str("```\n\n");
                }
                None => {
                    doc.push_str("_source not available at report time_\n\n");
                }
            }
        }
    } else {
        doc.push_str(
            "## Referenced Files\n\nNo `file:line` locations were found in the crash output.\n\n",
        );
    }

    doc.push_str("## Raw stderr\n\n```\n");
    doc.push_str(&result.stderr);
    if !result.stderr.ends_with('\n') {
        doc.push('\n');
    }
    doc.push_str("```\n");

    std::fs::write(&out_path, doc)
        .with_context(|| format!("couldn't write report to {}", out_path.display()))?;

    Ok(out_path)
}