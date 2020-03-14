use crate::domain::{Bundle, BundlePath};

pub fn make_directory(bundle: &mut Bundle, path: String) {
    bundle.mkdir(BundlePath::projection(&path));
}
