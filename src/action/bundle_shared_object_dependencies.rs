use crate::domain::{Bundle, Executable};
use crate::error::Result;

pub fn bundle_shared_object_dependencies(bundle: &mut Bundle, exe: &Executable) -> Result<()> {
    bundle.add(exe.interpreter());
    bundle.add(exe.dynamic_libraries()?);
    Ok(())
}
