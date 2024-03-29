use std::path::Path;

use crate::base::Result;
use crate::domain::{Bundle, BundlePath, Executable};

pub fn bundle_executable<S, P>(
    bundle: &mut Bundle,
    exe: &Executable,
    input_path: P,
    install_path: Option<S>,
) -> Result<()>
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    tracing::info!(
        exe = %exe.path().display(),
        install_path = ?install_path.as_ref().map(|x| x.as_ref()),
        "action: bundle executable",
    );

    match install_path {
        Some(p) => {
            let mut path = p.as_ref().to_owned();

            if path.ends_with('/') {
                path.push_str(exe.name());
                tracing::info!(
                    completed_path = %path,
                    "action: bundle_executable: completing full path",
                );
            }
            bundle.add_file_from(BundlePath::projection(&path), exe.path());
        }
        None => bundle.add_file_from(BundlePath::projection(&input_path), exe.path()),
    }

    Ok(())
}
