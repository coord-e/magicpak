use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use crate::error::{Error, Result};

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

pub struct Resolver {
    program_path: TempPath,
}

impl Resolver {
    pub fn new<P, Q, I>(interp: P, rpaths: I) -> Result<Self>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = Q>,
        Q: AsRef<Path>,
    {
        let mut source = NamedTempFile::new()?;
        write!(source, "{}", RESOLVER_SOURCE_CODE)?;
        let source_path = source.into_temp_path();
        let program_path = NamedTempFile::new()?.into_temp_path();
        let output = Command::new("gcc")
            .arg(format!("-Wl,-dynamic-linker,{}", interp.as_ref().display()))
            .args(
                rpaths
                    .into_iter()
                    .map(|x| format!("-Wl,-rpath,{}", x.as_ref().display()))
                    .collect::<Vec<_>>(),
            )
            .arg("-ldl")
            .arg("-o")
            .arg(&program_path)
            .arg("-xc")
            .arg(&source_path)
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::ResolverCompilation(stderr));
        }
        source_path.close()?;

        Ok(Resolver { program_path })
    }

    pub fn lookup(&self, name: &str) -> Result<PathBuf> {
        let output = Command::new(&self.program_path).arg(name).output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::SharedLibraryLookup(stderr));
        }

        Ok(str::from_utf8(&output.stdout)?.trim().to_string().into())
    }
}
