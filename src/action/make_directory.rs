use crate::domain::{Bundle, BundlePath};

pub fn make_directory(bundle: &mut Bundle, path: &str) {
    tracing::info!(%path, "action: make directory");
    bundle.mkdir(BundlePath::projection(&path));
}
