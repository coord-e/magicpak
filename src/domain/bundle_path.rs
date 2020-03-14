use std::borrow::Borrow;
use std::ffi::{OsStr, OsString};
use std::ops::Deref;
use std::path::{Path, PathBuf};

pub struct BundlePath {
    inner: OsStr,
}

impl BundlePath {
    pub fn new<S>(s: &S) -> &BundlePath
    where
        S: AsRef<OsStr> + ?Sized,
    {
        unsafe { &*(s.as_ref() as *const OsStr as *const BundlePath) }
    }

    pub fn projection<'a, P>(p: &'a P) -> &'a BundlePath
    where
        P: AsRef<Path> + 'a,
    {
        let path = p.as_ref().strip_prefix("/").unwrap_or(p.as_ref());
        BundlePath::new(path)
    }

    pub fn to_path_buf(&self) -> BundlePathBuf {
        BundlePathBuf {
            inner: self.inner.to_os_string(),
        }
    }

    pub fn reify<P>(&self, dist: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        dist.as_ref().join(&self.inner)
    }
}

impl ToOwned for BundlePath {
    type Owned = BundlePathBuf;

    fn to_owned(&self) -> BundlePathBuf {
        self.to_path_buf()
    }
}

impl AsRef<BundlePath> for BundlePath {
    fn as_ref(&self) -> &BundlePath {
        self
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct BundlePathBuf {
    inner: OsString,
}

impl Deref for BundlePathBuf {
    type Target = BundlePath;

    fn deref(&self) -> &BundlePath {
        BundlePath::new(&self.inner)
    }
}

impl AsRef<BundlePath> for BundlePathBuf {
    fn as_ref(&self) -> &BundlePath {
        self
    }
}

impl Borrow<BundlePath> for BundlePathBuf {
    fn borrow(&self) -> &BundlePath {
        self.deref()
    }
}
