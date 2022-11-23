use std::ffi::OsStr;

use crate::base::{Error, Result};
use crate::domain::Executable;

pub fn compress_exexcutable<I, S>(exe: &mut Executable, upx: &str, upx_opts: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    tracing::info!(exe = %exe.path().display(), "action: compress executable");

    let upx_path =
        which::which(upx).map_err(|e| Error::ExecutableLocateFailed(upx.to_owned(), e))?;

    let compressed = exe.compressed(upx_path, upx_opts)?;
    tracing::debug!(
        path = %compressed.path().display(),
        "action: compress_exexcutable: compressed executable",
    );

    *exe = compressed;
    Ok(())
}
