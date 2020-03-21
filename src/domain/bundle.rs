use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::path::{Path, PathBuf};

use crate::base::Result;
use crate::domain::{BundlePath, BundlePathBuf, Jail, Resource};

use log::{debug, info};

#[derive(Clone)]
enum Source {
    NewDirectory,
    NewFile(Vec<u8>),
    CopyFrom(PathBuf),
}

#[derive(Default, Clone)]
pub struct Bundle {
    entries: HashMap<BundlePathBuf, Source>,
}

impl Bundle {
    pub fn new() -> Bundle {
        Bundle {
            entries: HashMap::new(),
        }
    }

    pub fn mkdir<P>(&mut self, path: P)
    where
        P: AsRef<BundlePath>,
    {
        debug!("bundle: mkdir {}", path.as_ref().display());
        self.entries
            .insert(path.as_ref().to_owned(), Source::NewDirectory);
    }

    pub fn add_file<P>(&mut self, path: P, content: Vec<u8>)
    where
        P: AsRef<BundlePath>,
    {
        debug!(
            "bundle: add_file {} (content omitted)",
            path.as_ref().display()
        );
        self.entries
            .insert(path.as_ref().to_owned(), Source::NewFile(content));
    }

    pub fn add_file_from<P, Q>(&mut self, path: P, from: Q)
    where
        P: AsRef<BundlePath>,
        Q: AsRef<Path>,
    {
        debug_assert!(from.as_ref().is_absolute());
        debug!(
            "bundle: copy from: {} to: {}",
            from.as_ref().display(),
            path.as_ref().display()
        );

        self.entries.insert(
            path.as_ref().to_owned(),
            Source::CopyFrom(from.as_ref().to_owned()),
        );
    }

    pub fn add<R>(&mut self, resource: R)
    where
        R: Resource,
    {
        resource.bundle_to(self);
    }

    pub fn filter<P>(&mut self, mut predicate: P)
    where
        P: FnMut(&BundlePathBuf) -> bool,
    {
        let entries = std::mem::take(&mut self.entries);
        let updated = entries.into_iter().filter(|(k, _)| predicate(k)).collect();
        std::mem::replace(&mut self.entries, updated);
    }

    pub fn emit<P>(&self, dest: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        for (bpath, source) in self.entries.iter() {
            match source {
                Source::NewDirectory => {
                    let path = bpath.reify(&dest);
                    info!("emit: mkdir {}", path.display());
                    fs::create_dir_all(path)?
                }
                Source::NewFile(blob) => {
                    let path = bpath.reify(&dest);
                    info!("emit: write {} (content omitted)", path.display());
                    fs::write(path, blob)?
                }
                Source::CopyFrom(src_path) => {
                    sync_copy(src_path, bpath, dest.as_ref())?;
                }
            }
        }
        Ok(())
    }

    pub fn create_jail(&self) -> Result<Jail> {
        let jail = Jail::new()?;
        debug!("bundle: created jail {}", jail.path().display());

        self.emit(&jail)?;
        Ok(jail)
    }
}

// We don't use `fs::copy` directly because we want to respect symlinks.
// Also `fs::canonicalize` is not used because we don't want to skip intermediate links.
fn sync_copy(from: &Path, to: &BundlePath, dest: &Path) -> Result<()> {
    use std::os::unix;
    debug_assert!(from.is_absolute());
    debug_assert!(dest.is_absolute());

    let target = to.reify(dest);
    debug_assert!(target.is_absolute());

    match target.parent() {
        Some(parent) if !parent.exists() => fs::create_dir_all(parent)?,
        _ => (),
    }

    if fs::symlink_metadata(from)?.file_type().is_symlink() {
        let link_dest = from.read_link()?;
        let link_dest_absolute = if link_dest.is_relative() {
            // unwrap is ok because `from` here is an absolute path to a symbolic link
            from.parent().unwrap().join(link_dest)
        } else {
            link_dest
        };
        info!(
            "emit: link {} => {}",
            link_dest_absolute.display(),
            target.display()
        );
        unix::fs::symlink(&link_dest_absolute, target)?;
        sync_copy(
            &link_dest_absolute,
            BundlePath::projection(&link_dest_absolute),
            dest,
        )
    } else {
        info!("emit: copy {} => {}", from.display(), target.display());
        fs::copy(from, target)?;
        Ok(())
    }
}
