use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

use crate::base::log::CommandLogExt;
use crate::base::{Error, Result};

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

        tracing::info!(
            from_path = %busybox_path.as_ref().display(),
            jail_path = %busybox_jail_path.display(),
            "jail: copying busybox",
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
                tracing::debug!(path = %jail_path.display(), "jail: chroot");
                nix::unistd::chroot(&jail_path)?;
                tracing::debug!("jail: chdir to /");
                env::set_current_dir("/")
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::path::PathBuf;
    use std::process::Command;

    fn locate_busybox() -> std::result::Result<PathBuf, Box<dyn std::error::Error>> {
        let path = if let Ok(path) = std::env::var("MAGICPAK_TEST_STATIC_BUSYBOX") {
            path.into()
        } else {
            which::which("busybox")?
        };
        Ok(path)
    }

    #[test]
    fn test_install_busybox() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let jail = Jail::new()?;
        jail.install_busybox(locate_busybox()?)?;

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
        jail.install_busybox(locate_busybox()?)?;

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
