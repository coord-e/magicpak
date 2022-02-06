use std::ffi::{OsStr, OsString};
use std::io;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

use crate::base::{Error, Result};

#[derive(Hash, Default, Debug)]
pub struct SearchPaths {
    rpath: Option<Vec<PathBuf>>,
    runpath: Option<Vec<PathBuf>>,
    ld_library_path: Option<Vec<PathBuf>>,
    platform: OsString,
    origin: OsString,
}

impl SearchPaths {
    pub fn new(origin: OsString) -> Result<Self> {
        Ok(SearchPaths {
            rpath: None,
            runpath: None,
            ld_library_path: None,
            platform: auxv_platform()?,
            origin,
        })
    }

    pub fn rpath(&self) -> Option<&Vec<PathBuf>> {
        self.rpath.as_ref()
    }

    pub fn runpath(&self) -> Option<&Vec<PathBuf>> {
        self.runpath.as_ref()
    }

    pub fn iter_rpaths(&self) -> impl Iterator<Item = &Path> {
        self.rpath.iter().flat_map(|v| v.iter().map(AsRef::as_ref))
    }

    pub fn iter_runpaths(&self) -> impl Iterator<Item = &Path> {
        self.runpath
            .iter()
            .flat_map(|v| v.iter().map(AsRef::as_ref))
    }

    pub fn iter_ld_library_paths(&self) -> impl Iterator<Item = &Path> {
        self.ld_library_path
            .iter()
            .flat_map(|v| v.iter().map(AsRef::as_ref))
    }

    fn append<I, S>(paths: &mut Option<Vec<PathBuf>>, other: I, origin: &OsStr, platform: &OsStr)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let inner = paths.get_or_insert(Vec::new());
        inner.extend(
            other
                .into_iter()
                .map(|x| expand_tokens(x, origin, platform)),
        )
    }

    pub fn append_rpath<I, S>(&mut self, rpath: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let SearchPaths {
            origin, platform, ..
        } = self;

        Self::append(&mut self.rpath, rpath, origin, platform)
    }

    pub fn append_runpath<I, S>(&mut self, runpath: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let SearchPaths {
            origin, platform, ..
        } = self;

        Self::append(&mut self.runpath, runpath, origin, platform)
    }

    pub fn append_ld_library_path<I, S>(&mut self, ld_library_path: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let SearchPaths {
            origin, platform, ..
        } = self;

        Self::append(&mut self.ld_library_path, ld_library_path, origin, platform)
    }
}

fn expand_tokens<S, T, U>(input: S, origin: T, platform: U) -> PathBuf
where
    S: AsRef<OsStr>,
    T: AsRef<OsStr>,
    U: AsRef<OsStr>,
{
    let input = input.as_ref();

    let mut result = OsString::new();
    result.reserve(input.len());

    let mut buffer = Vec::new();
    buffer.reserve(16);

    enum ParseState {
        Standby,
        Scanning,
        ScanningBraced,
    }

    let state = input
        .as_bytes()
        .iter()
        .fold(ParseState::Standby, |state, b| match (state, b) {
            (ParseState::Standby, b'$') => {
                result.push(OsStr::from_bytes(&buffer));
                buffer.clear();
                ParseState::Scanning
            }
            (ParseState::Scanning, b'{') if buffer.is_empty() => ParseState::ScanningBraced,
            (ParseState::Scanning, b'$') => {
                result.push(substitute(&buffer, &origin, &platform));
                buffer.clear();

                ParseState::Scanning
            }
            (ParseState::Scanning, b'/') => {
                result.push(substitute(&buffer, &origin, &platform));
                buffer.clear();

                buffer.push(*b);
                ParseState::Standby
            }
            (ParseState::ScanningBraced, b'}') => {
                result.push(substitute(&buffer, &origin, &platform));
                buffer.clear();
                ParseState::Standby
            }
            (s, b) => {
                buffer.push(*b);
                s
            }
        });

    match state {
        ParseState::ScanningBraced => {
            tracing::warn!(
                token = ?String::from_utf8_lossy(&buffer),
                "search_paths: unterminated braced token",
            );
            result.push(OsStr::from_bytes(&buffer));
        }
        ParseState::Scanning => {
            result.push(substitute(&buffer, &origin, &platform));
        }
        ParseState::Standby => {
            result.push(OsStr::from_bytes(&buffer));
        }
    }

    if input != result {
        tracing::info!(
            input = %input.to_string_lossy(),
            result = %result.to_string_lossy(),
            "search_paths: expand",
        );
    }

    result.into()
}

