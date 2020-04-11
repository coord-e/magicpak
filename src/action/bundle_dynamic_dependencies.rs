use std::cell::RefCell;
use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::rc::Rc;

use crate::base::log::{log_output, CommandLogExt};
use crate::base::trace::{ChildTraceExt, CommandTraceExt, SyscallHandler};
use crate::base::{Error, Result};
use crate::domain::{Bundle, Executable};

use log::{debug, info};

pub fn bundle_dynamic_dependencies<I, S, T>(
    bundle: &mut Bundle,
    exe: &Executable,
    args: I,
    stdin: Option<T>,
) -> Result<()>
where
    I: IntoIterator<Item = S> + Debug,
    S: AsRef<OsStr>,
    T: AsRef<str>,
{
    info!(
        "action: bundle dynamically analyzed dependencies of {} with arguments {:?} and stdin {:?}",
        exe.path().display(),
        args,
        stdin.as_ref().map(AsRef::as_ref)
    );

    let mut child = Command::new(exe.path())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .traceme()
        .spawn_with_log()?;

    if let Some(content) = stdin {
        // unwrap is ok here because stdin is surely piped
        write!(child.stdin.take().unwrap(), "{}", content.as_ref())?;
    }

    let bundle_ref = Rc::new(RefCell::new(bundle));

    let output = child.trace_syscalls(SyscallHandler {
        open: |pathname, _| open_handler(&bundle_ref, "open", pathname),
        openat: |_, pathname, _| open_handler(&bundle_ref, "openat", pathname),
    })?;
    log_output("<dynamic analysis command>", &output);

    if !output.status.success() {
        return Err(Error::DynamicFailed(output.status));
    }

    Ok(())
}

fn open_handler(bundle: &Rc<RefCell<&mut Bundle>>, name: &str, pathname: OsString) {
    let path: PathBuf = pathname.into();

    debug!(
        "action: bundle_dynamic_dependencies: syscall '{}' opens {}",
        name,
        path.display()
    );

    if path.is_file() {
        info!(
            "action: bundle_dynamic_dependencies: found {}",
            path.display()
        );

        bundle.borrow_mut().add(path);
    }
}
