use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

use goblin::elf::dynamic::{DT_RPATH, DT_RUNPATH};
use goblin::elf::Elf;

mod resolver;

pub struct Executable<'a> {
    path: PathBuf,
    elf: Elf<'a>,
    _buffer: Vec<u8>,
}

impl Executable<'_> {
    pub fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + Clone,
    {
        let buffer = fs::read(path.clone())?;
        let elf = Elf::parse(unsafe { std::mem::transmute(buffer.as_slice()) })?;
        Ok(Executable {
            path: path.as_ref().to_owned(),
            elf,
            _buffer: buffer,
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn interpreter(&self) -> Option<PathBuf> {
        self.elf.interpreter.map(Into::into)
    }

    pub fn dynamic_libraries(&self) -> Result<Vec<PathBuf>> {
        match &self.elf {
            Elf {
                interpreter: Some(interp),
                libraries,
                ..
            } => {
                let resolver = resolver::Resolver::new(interp, self.rpaths()?)?;
                libraries
                    .into_iter()
                    .map(|lib| resolver.lookup(&lib))
                    .collect()
            }
            _ => Ok(Vec::new()),
        }
    }

    fn rpaths(&self) -> Result<Vec<PathBuf>> {
        let mut rpaths = Vec::new();
        match &self.elf {
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
}