fn substitute<S, T>(s: &[u8], origin: S, platform: T) -> OsString
where
    S: AsRef<OsStr>,
    T: AsRef<OsStr>,
{
    match s {
        b"ORIGIN" => origin.as_ref().to_owned(),
        b"LIB" => match is_64bit(&platform) {
            Some(true) => OsStr::new("lib64").to_owned(),
            Some(false) => OsStr::new("lib").to_owned(),
            None => {
                tracing::warn!(
                    platform = %platform.as_ref().to_string_lossy(),
                    "search_paths: assuming 32-bit platform",
                );
                OsStr::new("lib").to_owned()
            }
        },
        b"PLATFORM" => platform.as_ref().to_owned(),
        _ => {
            tracing::warn!(
                token = %format!("${}", String::from_utf8_lossy(s)),
                "search_paths: unknown dynamic string token",
            );
            OsString::from_vec([&[b'$'], s].concat().to_vec())
        }
    }
}

fn is_64bit<S>(platform: S) -> Option<bool>
where
    S: AsRef<OsStr>,
{
    match platform.as_ref().to_string_lossy().as_ref() {
        "x86_64" | "amd64" | "aarch64" => Some(true),
        "i386" | "i686" | "x86" | "arm" => Some(false),
        _ => None,
    }
}

fn auxv_platform() -> Result<OsString> {
    let mut reader = crt0stack::Reader::from_environ().done();
    let platform = reader
        .find_map(|entry| {
            if let crt0stack::Entry::Platform(platform) = entry {
                Some(platform)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            Error::IO(io::Error::new(
                io::ErrorKind::NotFound,
                "could not find AT_PLATFORM auxval",
            ))
        })?;
    tracing::debug!(%platform, "search_paths: read platform from auxv");
    Ok(platform.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute() {
        let origin = "/home/user/";
        let platform = "x86_64";
        assert_eq!(substitute(b"ORIGIN", origin, platform), origin);
        assert_eq!(substitute(b"PLATFORM", origin, platform), platform);

        assert_eq!(substitute(b"LIB", origin, "x86_64"), "lib64");
        assert_eq!(substitute(b"LIB", origin, "x86"), "lib");

        assert_eq!(substitute(b"WTF", origin, platform), "$WTF");
    }

    #[test]
    fn test_expand_tokens() {
        let origin = "/home/user/";
        let platform = "x86_64";
        assert_eq!(
            expand_tokens("$ORIGIN/$LIB", origin, platform),
            PathBuf::from(format!("{}/lib64", origin))
        );
        assert_eq!(
            expand_tokens("${ORIGIN}/${LIB}", origin, platform),
            PathBuf::from(format!("{}/lib64", origin))
        );
        assert_eq!(
            expand_tokens("/$ORIGIN$LIB", origin, platform),
            PathBuf::from(format!("/{}lib64", origin))
        );
        assert_eq!(
            expand_tokens("/lib/$PLATFORM", origin, platform),
            PathBuf::from(format!("/lib/{}", platform))
        );
        assert_eq!(
            expand_tokens("${PLATFORM}${ORIGIN}", origin, platform),
            PathBuf::from(format!("{}{}", platform, origin))
        );
    }
}
