use crate::domain::{Bundle, BundlePath};

pub fn make_directory(bundle: &mut Bundle, path: &str) {
    bundle.mkdir(BundlePath::projection(&path));
}
