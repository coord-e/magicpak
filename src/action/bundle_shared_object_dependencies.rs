use crate::base::Result;
use crate::domain::{Bundle, Executable};

use log::info;

pub fn bundle_shared_object_dependencies(bundle: &mut Bundle, exe: &Executable) -> Result<()> {
    info!(
        "action: bundle shared object dependencies of {}",
        exe.path().display()
    );

    bundle.add(exe.interpreter());
    bundle.add(exe.dynamic_libraries()?);
    Ok(())
}
