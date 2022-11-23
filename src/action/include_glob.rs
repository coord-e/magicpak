use crate::base::Result;
use crate::domain::Bundle;

pub fn include_glob(bundle: &mut Bundle, pattern: &str) -> Result<()> {
    tracing::info!(%pattern, "action: include using glob");

    for entry in glob::glob(pattern)? {
        match entry {
            Ok(path) => bundle.add(path),
            Err(e) => tracing::warn!(error = %e, "action: include_glob: Ignoring glob match"),
        }
    }

    Ok(())
}
