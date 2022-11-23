use crate::base::Result;
use crate::domain::{Bundle, Executable};

pub fn include_glob(bundle: &mut Bundle, pattern: &str, cc: &str) -> Result<()> {
    tracing::info!(%pattern, "action: include using glob");

    let cc_path = which::which(cc)?;

    for entry in glob::glob(pattern)? {
        match entry {
            Ok(path) => {
                if let Ok(obj) = Executable::load(&path) {
                    bundle.add(obj.interpreter());
                    bundle.add(obj.dynamic_libraries(&cc_path)?);
                }
                bundle.add(path);
            }
            Err(e) => tracing::warn!(error = %e, "action: include_glob: Ignoring glob match"),
        }
    }

    Ok(())
}
