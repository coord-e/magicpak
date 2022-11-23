use crate::base::{Error, Result};
use crate::domain::{Bundle, Executable};

pub fn bundle_shared_object_dependencies(
    bundle: &mut Bundle,
    exe: &Executable,
    cc: &str,
) -> Result<()> {
    tracing::info!(
        exe = %exe.path().display(),
        "action: bundle shared object dependencies",
    );

    let cc_path = which::which(cc).map_err(|e| Error::ExecutableLocateFailed(cc.to_owned(), e))?;

    bundle.add(exe.interpreter());
    bundle.add(exe.dynamic_libraries(cc_path)?);
    Ok(())
}
