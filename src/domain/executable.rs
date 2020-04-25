use std::ffi::OsStr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, io};

use crate::base::log::CommandLogExt;
use crate::base::{Error, Result};

use goblin::elf::dynamic::{Dyn, DT_RPATH, DT_RUNPATH};
use goblin::elf::Elf;
use goblin::strtab::Strtab;
use log::{debug, info, warn};
use tempfile::{NamedTempFile, TempPath};

mod resolver;
mod search_paths;
use search_paths::SearchPaths;

#[derive(Debug)]
enum ExecutableLocation {
    Fixed(PathBuf),
    Temporary(TempPath),
}

impl AsRef<Path> for ExecutableLocation {
    fn as_ref(&self) -> &Path {
        match self {
            ExecutableLocation::Fixed(path) => path.as_ref(),
            ExecutableLocation::Temporary(temp_path) => temp_path.as_ref(),
        }
    }
}

#[derive(Debug)]
pub struct Executable {
    location: ExecutableLocation,
    name: String,
    interpreter: Option<PathBuf>,
    libraries: Vec<String>,
    search_paths: SearchPaths,
}

impl Executable {
    fn load_impl(
        location: ExecutableLocation,
        name: String,
        propagated_rpaths: Option<Vec<PathBuf>>,
    ) -> Result<Self> {
        info!("exe: loading {}", location.as_ref().display());
        let buffer = fs::read(location.as_ref())?;
        let elf = Elf::parse(buffer.as_slice())?;
        let interpreter = if let Some(interp) = elf.interpreter {
            Some(interp.into())
        } else {
            let interp = default_interpreter(&location)?;
            if let Some(interp) = &interp {
                info!("exe: using default interpreter {}", interp.display());
            } else {
                warn!("exe: interpreter could not be found. static or compressed executable?");
            }
            interp
        };
        let mut search_paths = collect_paths(&elf, location.as_ref())?;
        let libraries = elf.libraries.into_iter().map(ToOwned::to_owned).collect();

        if let Some(paths) = propagated_rpaths {
            search_paths.append_rpath(paths);
        }

        let exe = Executable {
            location,
            name,
            interpreter,
            libraries,
            search_paths,
        };

        debug!("exe: loaded {:?}", exe);
        Ok(exe)
    }

    fn load_with_rpaths<P>(exe_path: P, propagated_rpaths: Option<Vec<PathBuf>>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = exe_path.as_ref();
        if path.is_dir() {
            return Err(Error::IO(io::Error::from_raw_os_error(21)));
        }

        let location = ExecutableLocation::Fixed(path.to_owned());
        // unwrap is ok because `path` here is not a directory
        let file_name = path.file_name().unwrap();
        let file_name_str = file_name
            .to_str()
            .ok_or_else(|| Error::PathEncoding(file_name.to_os_string()))?
            .to_string();
        Executable::load_impl(location, file_name_str, propagated_rpaths)
    }

    pub fn load<P>(exe_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        Executable::load_with_rpaths(exe_path, None)
    }

