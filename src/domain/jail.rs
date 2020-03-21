use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

use crate::base::error;
use crate::base::log::CommandLogExt;
use crate::base::{Error, Result};

use log::{debug, info};
use tempfile::TempDir;

pub struct Jail {
    dir: TempDir,
}

impl Jail {
    pub fn new() -> Result<Self> {
        let dir = TempDir::new()?;
        Ok(Jail { dir })
    }

    pub fn install_busybox<P>(&self, busybox_path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let bindir = self.dir.path().join("bin/");
        let busybox_jail_path = bindir.join("busybox");

        info!(
            "jail: copying busybox {} => {}",
            busybox_path.as_ref().display(),
            busybox_jail_path.display()
        );
        fs::copy(&busybox_path, &busybox_jail_path)?;

        let output = Command::new(busybox_jail_path)
            .arg("--install")
            .arg(bindir)
            .output_with_log()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::BusyBoxInstall(stderr));
        }

        Ok(())
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }
}

impl AsRef<Path> for Jail {
    fn as_ref(&self) -> &Path {
        self.path()
    }
}

pub trait CommandJailExt {
    fn in_jail(&mut self, jail: &Jail) -> &mut Self;
}

impl CommandJailExt for Command {
    fn in_jail(&mut self, jail: &Jail) -> &mut Command {
        let jail_path = jail.path().to_owned();
        unsafe {
            self.pre_exec(move || {
                debug!("jail: chroot to {}", &jail_path.display());
                nix::unistd::chroot(&jail_path).map_err(error::nix_to_io)?;
                debug!("jail: chdir to /");
                env::set_current_dir("/")
            })
        }
    }
}
