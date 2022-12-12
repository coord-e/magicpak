use crate::base::{Error, Result};
use crate::domain::{Bundle, Executable};

fn include_glob_impl(
    bundle: &mut Bundle,
    pattern: &str,
    cc: &str,
    noload_resolver: bool,
) -> Result<()> {
    tracing::info!(%pattern, "action: include using glob");

    let cc_path = which::which(cc).map_err(|e| Error::ExecutableLocateFailed(cc.to_owned(), e))?;

    for entry in glob::glob(pattern)? {
        match entry {
            Ok(path) => {
                if let Ok(obj) = Executable::load(&path) {
                    bundle.add(obj.interpreter());
                    if noload_resolver {
                        bundle.add(obj.dynamic_libraries_noload(&cc_path)?);
                    } else {
                        bundle.add(obj.dynamic_libraries(&cc_path)?);
                    }
                }
                bundle.add(path);
            }
            Err(e) => tracing::warn!(error = %e, "action: include_glob: Ignoring glob match"),
        }
    }

    Ok(())
}

pub fn include_glob(bundle: &mut Bundle, pattern: &str, cc: &str) -> Result<()> {
    include_glob_impl(bundle, pattern, cc, false)
}

pub fn include_glob_noload(bundle: &mut Bundle, pattern: &str, cc: &str) -> Result<()> {
    include_glob_impl(bundle, pattern, cc, true)
}
