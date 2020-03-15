use crate::base::Result;
use crate::domain::Bundle;

use log::{debug, info};

pub fn exclude_glob(bundle: &mut Bundle, pattern: &str) -> Result<()> {
    info!("action: exclude using glob {}", pattern);

    let pattern = glob::Pattern::new(pattern)?;
    bundle.filter(|path| {
        let str_path = path.to_str_lossy();
        let pseudo_path = format!("/{}", str_path);
        debug!(
            "action: exclude_glob: matching {} with pseudo path {}",
            pattern, pseudo_path
        );
        !pattern.matches(&pseudo_path)
    });
    Ok(())
}
