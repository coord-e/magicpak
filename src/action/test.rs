use std::fs;
use std::process::Command;

use crate::base::command_ext::CommandExt;
use crate::base::{Error, Result};
use crate::domain::Bundle;

use log::{debug, info};
use tempfile::TempDir;

pub fn test(bundle: &Bundle, command: &str, busybox_path: &str) -> Result<()> {
    info!("action: test the bundle with command '{}'", command);

    let tmp = TempDir::new()?;
    bundle.emit(tmp.path())?;

    let busybox_path_host = which::which(busybox_path)?;

    let bindir = tmp.path().join("bin/");
    let busybox_path = bindir.join("busybox");

    info!(
        "action: test: copying busybox {} => {}",
        busybox_path_host.display(),
        busybox_path.display()
    );
    fs::copy(&busybox_path_host, &busybox_path)?;

    let output = Command::new(busybox_path)
        .arg("--install")
        .arg(bindir)
        .output_with_log()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::BusyBoxInstall(stderr));
    }

    // fork --> chroot --> chdir --> execv
    //      --> wait for child to exit
    use nix::sys::wait::WaitStatus;
    use nix::unistd::ForkResult;
    match nix::unistd::fork()? {
        ForkResult::Parent { child, .. } => {
            debug!("action: test: forked {}", child);
            match nix::sys::wait::waitpid(child, None)? {
                WaitStatus::Exited(_, 0) => Ok(()),
                _ => Err(Error::TestFailed(command.to_owned())),
            }
        }
        ForkResult::Child => {
            debug!("action: test: chroot to {}", tmp.path().display());
            nix::unistd::chroot(tmp.path())?;
            debug!("action: test: chdir to /");
            nix::unistd::chdir("/")?;
            debug!("action: test: executing '{}' with /bin/sh", command);

            use std::ffi::CString;
            // unwrap is ok here because they don't contain interior NULL byte
            let binsh = CString::new(b"/bin/sh" as &[u8]).unwrap();
            let c = CString::new(b"-c" as &[u8]).unwrap();
            let cmd = CString::new(command).unwrap();
            match nix::unistd::execv(&binsh, &[&binsh, &c, &cmd])? {}
        }
    }
}
