use crate::base::Result;
use crate::domain::Bundle;

use log::{info, warn};

pub fn include_glob(bundle: &mut Bundle, pattern: &str) -> Result<()> {
    info!("action: include using glob {}", pattern);

    for entry in glob::glob(pattern)? {
        match entry {
            Ok(path) => bundle.add(path.canonicalize()?),
            Err(e) => warn!("action: include_glob: Ignoring glob match: {}", e),
        }
    }

    Ok(())
}