    pub fn path(&self) -> &Path {
        self.location.as_ref()
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn interpreter(&self) -> Option<&PathBuf> {
        self.interpreter.as_ref()
    }

    pub fn dynamic_libraries<P>(&self, cc_path: P) -> Result<Vec<PathBuf>>
    where
        P: AsRef<Path>,
    {
        let interpreter = if let Some(interp) = &self.interpreter {
            interp
        } else {
            warn!("exe: requesting dynamic libraries of the executable without the interpreter");
            return Ok(Vec::new());
        };

        let resolver = resolver::Resolver::new(&interpreter, &self.search_paths, cc_path.as_ref())?;

        let mut paths = Vec::new();
        for lib in &self.libraries {
            let path = resolver.lookup(&lib)?;
            debug!("exe: found shared object {} => {}", lib, path.display());

            // TODO: cache once traversed
            // TODO: deal with semantic inconsistency (Executable on shared object)
            let mut children =
                Executable::load_with_rpaths(path.clone(), self.search_paths.rpath().cloned())?
                    .dynamic_libraries(cc_path.as_ref())?;

            paths.push(path);
            paths.append(&mut children);
        }

        Ok(paths)
    }

    pub fn compressed<P, T, I>(&self, upx_path: P, upx_opts: I) -> Result<Executable>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = T>,
        T: AsRef<OsStr>,
    {
        let result_path = NamedTempFile::new()?.into_temp_path();

        // NOTE: We use `TempPath` to delete it in `Drop::drop`, and TempPath can be obtained from `NamedTempFile`.
        // However, upx requires nonexistent output path. So we delete it once here.
        // NOTE: We expect `fs::remove_file` to remove the file immediately, though the
        // documentation says 'there is no guarantee that the file is immediately deleted'.
        fs::remove_file(&result_path)?;
        assert!(!result_path.exists());
        let output = Command::new(upx_path.as_ref())
            .args(upx_opts)
            .arg("--no-progress")
            .arg(self.path().canonicalize()?)
            .arg("-o")
            .arg(&result_path)
            .output_with_log()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::Upx(stderr));
        }

        Executable::load_impl(
            ExecutableLocation::Temporary(result_path),
            self.name().clone(),
            None,
        )
    }
}

fn default_interpreter<P>(exe: P) -> Result<Option<PathBuf>>
where
    P: AsRef<Path>,
{
    // from the source code of ldd(1); TODO: deal with hardcoded paths
    let rtld_list = &[
        "/usr/lib/ld-linux.so.2",
        "/usr/lib64/ld-linux-x86-64.so.2",
        "/usr/libx32/ld-linux-x32.so.2",
        "/lib/ld-linux.so.2",
        "/lib64/ld-linux-x86-64.so.2",
        "/libx32/ld-linux-x32.so.2",
    ];
    for rtld in rtld_list {
        let path = Path::new(rtld);
        if !path.exists() {
            continue;
        }

        let status = Command::new(rtld)
            .arg("--verify")
            .arg(exe.as_ref())
            .status_with_log()?;
        match status.code() {
            Some(0) | Some(2) => return Ok(Some(rtld.into())),
            _ => continue,
        }
    }

    Ok(None)
}

fn collect_paths(elf: &Elf<'_>, executable_path: &Path) -> Result<SearchPaths> {
    debug_assert!(executable_path.is_absolute());
    // unwrap is ok here because the path points to file and is absolute
    let origin = executable_path.parent().unwrap();
    let mut paths = SearchPaths::new(origin.into())?;

    if let Elf {
        dynamic: Some(dynamic),
        dynstrtab,
        ..
    } = elf
    {
        for d in &dynamic.dyns {
            if d.d_tag == DT_RUNPATH {
                paths.append_runpath(get_paths_in_strtab(d, dynstrtab)?);
            } else if d.d_tag == DT_RPATH {
                paths.append_rpath(get_paths_in_strtab(d, dynstrtab)?);
            }
        }
    }

    if let Some(paths_str) = env::var_os("LD_LIBRARY_PATH") {
        debug!(
            "executable: LD_LIBRARY_PATH={}",
            paths_str.to_string_lossy()
        );

        paths.append_ld_library_path(
            paths_str
                .into_vec()
                .split(|b| *b == b':' || *b == b';')
                .map(|x| OsStr::from_bytes(x)),
        );
    }

    Ok(paths)
}

fn get_paths_in_strtab(d: &Dyn, strtab: &Strtab<'_>) -> Result<Vec<PathBuf>> {
    let content = get_content_in_strtab(d, strtab)?;
    // assuming paths in DT_RPATH and DT_RUNPATH are separated by colons.
    Ok(content.split(':').map(Into::into).collect())
}

fn get_content_in_strtab(d: &Dyn, strtab: &Strtab<'_>) -> Result<String> {
    if let Some(x) = strtab.get(d.d_val as usize) {
        Ok(x?.to_owned())
    } else {
        Err(Error::ValueNotFoundInStrtab {
            tag: d.d_tag,
            val: d.d_val,
        })
    }
}
