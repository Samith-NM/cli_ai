use anyhow::{Context, Result};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

#[derive(Debug)]
pub struct ProcessResult {
    pub stdout: String,
    pub stderr: String,
    pub signal: Option<i32>,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
}

impl ProcessResult {
    pub fn crashed(&self) -> bool {
        if self.signal.is_some() {
            return true;
        }

        #[cfg(windows)]
        {
            matches!(
                self.exit_code,
                Some(-1073741819)
                    | Some(-1073740791)
                    | Some(-1073740762)
                    | Some(-1073741816)
            )
        }

        #[cfg(not(windows))]
        {
            false
        }
    }

    pub fn hung(&self) -> bool {
        self.timed_out
    }

    pub fn signal_name(&self) -> Option<&'static str> {
        #[cfg(unix)]
        {
            match self.signal {
                Some(4) => Some("SIGILL (illegal instruction)"),
                Some(6) => Some("SIGABRT (abort)"),
                Some(8) => Some("SIGFPE (floating point exception)"),
                Some(11) => Some("SIGSEGV (segmentation fault)"),
                Some(9) => Some("SIGKILL (killed)"),
                Some(_) => Some("unknown signal"),
                None => None,
            }
        }

        #[cfg(windows)]
        {
            match self.exit_code {
                Some(-1073741819) => Some("SIGSEGV (access violation / STATUS_ACCESS_VIOLATION)"),
                Some(-1073740791) => Some("SIGABRT (abort / STATUS_FATAL_APP_EXIT)"),
                Some(-1073740762) => Some("SIGILL (illegal instruction / STATUS_ILLEGAL_INSTRUCTION)"),
                Some(-1073741816) => Some("SIGFPE (floating point exception / STATUS_FLOAT_DIVIDE_BY_ZERO)"),
                _ => None,
            }
        }

        #[cfg(not(unix))]
        #[cfg(not(windows))]
        {
            None
        }
    }
}

pub async fn run_processed(path: &str, args: &[String], timeout_secs: u64) -> Result<ProcessResult> {
    let mut child = Command::new(path)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to launch `{path}`"))?;

    let stdout_pipe = child.stdout.take().context("stdout was piped")?;
    let stderr_pipe = child.stderr.take().context("stderr was piped")?;

    let stdout_task = tokio::spawn(async move {
        let mut buf = String::new();
        let mut pipe = stdout_pipe;
        let _ = pipe.read_to_string(&mut buf).await;
        buf
    });

    let stderr_task = tokio::spawn(async move {
        let mut buf = String::new();
        let mut pipe = stderr_pipe;
        let _ = pipe.read_to_string(&mut buf).await;
        buf
    });

    let (status, timed_out) = if timeout_secs == 0 {
        (child.wait().await.context("child process wait() failed")?, false)
    } else {
        match tokio::time::timeout(Duration::from_secs(timeout_secs), child.wait()).await {
            Ok(status) => (status.context("child process wait() failed")?, false),
            Err(_elapsed) => {
                child.kill().await.context("failed to kill the hung child process")?;
                let status = child.wait().await.context("wait() after kill failed")?;
                (status, true)
            }
        }
    };

    let stdout = stdout_task.await.unwrap_or_default();
    let stderr = stderr_task.await.unwrap_or_default();

    #[cfg(unix)]
    let signal = if timed_out { None } else { status.signal() };

    #[cfg(not(unix))]
    let signal = None;

    Ok(ProcessResult {
        stdout,
        stderr,
        signal,
        exit_code: if timed_out { None } else { status.code() },
        timed_out,
    })
}

#[cfg(test)]
mod tests {
    use super::ProcessResult;

    #[test]
    fn windows_access_violation_is_detected_as_a_crash() {
        let result = ProcessResult {
            stdout: String::new(),
            stderr: String::new(),
            signal: None,
            exit_code: Some(-1073741819),
            timed_out: false,
        };

        assert!(result.crashed());
        assert_eq!(
            result.signal_name(),
            Some("SIGSEGV (access violation / STATUS_ACCESS_VIOLATION)")
        );
    }
}
