use crate::base::{Error, Result};
use crate::domain::{Bundle, Executable};

fn bundle_shared_object_dependencies_impl(
    bundle: &mut Bundle,
    exe: &Executable,
    cc: &str,
    noload_resolver: bool,
) -> Result<()> {
    tracing::info!(
        exe = %exe.path().display(),
        "action: bundle shared object dependencies",
    );

    let cc_path = which::which(cc).map_err(|e| Error::ExecutableLocateFailed(cc.to_owned(), e))?;

    bundle.add(exe.interpreter());
    if noload_resolver {
        bundle.add(exe.dynamic_libraries_noload(cc_path)?);
    } else {
        bundle.add(exe.dynamic_libraries(cc_path)?);
    }

    Ok(())
}

pub fn bundle_shared_object_dependencies(
    bundle: &mut Bundle,
    exe: &Executable,
    cc: &str,
) -> Result<()> {
    bundle_shared_object_dependencies_impl(bundle, exe, cc, false)
}

pub fn bundle_shared_object_dependencies_noload(
    bundle: &mut Bundle,
    exe: &Executable,
    cc: &str,
) -> Result<()> {
    bundle_shared_object_dependencies_impl(bundle, exe, cc, true)
}
