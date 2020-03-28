use std::ffi::OsStr;
use std::mem;

use crate::base::Result;
use crate::domain::Executable;

use log::{debug, info};

pub fn compress_exexcutable<I, S>(exe: &mut Executable, upx_path: &str, upx_opts: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    info!("action: compress {}", exe.path().display());

    let compressed = exe.compressed(upx_path, upx_opts)?;
    debug!(
        "action: compress_exexcutable: compressed executable {}",
        compressed.path().display()
    );

    mem::replace(exe, compressed);
    Ok(())
}
