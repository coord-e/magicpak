use crate::base::{Error, Result};
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

    let executable_path = exe.path().canonicalize()?;

    match install_path {
        Some(mut path) => {
            if path.ends_with('/') {
                // unwrap is ok because `executable_path` here is canonicalized
                let file_name = executable_path.file_name().unwrap();
                let file_name_str = file_name
                    .to_str()
                    .ok_or_else(|| Error::PathEncoding(file_name.to_os_string()))?;
                path.push_str(file_name_str);
                info!(
                    "action: bundle_executable: completing full path to {}",
                    path
                );
            }
            bundle.add_file_from(BundlePath::projection(&path), executable_path)
        }
        None => bundle.add(executable_path),
    }

    Ok(())
}
