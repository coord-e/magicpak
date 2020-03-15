use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::base::command_ext::CommandExt;
use crate::base::{Error, Result};

use goblin::elf::dynamic::{Dyn, DT_RPATH, DT_RUNPATH};
use goblin::elf::Elf;
use goblin::strtab::Strtab;
use log::{debug, info};

mod resolver;

#[derive(Debug)]
pub struct Executable {
    path: PathBuf,
    interpreter: PathBuf,
    libraries: Vec<String>,
    rpaths: Vec<PathBuf>,
    runpaths: Vec<PathBuf>,
}

impl Executable {
    pub fn load<P>(exe_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        info!("exe: loading {}", exe_path.as_ref().display());
        let buffer = fs::read(exe_path.as_ref())?;
        let elf = Elf::parse(buffer.as_slice())?;
        let path = exe_path.as_ref().to_owned();
        let interpreter = if let Some(interp) = elf.interpreter {
            interp.into()
        } else {
            let interp = default_interpreter(exe_path)?;
            info!("exe: using default interpreter {}", interp.display());
            interp
        };
        let (rpaths, runpaths) = collect_paths(&elf)?;
        let libraries = elf.libraries.into_iter().map(ToOwned::to_owned).collect();

        let exe = Executable {
            path,
            interpreter,
            libraries,
            rpaths,
            runpaths,
        };

        debug!("exe: loaded {:?}", exe);
        Ok(exe)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn interpreter(&self) -> &PathBuf {
        &self.interpreter
    }

    pub fn dynamic_libraries(&self) -> Result<Vec<PathBuf>> {
        let resolver = resolver::Resolver::new(&self.interpreter, &self.rpaths, &self.runpaths)?;

        let mut paths = Vec::new();
        for lib in &self.libraries {
            let path = resolver.lookup(&lib)?;
            debug!("exe: found shared object {} => {}", lib, path.display());

            // TODO: cache once traversed
            // TODO: deal with semantic inconsistency (Executable on shared object)
            // TODO: propatage rpaths into children (see ld.so(8))
            let mut children = Executable::load(path.clone())?.dynamic_libraries()?;

            paths.push(path);
            paths.append(&mut children);
        }

        Ok(paths)
    }
}

fn default_interpreter<P>(exe: P) -> Result<PathBuf>
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
            Some(0) | Some(2) => return Ok(rtld.into()),
            _ => continue,
        }
    }
    Err(Error::InterpretorNotFound)
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
