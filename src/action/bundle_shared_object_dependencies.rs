use crate::base::Result;
use crate::domain::{Bundle, Executable};

use log::info;

pub fn bundle_shared_object_dependencies(
    bundle: &mut Bundle,
    exe: &Executable,
    cc: &str,
) -> Result<()> {
    info!(
        "action: bundle shared object dependencies of {}",
        exe.path().display()
    );

    let cc_path = which::which(cc)?;

    bundle.add(exe.interpreter());
    bundle.add(exe.dynamic_libraries(cc_path)?);
    Ok(())
}
