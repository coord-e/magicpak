use crate::domain::Bundle;
use crate::error::Result;

pub fn exclude_glob(bundle: &mut Bundle, pattern: &str) -> Result<()> {
    let pattern = glob::Pattern::new(pattern)?;
    bundle.filter(|path| {
        let str_path = path.to_str_lossy();
        let pseudo_path = format!("/{}", str_path);
        !pattern.matches(&pseudo_path)
    });
    Ok(())
}
