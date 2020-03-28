use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::base::log::CommandLogExt;
use crate::base::{Error, Result};

use goblin::elf::dynamic::{Dyn, DT_RPATH, DT_RUNPATH};
use goblin::elf::Elf;
use goblin::strtab::Strtab;
use log::{debug, info, warn};
use tempfile::{NamedTempFile, TempPath};

mod resolver;

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
    rpaths: Vec<PathBuf>,
    runpaths: Vec<PathBuf>,
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
        let (mut rpaths, runpaths) = collect_paths(&elf)?;
        let libraries = elf.libraries.into_iter().map(ToOwned::to_owned).collect();

        if let Some(mut paths) = propagated_rpaths {
            rpaths.append(&mut paths);
        }

        let exe = Executable {
            location,
            name,
            interpreter,
            libraries,
            rpaths,
            runpaths,
        };

        debug!("exe: loaded {:?}", exe);
        Ok(exe)
    }

    fn load_with_rpaths<P>(exe_path: P, propagated_rpaths: Option<Vec<PathBuf>>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = exe_path.as_ref().canonicalize()?;
        let location = ExecutableLocation::Fixed(path.to_owned());
        // unwrap is ok because `path` here is canonicalized
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

    pub fn dynamic_libraries(&self) -> Result<Vec<PathBuf>> {
        let interpreter = if let Some(interp) = &self.interpreter {
            interp
        } else {
            warn!("exe: requesting dynamic libraries of the executable without the interpreter");
            return Ok(Vec::new());
        };

        let resolver = resolver::Resolver::new(&interpreter, &self.rpaths, &self.runpaths)?;

        let mut paths = Vec::new();
        for lib in &self.libraries {
            let path = resolver.lookup(&lib)?;
            debug!("exe: found shared object {} => {}", lib, path.display());

            // TODO: cache once traversed
            // TODO: deal with semantic inconsistency (Executable on shared object)
            let mut children =
                Executable::load_with_rpaths(path.clone(), Some(self.rpaths.clone()))?
                    .dynamic_libraries()?;

            paths.push(path);
            paths.append(&mut children);
        }

        Ok(paths)
    }

    pub fn compressed<S, T, I>(&self, upx_path: S, upx_opts: I) -> Result<Executable>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let upx = which::which(upx_path.as_ref())?;
        let result_path = NamedTempFile::new()?.into_temp_path();

        // NOTE: We use `TempPath` to delete it in `Drop::drop`, and TempPath can be obtained from `NamedTempFile`.
        // However, upx requires nonexistent output path. So we delete it once here.
        // TODO: We expect `fs::remove_file` to remove the file immediately, though the
        // documentation says 'there is no guarantee that the file is immediately deleted'.
        fs::remove_file(&result_path)?;
        let output = Command::new(upx)
            .args(upx_opts.into_iter().map(|x| x.as_ref().to_owned())) // TODO: do we really need to own x here?
            .arg("--no-progress")
            .arg(self.path())
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
    // from ldd(1); TODO: deal with hardcoded paths
    let rtld_list = &[
        "/usr/lib/ld-linux.so.2",
        "/usr/lib64/ld-linux-x86-64.so.2",
        "/usr/libx32/ld-linux-x32.so.2",
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

fn collect_paths(elf: &Elf<'_>) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut rpaths = Vec::new();
    let mut runpaths = Vec::new();
    match elf {
        Elf {
            dynamic: Some(dynamic),
            dynstrtab,
            ..
        } => {
            for d in &dynamic.dyns {
                if d.d_tag == DT_RUNPATH {
                    runpaths.append(&mut get_paths_in_strtab(d, dynstrtab)?);
                } else if d.d_tag == DT_RPATH {
                    rpaths.append(&mut get_paths_in_strtab(d, dynstrtab)?);
                }
            }
            Ok((rpaths, runpaths))
        }
        _ => Ok((Vec::new(), Vec::new())),
    }
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
