use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

use goblin::elf::dynamic::{DT_RPATH, DT_RUNPATH};
use goblin::elf::Elf;

mod resolver;

pub struct Executable {
    path: PathBuf,
    interpreter: PathBuf,
    libraries: Vec<String>,
    rpaths: Vec<PathBuf>,
}

impl Executable {
    pub fn load<P>(exe_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let buffer = fs::read(exe_path.as_ref())?;
        let elf = Elf::parse(buffer.as_slice())?;
        let path = exe_path.as_ref().to_owned();
        let interpreter = if let Some(interp) = elf.interpreter {
            interp.into()
        } else {
            default_interpreter(exe_path)?
        };
        let rpaths = rpaths(&elf)?;
        let libraries = elf.libraries.into_iter().map(ToOwned::to_owned).collect();
        Ok(Executable {
            path,
            interpreter,
            libraries,
            rpaths,
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn interpreter(&self) -> &PathBuf {
        &self.interpreter
    }

    pub fn dynamic_libraries(&self) -> Result<Vec<PathBuf>> {
        let resolver = resolver::Resolver::new(&self.interpreter, &self.rpaths)?;
        self.libraries
            .iter()
            .map(|lib| resolver.lookup(lib))
            .collect()
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
            .status()?;
        match status.code() {
            Some(0) | Some(2) => return Ok(rtld.into()),
            _ => continue,
        }
    }
    Err(Error::InterpretorNotFound)
}

fn rpaths(elf: &Elf<'_>) -> Result<Vec<PathBuf>> {
    let mut rpaths = Vec::new();
    match elf {
        Elf {
            dynamic: Some(dynamic),
            dynstrtab,
            ..
        } => {
            for d in &dynamic.dyns {
                if d.d_tag == DT_RUNPATH || d.d_tag == DT_RPATH {
                    if let Some(epath) = dynstrtab.get(d.d_val as usize) {
                        rpaths.push(epath?.into());
                    } else {
                        return Err(Error::ValueNotFoundInStrtab {
                            tag: d.d_tag,
                            val: d.d_val,
                        });
                    }
                }
            }
            Ok(rpaths)
        }
        _ => Ok(Vec::new()),
    }
}
