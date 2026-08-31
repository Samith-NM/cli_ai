use anyhow::{Context,Result};
use std::process::Stdio;
use std::os::unix::process::ExitStatusExt;
use std::io::Read;

use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

#[derive(Debug)]
pub struct ProcessResult{
    pub stdout :String,
    pub stderr:String,

    pub signal:Option<i32>,
    pub exit_code:Option<i32>,
    pub timed_out:bool,
}

impl ProcessResult{
    pub fn crashed(&self) -> bool{
        self.signal.is_some()

    }
    pub fn hung(&self) -> bool {
        self.timed_out
    }
    pub fn signal_name(&self) -> Option<&'static str> {
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

}

pub async fn run_processed(
    path:&str,
    args:&[String],
    timeout_secs: u64,

) -> Result<ProcessResult>{
    let mut child = Command::new(path)
    .args(args)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .with_context(|| format!("failed to launch `{path}`"))?;
    let mut stdout_pipe = child.stdout.take().except("stdout was piped");
    let mut stder_pipe = child.stderr.take().except("stderr was piped");

    let stdout_task = tokio::spawn(async move{
        let mut buf = String::new();
        let _ = stdout_pipe.read_to_string(&mut buf).await;
        buf
    });
    let (status,timed_out) = if timeout_secs == 0 {
        (child.wait().await.context("child process wait() failed")?,false)

    }else{
            match tokio::time::timeout(Duration::from_secs(timeout_secs),child.wait()).await{
                Ok(status) => (status.context("child process wait() failed")?,false),

                Err(_elapsed) => {
                    child.kill().await.context("failed to kill the hunged child process!")?;
                    let status = child.wait().await.context("wait() after kill failed")?;
                     (status,true)

                }
            }
    };
    let stdout = stdout_task.await.unwrap_or_default();
    let stderr = stderr_task.await.unwrap_or_default();

    Ok(ProcessResult{
        stdout,
        stderr,
        signal:if timed_out{None} else {status.signal()},
        exit_code:if timeout{None} else{status.code()},
        timed_out,
    })
}
