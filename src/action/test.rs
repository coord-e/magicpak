use std::process::Command;

use crate::base::log::CommandLogExt;
use crate::base::{Error, Result};
use crate::domain::jail::CommandJailExt;
use crate::domain::Bundle;

use log::info;

pub fn test<S>(
    bundle: &Bundle,
    command: &str,
    command_stdout: Option<S>,
    busybox: &str,
) -> Result<()>
where
    S: AsRef<str>,
{
    info!("action: test the bundle with command '{}'", command);

    let busybox_path = which::which(busybox)?;

    let jail = bundle.create_jail()?;
    jail.install_busybox(busybox_path)?;

    let output = Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .in_jail(&jail)
        .output_with_log()?;

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
