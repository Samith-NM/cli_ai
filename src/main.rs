mod autodocumentation;
mod cli;
mod context;
mod gateway;
mod process;
mod trender;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            cli_ai,
            args,
            context,
            mock,
            no_reports,
            report_dir,
            timeout,
        } => {
            run(&cli_ai, &args, context, mock, !no_reports, &report_dir, timeout).await?;
        }
    }

    Ok(())
}

async fn run(
    file: &str,
    args: &[String],
    context_lines: usize,
    mock: bool,
    write_report: bool,
    report_dir: &str,
    timeout_secs: u64,
) -> Result<()> {
    println!("running {file} in sandbox...\n");

    let result = process::run_processed(file, args, timeout_secs).await?;

    if !result.stdout.is_empty() {
        println!("--- stdout ---\n{}", result.stdout);
    }

    let (signal_name, synthetic_stderr): (Option<String>, Option<String>) = if result.crashed() {
        (result.signal_name().map(|s| s.to_string()), None)
    } else if result.hung() {
        let msg = format!(
            "Process did not exit within {timeout_secs}s and was forcibly killed.\n\
             This points to an infinite loop, deadlock, or a blocking wait \
             (I/O, lock, channel) that never resolves — not a memory-fault crash.\n\n\
             --- partial stdout captured before kill ---\n{}\n\
             --- partial stderr captured before kill ---\n{}\n",
            result.stdout, result.stderr
        );
        (Some(format!("TIMEOUT (no exit after {timeout_secs}s)")), Some(msg))
    } else {
        println!(
            "exited normally (code {})",
            result.exit_code.map(|c| c.to_string()).unwrap_or_else(|| "unknown".into())
        );
        return Ok(());
    };

    println!(
        "{}: {}\n",
        if result.hung() { "hung" } else { "crashed" },
        signal_name.as_deref().unwrap_or("unknown")
    );

    let stderr_for_analysis = synthetic_stderr.as_deref().unwrap_or(&result.stderr);
    let payload = context::build_payload(stderr_for_analysis, signal_name.as_deref(), context_lines)?;

    if let (Some(f), Some(l)) = (&payload.file, payload.line) {
        println!("  crash site: {f}:{l}");
    }
    if payload.references.len() > 1 {
        println!("  ({} other referenced locations found)", payload.references.len() - 1);
    }
    if let Some(snippet) = &payload.extracted_snippet {
        println!("\n{snippet}");
    }

    println!("\n--- AI analysis ---\n");
    let explanation = gateway::explain_crash(&payload, mock).await?;
    if write_report {
        let out_dir = std::path::Path::new(report_dir);
        let path = autodocumentation::write_report(file, &result, &payload, &explanation, out_dir)?;
        println!("\nreport written to: {}", path.display());
    }

    Ok(())
}
