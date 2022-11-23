use std::io::Write;
use std::process::{Command, Stdio};

use crate::base::log::{ChildLogExt, CommandLogExt};
use crate::base::{Error, Result};
use crate::domain::jail::CommandJailExt;
use crate::domain::{Bundle, Executable};

pub fn test<S, T, U>(
    bundle: &Bundle,
    exe: &Executable,
    command: Option<S>,
    command_stdin: Option<T>,
    command_stdout: Option<U>,
    busybox: &str,
) -> Result<()>
where
    S: AsRef<str>,
    T: AsRef<str>,
    U: AsRef<str>,
{
    let command = command
        .as_ref()
        .map(AsRef::as_ref)
        .unwrap_or_else(|| exe.name());

    tracing::info!(%command, "action: test the bundle");

    let busybox_path =
        which::which(busybox).map_err(|e| Error::ExecutableLocateFailed(busybox.to_owned(), e))?;

    let mut test_bundle = bundle.clone();
    test_bundle.add_pseudo_proc(exe);

    let jail = test_bundle.create_jail()?;
    jail.install_busybox(busybox_path)?;

    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .in_jail(&jail)
        .spawn_with_log()?;

    if let Some(content) = command_stdin {
        // unwrap is ok here because stdin is surely piped
        write!(child.stdin.as_mut().unwrap(), "{}", content.as_ref())?;
    }

    let output = child.wait_output_with_log()?;

    if !output.status.success() {
        return Err(Error::TestFailed(command.to_owned()));
    }
    tracing::info!(status = %output.status, "action: test command succeeded");

    if let Some(content) = command_stdout {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout != content.as_ref() {
            return Err(Error::TestStdoutMismatch {
                expected: content.as_ref().to_string(),
                got: stdout,
            });
        }
    }

    tracing::info!("action: test succeeded");
    Ok(())
}
