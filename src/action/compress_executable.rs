use std::mem;

use crate::base::Result;
use crate::domain::Executable;

use log::{debug, info};

pub fn compress_exexcutable<I, S>(exe: &mut Executable, upx_path: &str, upx_opts: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    info!("action: compress {}", exe.path().display());

    let upx_opts_vec: Vec<Vec<_>> = upx_opts
        .into_iter()
        .map(|opt| {
            let splitted = shell_words::split(opt.as_ref())?;
            debug!(
                "action: compress_exexcutable: splited upx options {} into {:?}",
                opt.as_ref(),
                splitted
            );
            Ok(splitted)
        })
        .collect::<Result<_>>()?;

    let compressed = exe.compressed(upx_path, upx_opts_vec.into_iter().flatten())?;
    debug!(
        "action: compress_exexcutable: compressed executable {}",
        compressed.path().display()
    );

    mem::replace(exe, compressed);
    Ok(())
}
