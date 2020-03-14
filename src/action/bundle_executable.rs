use crate::domain::{Bundle, Executable};
use crate::error::Result;

pub fn bundle_executable(bundle: &mut Bundle, exe: &Executable) -> Result<()> {
    // TODO: check existence etc..
    bundle.add(exe.path());
    Ok(())
}
