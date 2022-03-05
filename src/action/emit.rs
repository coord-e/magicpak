use std::fs;
use std::path::Path;

use crate::base::{Error, Result};
use crate::domain::Bundle;

pub fn emit<P>(bundle: &mut Bundle, path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let dest = path.as_ref();
    tracing::info!(dest = %dest.display(), "action: emit");

    if dest.exists() {
        if !dest.is_dir() {
            return Err(Error::InvalidDestination(dest.to_owned()));
        }
        if dest.read_dir()?.next().is_some() {
            return Err(Error::NonEmptyDestionation(dest.to_owned()));
        }
    } else {
        tracing::info!(
            dest = %dest.display(),
            "action: emit: creating destination dir as it does not exist",
        );
        fs::create_dir(dest)?;
    };
    bundle.emit(dest.canonicalize()?)
}
