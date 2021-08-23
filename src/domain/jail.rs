use std::os::unix::fs::PermissionsExt;
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
        if !bindir.exists() {
            fs::create_dir(&bindir)?;
        }
        fs::copy(&busybox_path, &busybox_jail_path)?;
        fs::set_permissions(&busybox_jail_path, fs::Permissions::from_mode(0o755))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::prelude::*;
    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use std::io::Read;
    use std::process::Command;

    fn download_busybox(
    ) -> std::result::Result<assert_fs::NamedTempFile, Box<dyn std::error::Error>> {
        let url =
            "https://busybox.net/downloads/binaries/1.31.0-defconfig-multiarch-musl/busybox-x86_64";
        let mut bytes = Vec::new();
        reqwest::blocking::get(url)?.read_to_end(&mut bytes)?;
        let dest = assert_fs::NamedTempFile::new("busybox")?;
        dest.write_binary(&bytes)?;
        Ok(dest)
    }

    #[test]
    fn test_install_busybox() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let jail = Jail::new()?;
        jail.install_busybox(download_busybox()?.path())?;

        assert_eq!(
            true,
            predicate::path::is_file().eval(&jail.path().join("bin/busybox"))
        );
        assert_eq!(
            true,
            predicate::path::is_file().eval(&jail.path().join("bin/sh"))
        );
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_jail() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let jail = Jail::new()?;
        jail.install_busybox(download_busybox()?.path())?;

        Command::new("pwd")
            .in_jail(&jail)
            .assert()
            .success()
            .stdout("/\n");

        Command::new("ls")
            .in_jail(&jail)
            .assert()
            .success()
            .stdout("bin\n");

        Ok(())
    }
}
