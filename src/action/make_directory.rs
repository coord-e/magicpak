use crate::domain::{Bundle, BundlePath};

use log::info;

pub fn make_directory(bundle: &mut Bundle, path: &str) {
    info!("action: make directory {}", path);
    bundle.mkdir(BundlePath::projection(&path));
}
