use std::fs;
use std::path::Path;

use crate::base::{Error, Result};
use crate::domain::Bundle;

pub fn emit<P>(bundle: &mut Bundle, path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let dest = path.as_ref();
    if dest.exists() {
        if !dest.is_dir() {
            return Err(Error::InvalidDestination(dest.to_owned()));
        }
        if dest.read_dir()?.next().is_some() {
            return Err(Error::NonEmptyDestionation(dest.to_owned()));
        }
    } else {
        fs::create_dir(dest)?;
    };
    bundle.emit(dest.canonicalize()?)
}
