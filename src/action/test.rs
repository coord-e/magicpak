use std::io::Write;
use std::process::{Command, Stdio};

use crate::base::log::{ChildLogExt, CommandLogExt};
use crate::base::{Error, Result};
use crate::domain::jail::CommandJailExt;
use crate::domain::Bundle;

use log::info;

pub fn test<S, T>(
    bundle: &Bundle,
    command: &str,
    command_stdin: Option<S>,
    command_stdout: Option<T>,
    busybox: &str,
) -> Result<()>
where
    S: AsRef<str>,
    T: AsRef<str>,
{
    info!("action: test the bundle with command '{}'", command);

    let busybox_path = which::which(busybox)?;

    let jail = bundle.create_jail()?;
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
    info!("action: test command succeeded with {}", output.status);

    if let Some(content) = command_stdout {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout != content.as_ref() {
            return Err(Error::TestFailed(command.to_owned()));
        }
    }

    info!("action: test succeeded");
    Ok(())
}
