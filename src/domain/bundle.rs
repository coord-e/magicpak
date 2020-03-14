use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{BundlePath, BundlePathBuf};
use crate::error::Result;

enum Source {
    NewDirectory,
    NewFile(Vec<u8>),
    CopyFrom(PathBuf),
}

#[derive(Default)]
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
        self.entries
            .insert(path.as_ref().to_owned(), Source::NewDirectory);
    }

    pub fn add_file<P>(&mut self, path: P, content: Vec<u8>)
    where
        P: AsRef<BundlePath>,
    {
        self.entries
            .insert(path.as_ref().to_owned(), Source::NewFile(content));
    }

    pub fn add_file_from<P, Q>(&mut self, path: P, from: Q)
    where
        P: AsRef<BundlePath>,
        Q: AsRef<Path>,
    {
        debug_assert!(from.as_ref().is_absolute());

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

    pub fn filter<P>(&mut self, predicate: P)
    where
        P: FnMut(&BundlePathBuf) -> bool,
    {
        let mut predicate = predicate;
        let entries = std::mem::replace(&mut self.entries, HashMap::default());
        let updated = entries.into_iter().filter(|(k, _)| predicate(k)).collect();
        std::mem::replace(&mut self.entries, updated);
    }

    pub fn emit<P>(&mut self, dest: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        for (bpath, source) in self.entries.iter() {
            match source {
                Source::NewDirectory => fs::create_dir_all(bpath.reify(&dest))?,
                Source::NewFile(blob) => fs::write(bpath.reify(&dest), blob)?,
                Source::CopyFrom(src_path) => {
                    sync_copy(src_path, bpath, dest.as_ref())?;
                }
            }
        }
        Ok(())
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
        unix::fs::symlink(&link_dest_absolute, target)?;
        sync_copy(
            &link_dest_absolute,
            BundlePath::projection(&link_dest_absolute),
            dest,
        )
    } else {
        fs::copy(from, target)?;
        Ok(())
    }
}

pub trait Resource {
    fn bundle_to(self, bundle: &mut Bundle);
}

impl Resource for &Path {
    fn bundle_to(self, bundle: &mut Bundle) {
        bundle.add_file_from(BundlePath::projection(&self), self);
    }
}

impl Resource for PathBuf {
    fn bundle_to(self, bundle: &mut Bundle) {
        self.as_path().bundle_to(bundle);
    }
}

impl Resource for &PathBuf {
    fn bundle_to(self, bundle: &mut Bundle) {
        self.as_path().bundle_to(bundle);
    }
}

impl<R> Resource for Option<R>
where
    R: Resource,
{
    fn bundle_to(self, bundle: &mut Bundle) {
        if let Some(x) = self {
            x.bundle_to(bundle);
        }
    }
}

impl<R> Resource for Vec<R>
where
    R: Resource,
{
    fn bundle_to(self, bundle: &mut Bundle) {
        for x in self {
            x.bundle_to(bundle);
        }
    }
}
