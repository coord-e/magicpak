use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::base::error;
use crate::base::log::CommandLogExt;
use crate::base::{Error, Result};
use crate::domain::{Bundle, Executable};

use log::{debug, info, warn};

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

    let child = unsafe {
        Command::new(exe.path())
            .args(args)
            .pre_exec(|| nix::sys::ptrace::traceme().map_err(error::nix_to_io))
    }
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn_with_log()?;
    let child_pid = nix::unistd::Pid::from_raw(child.id() as i32);

    if let Some(content) = stdin {
        // unwrap is ok here because stdin is surely piped
        write!(child.stdin.unwrap(), "{}", content.as_ref())?;
    }

    use nix::sys::signal::Signal;
    use nix::sys::wait::WaitStatus;

    match nix::sys::wait::waitpid(child_pid, None)? {
        WaitStatus::Stopped(_, Signal::SIGTRAP) => (),
        WaitStatus::Signaled(_, sig, _) | WaitStatus::Stopped(_, sig) => {
            return Err(Error::DynamicSignaled(sig))
        }
        WaitStatus::Exited(_, code) => return Err(Error::DynamicFailed(code)),
        _ => unreachable!(),
    }

    // TODO: should we handle forks?
    use nix::sys::ptrace::Options;
    nix::sys::ptrace::setoptions(
        child_pid,
        Options::PTRACE_O_TRACESYSGOOD | Options::PTRACE_O_EXITKILL,
    )?;
    nix::sys::ptrace::syscall(child_pid, None)?;

    loop {
        match nix::sys::wait::waitpid(child_pid, None)? {
            WaitStatus::Exited(_, 0) => {
                let mut stdout = String::new();
                let mut stderr = String::new();
                // unwrap is ok here because they are surely piped
                child.stdout.unwrap().read_to_string(&mut stdout)?;
                child.stderr.unwrap().read_to_string(&mut stderr)?;
                debug!("action: bundle_dynamic_dependencies: stdout {}", stdout);
                debug!("action: bundle_dynamic_dependencies: stderr {}", stderr);
                return Ok(());
            }
            WaitStatus::Stopped(pid, sig) => {
                warn!(
                    "action: bundle_dynamic_dependencies: stopped with {}, we attempt to continue",
                    sig
                );
                nix::sys::ptrace::syscall(pid, None)?;
            }
            WaitStatus::PtraceSyscall(pid) => {
                // NOTE: the end of syscall also comes to this branch with its return value in
                // `regs.rax`, but it doesn't matter because `regs.orig_rax` won't hold effective
                // value in that situation.
                let regs = getregs(pid)?;
                match regs.orig_rax as i64 {
                    libc::SYS_openat => open_handler(bundle, pid, "openat", regs.rsi)?,
                    libc::SYS_open => open_handler(bundle, pid, "open", regs.rdi)?,
                    _ => (),
                }
                nix::sys::ptrace::syscall(pid, None)?;
            }
            WaitStatus::Exited(_, code) => return Err(Error::DynamicFailed(code)),
            WaitStatus::Signaled(_, sig, _) => return Err(Error::DynamicSignaled(sig)),
            _ => unreachable!(),
        }
    }
}

#[cfg(target_env = "musl")]
fn getregs(pid: nix::unistd::Pid) -> Result<libc::user_regs_struct> {
    use nix::sys::ptrace::Request;
    use std::ffi::c_void;
    use std::{mem, ptr};

    let mut data = mem::MaybeUninit::uninit();
    unsafe {
        nix::sys::ptrace::ptrace(
            Request::PTRACE_GETREGS,
            pid,
            ptr::null_mut(),
            data.as_mut_ptr() as *mut c_void,
        )?;
        Ok(data.assume_init())
    }
}

#[cfg(not(target_env = "musl"))]
fn getregs(pid: nix::unistd::Pid) -> Result<libc::user_regs_struct> {
    nix::sys::ptrace::getregs(pid).map_err(Into::into)
}

fn open_handler(
    bundle: &mut Bundle,
    pid: nix::unistd::Pid,
    name: &str,
    regvalue: u64,
) -> Result<()> {
    let path: PathBuf = read_string_at(pid, regvalue)?.into();
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

        bundle.add(path);
    }

    Ok(())
}

fn read_string_at(pid: nix::unistd::Pid, mut addr: u64) -> Result<OsString> {
    use std::ffi::c_void;

    let mut result = Vec::new();
    loop {
        let word = nix::sys::ptrace::read(pid, addr as *mut c_void)? as u32;
        let bytes: [u8; 4] = unsafe { std::mem::transmute(word) };
        for byte in bytes.iter() {
            if *byte == 0 {
                return Ok(OsString::from_vec(result));
            }
            result.push(*byte);
        }
        addr += 4;
    }
}
