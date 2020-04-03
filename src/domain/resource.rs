use std::path::{Path, PathBuf};

use crate::domain::{Bundle, BundlePath};

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
