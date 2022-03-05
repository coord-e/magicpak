use crate::base::Result;
use crate::domain::Bundle;

pub fn exclude_glob(bundle: &mut Bundle, pattern: &str) -> Result<()> {
    tracing::info!(%pattern, "action: exclude using glob");

    let pattern = glob::Pattern::new(pattern)?;
    bundle.filter(|path| {
        let str_path = path.to_str_lossy();
        let pseudo_path = format!("/{}", str_path);
        tracing::debug!(
            %pattern,
            %pseudo_path,
            "action: exclude_glob: matching with pseudo path",
        );
        !pattern.matches(&pseudo_path)
    });
    Ok(())
}
