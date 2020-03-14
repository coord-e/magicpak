use std::path::Path;

use crate::domain::Bundle;
use crate::error::Result;

pub fn emit<P>(bundle: &mut Bundle, path: P) -> Result<()>
where
    P: AsRef<Path> + Clone,
{
    // TODO: Check existence etc...
    bundle.emit(path)
}
