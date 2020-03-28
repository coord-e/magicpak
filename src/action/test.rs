use std::process::Command;

use crate::base::log::CommandLogExt;
use crate::base::{Error, Result};
use crate::domain::jail::CommandJailExt;
use crate::domain::Bundle;

use log::info;

pub fn test(bundle: &Bundle, command: &str, busybox: &str) -> Result<()> {
    info!("action: test the bundle with command '{}'", command);

    let busybox_path = which::which(busybox)?;

    let jail = bundle.create_jail()?;
    jail.install_busybox(busybox_path)?;

    let output = Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .in_jail(&jail)
        .output_with_log()?;

    if output.status.success() {
        info!("action: test succeeded with {}", output.status);
        Ok(())
    } else {
        Err(Error::TestFailed(command.to_owned()))
    }
}
