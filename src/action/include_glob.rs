use crate::base::Result;
use crate::domain::Bundle;

pub fn include_glob(bundle: &mut Bundle, pattern: &str) -> Result<()> {
    for entry in glob::glob(pattern)? {
        match entry {
            Ok(path) => bundle.add(path.canonicalize()?),
            Err(_) => {} // TODO: log
        }
    }

    Ok(())
}
