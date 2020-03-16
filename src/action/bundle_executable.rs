use crate::base::Result;
use crate::domain::{Bundle, BundlePath, Executable};

use log::info;

pub fn bundle_executable(
    bundle: &mut Bundle,
    exe: &Executable,
    install_path: Option<String>,
) -> Result<()> {
    info!(
        "action: bundle executable {} as {:?}",
        exe.path().display(),
        install_path
    );

    match install_path {
        Some(mut path) => {
            if path.ends_with('/') {
                path.push_str(exe.name());
                info!(
                    "action: bundle_executable: completing full path to {}",
                    path
                );
            }
            bundle.add_file_from(BundlePath::projection(&path), exe.path())
        }
        None => bundle.add(exe.path()),
    }

    Ok(())
}
