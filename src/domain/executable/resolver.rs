use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, str};

use crate::base::command_ext::CommandExt;
use crate::base::{Error, Result};

use tempfile::{NamedTempFile, TempPath};

static RESOLVER_SOURCE_CODE: &str = r"
#define _GNU_SOURCE
#include <dlfcn.h>
#include <link.h>

#include <stdio.h>

int main(int argc, char** argv) {
  char* name = argv[1];
  void* handle = dlopen(name, RTLD_LAZY);
  if (handle == NULL) {
    fputs(dlerror(), stderr);
    return 1;
  }
  struct link_map* link_map;
  if (dlinfo(handle, RTLD_DI_LINKMAP, &link_map) != 0) {
    fputs(dlerror(), stderr);
    return 2;
  }
  puts(link_map->l_name);
  dlclose(handle);
}";

pub struct Resolver<'a> {
    program_path: TempPath,
    rpaths: &'a Vec<PathBuf>,
    runpaths: &'a Vec<PathBuf>,
}

impl<'a> Resolver<'a> {
    pub fn new<P>(interp: P, rpaths: &'a Vec<PathBuf>, runpaths: &'a Vec<PathBuf>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut source = NamedTempFile::new()?;
        write!(source, "{}", RESOLVER_SOURCE_CODE)?;
        let source_path = source.into_temp_path();
        let program_path = NamedTempFile::new()?.into_temp_path();
        let output = Command::new("gcc")
            .arg(format!("-Wl,-dynamic-linker,{}", interp.as_ref().display()))
            .arg("-ldl")
            .arg("-o")
            .arg(&program_path)
            .arg("-xc")
            .arg(&source_path)
            .output_with_log()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::ResolverCompilation(stderr));
        }
        source_path.close()?;

        Ok(Resolver {
            program_path,
            rpaths,
            runpaths,
        })
    }

    // lookup_rpath --> lookup_env --> lookup_runpath --> lookup_rest
    // TODO: implement Rpath token expansion
    // TODO: take secure-execution mode into consideration
    pub fn lookup(&self, name: &str) -> Result<PathBuf> {
        if let Some(path) = self.lookup_rpath(name) {
            return Ok(path);
        }

        if let Some(path) = self.lookup_env(name)? {
            return Ok(path);
        }

        if let Some(path) = self.lookup_runpath(name) {
            return Ok(path);
        }

        self.lookup_rest(name)
    }

    fn lookup_rpath(&self, name: &str) -> Option<PathBuf> {
        self.rpaths.iter().find_map(|x| try_joined(x, name))
    }

    fn lookup_runpath(&self, name: &str) -> Option<PathBuf> {
        self.runpaths.iter().find_map(|x| try_joined(x, name))
    }

    fn lookup_env(&self, name: &str) -> Result<Option<PathBuf>> {
        if let Some(paths_os_str) = env::var_os("LD_LIBRARY_PATH") {
            // TODO: it would be better to process within OsString
            let paths_str = paths_os_str.into_string().map_err(Error::PathEncoding)?;
            let result = paths_str
                .split(|c| c == ':' || c == ';')
                .find_map(|x| try_joined(x, name));
            Ok(result)
        } else {
            Ok(None)
        }
    }

    fn lookup_rest(&self, name: &str) -> Result<PathBuf> {
        let output = Command::new(&self.program_path)
            .arg(name)
            .env_clear()
            .output_with_log()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::SharedLibraryLookup(stderr));
        }

        Ok(str::from_utf8(&output.stdout)?.trim().to_string().into())
    }
}

fn try_joined<P, Q>(path1: P, path2: Q) -> Option<PathBuf>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let joined = path1.as_ref().join(path2);
    if joined.exists() {
        Some(joined)
    } else {
        None
    }
}
