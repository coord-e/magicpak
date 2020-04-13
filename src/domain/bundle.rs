use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::path::{Path, PathBuf};

use crate::base::Result;
use crate::domain::{BundlePath, BundlePathBuf, Executable, Jail, Resource};

use log::{debug, info, warn};

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
                    create_parent_dir(&path)?;
                    fs::write(path, blob)?
                }
                Source::CopyFrom(src_path) => {
                    sync_copy(src_path, bpath, dest.as_ref())?;
                }
            }
        }
        Ok(())
    }

    pub fn add_pseudo_proc(&mut self, exe: &Executable) {
        // TODO: using symlink would be better
        self.add_file_from(BundlePath::new("proc/self/exe"), exe.path());
    }

    pub fn create_jail(&self) -> Result<Jail> {
        let jail = Jail::new()?;
        debug!("bundle: created jail {}", jail.path().display());

        self.emit(&jail)?;
        Ok(jail)
    }
}

fn create_parent_dir<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    match path.as_ref().parent() {
        Some(parent) if !parent.exists() => fs::create_dir_all(parent).map_err(Into::into),
        _ => Ok(()),
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

    create_parent_dir(&target)?;

    if !from.exists() {
        warn!(
            "emit: copy source {} does not exist. skipping.",
            from.display()
        );
        return Ok(());
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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use std::os::unix;

    #[test]
    fn test_sync_copy() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dest = assert_fs::TempDir::new()?;
        let src = assert_fs::NamedTempFile::new("x.txt")?;
        src.write_str("hello")?;
        let bundle_path = BundlePath::new("a/b/c.txt");
        sync_copy(src.path(), bundle_path, dest.path())?;
        dest.child("a/b/c.txt").assert("hello");
        Ok(())
    }

    #[test]
    fn test_sync_copy_nonexistent() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dest = assert_fs::TempDir::new()?;
        let src = assert_fs::TempDir::new()?.child("nonexistent.txt");
        let bundle_path = BundlePath::new("a/b/c.txt");
        sync_copy(src.path(), bundle_path, dest.path())?;
        dest.child("a/b/c.txt").assert(predicate::path::missing());
        Ok(())
    }

    #[test]
    fn test_sync_copy_link() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dest = assert_fs::TempDir::new()?;
        let src_dir = assert_fs::TempDir::new()?;
        let src = src_dir.child("x.txt");
        src.touch()?;
        src.write_str("hello")?;
        let link = src_dir.child("y.txt");
        unix::fs::symlink(src.path(), link.path())?;

        let bundle_path = BundlePath::new("a/b/c.txt");
        sync_copy(link.path(), bundle_path, dest.path())?;

        assert!(fs::symlink_metadata(dest.child("a/b/c.txt").path())?
            .file_type()
            .is_symlink());
        // dest.child("a/b/c.txt")
        //     .assert(predicate::path::is_symlink());
        dest.child("a/b/c.txt").assert("hello");

        let bundle_src_path = dest.child(src.path().strip_prefix("/").unwrap());
        bundle_src_path.assert("hello");
        bundle_src_path.assert(predicate::path::is_file());
        Ok(())
    }

    #[test]
    fn test_mkdir() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dest = assert_fs::TempDir::new()?;
        let mut bundle = Bundle::new();
        bundle.mkdir(BundlePath::new("dir/dirdir"));
        bundle.emit(dest.path())?;

        dest.child("dir/dirdir").assert(predicate::path::is_dir());
        Ok(())
    }

    #[test]
    fn test_add_file() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dest = assert_fs::TempDir::new()?;
        let mut bundle = Bundle::new();
        bundle.add_file(BundlePath::new("dir/text.txt"), b"hello".to_vec());
        bundle.emit(dest.path())?;

        dest.child("dir/text.txt").assert("hello");
        Ok(())
    }

    #[test]
    fn test_add_file_from() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dest = assert_fs::TempDir::new()?;
        let src = assert_fs::NamedTempFile::new("x.txt")?;
        src.write_str("hello")?;

        let mut bundle = Bundle::new();
        bundle.add_file_from(BundlePath::new("dir/text.txt"), src.path());
        bundle.emit(dest.path())?;

        dest.child("dir/text.txt").assert("hello");
        Ok(())
    }

    #[test]
    fn test_filter() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dest = assert_fs::TempDir::new()?;

        let mut bundle = Bundle::new();
        bundle.add_file(BundlePath::new("a.txt"), b"hello1".to_vec());
        bundle.add_file(BundlePath::new("b.txt"), b"hello2".to_vec());
        bundle.filter(|path| path.to_str_lossy().contains("a"));
        bundle.emit(dest.path())?;

        dest.child("a.txt").assert("hello1");
        dest.child("b.txt").assert(predicate::path::missing());
        Ok(())
    }
}
