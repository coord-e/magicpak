# magicpak

[![Actions Status](https://github.com/coord-e/magicpak/workflows/Test%20and%20Lint/badge.svg)](https://github.com/coord-e/r53ddns/actions?workflow=Test+and+Lint)
[![Actions Status](https://github.com/coord-e/magicpak/workflows/Release/badge.svg)](https://github.com/coord-e/r53ddns/actions?workflow=Release)
[![License](https://img.shields.io/crates/l/mkbookpdf)](https://crates.io/crates/mkbookpdf)

`magicpak` enables you to build minimal docker images without any bothersome preparation such as static linking.

```dockerfile
# You prepare /bin/your_executable here...

ADD https://github.com/coord-e/magicpak/releases/latest/download/magicpak-x86_64-unknown-linux-musl /usr/bin/magicpak
RUN chmod +x /usr/bin/magicpak

RUN /usr/bin/magicpak -v /bin/your_executable /bundle

FROM scratch
COPY --from=0 /bundle /.

CMD ["/bin/your_executable"]
```

That's it! The resulting image shall only contain what your executable requires at runtime. You can find more useful examples of `magicpak` under [example/](/example).

## Feature

`magicpak` is a command-line utility that analyzes and bundles runtime dependencies of the executable.  `magicpak` basically collects all shared object dependencies that are required by a dynamic linker at runtime. Additionally, `magicpak`'s contributions are summarized as follows:

- **Simple**. You can build a minimal image just by adding a few lines to your `Dockerfile`.
- **Full-featured**. You can bundle, test, and compress your executable at once. You can focus on your business because `magicpak` handles all `Dockerfile`-specific matters to decrease image size.
- **Dynamic analysis**. `--dynamic` flag enables a dynamic analysis that can discover dependencies other than dynamically linked libraries.
- **Flexible**. We expose a full control of resulting bundle with a family of options like `--include` and  `--exclude`. You can deal with dependencies that cannot be detected automatically.
- **Stable**. We don't parse undocumented and sometimes inaccurate ldd(1) outputs. Instead, we use dlopen(3) and dlinfo(3) in glibc to query shared library locations to ld.so(8).

`magicpak` is especially useful when you find it difficult to produce a statically linked executable. Also, `magicpak` is powerful when building from source is bothering or the source code is not public, because `magicpak` only requires the executable to build a minimal docker image.

## Usage

You can start with `magicpak path/to/executable path/to/output`. This simply analyzes runtime dependencies of your executable statically and put everything your executable needs in runtime to the specified output directory. Once they've bundled, we can simply copy them to the `scratch` image in the second stage as follows.

```dockerfile
RUN magicpak path/to/executable /bundle

FROM scratch
COPY --from=0 /bundle /.
```

Some executables work well in this way. However, others fail to run properly because `magicpak`'s static analysis isn't enough to detect all files needed by them at runtime. For this case, `magicpak` has `--include <GLOB>` option to specify the missing requirements manually. Moreover, you can use `--dynamic` to automatically include files that are accessed by the executable during execution.

Despite our careful implementation, our analysis is unreliable in a way because we can't completely determine the runtime behavior before its execution. To ensure that `magicpak` collected all dependencies to perform a specific task, `--test` option is implemented. `--test` enables testing of the resulting bundle using chroot(2).

The size of the resulting image is our main concern. `magicpak` supports executable compression using `upx`. You can enable it with `--compress`.

### Supported options

```
  magicpak [OPTIONS] <INPUT> <OUTPUT>

    -r, --install-to <PATH>          Specify the installation path of the executable in the bundle
    -e, --exclude <GLOB>...          Exclude files/directories from the resulting bundle with glob patterns
    -i, --include <GLOB>...          Additionally include files/directories with glob patterns
        --mkdir <PATH>...            Make directories in the resulting bundle
    -d, --dynamic                    Enable dynamic analysis
        --dynamic-arg <ARG>...       Specify arguments passed to the executable in --dynamic
        --dynamic-stdin <CONTENT>    Specify stdin content supplied to the executable in --dynamic
    -t, --test                       Enable testing
        --test-command <COMMAND>     Specify the test command to use in --test
        --test-stdin <CONTENT>       Specify stdin content supplied to the test command in --test
        --test-stdout <CONTENT>      Test stdout of the test command
    -c, --compress                   Compress the executable with npx
        --upx-arg <ARG>...           Specify arguments passed to upx in --compress
        --upx <PATH or NAME>         Specify the path or name of upx that would be used in compression
        --busybox <PATH or NAME>     Specify the path or name of busybox that would be used in testing
        --cc <PATH or NAME>          Specify the path or name of c compiler
        --log-level <LEVEL>          Specify the log level
    -v, --verbose                    Verbose mode, same as --log-level Info
    -h, --help                       Prints help information
    -V, --version                    Prints version information
```

### Docker images

We provide some base images that contain `magicpak` and its optional dependencies to get started.

| name                                                         | description                                                  |
| ------------------------------------------------------------ | ------------------------------------------------------------ |
| [magicpak/debian ![magicpak/debian](https://img.shields.io/docker/pulls/magicpak/debian)](https://hub.docker.com/r/magicpak/debian) | [library/debian](http://hub.docker.com/_/debian) with `magicpak` |
| [magicpak/cc ![magicpak/cc](https://img.shields.io/docker/pulls/magicpak/cc)](https://hub.docker.com/r/magicpak/cc) | [library/debian](http://hub.docker.com/_/debian) with `build-essential`, `clang`, and `magicpak` |
| [magicpak/haskell ![magicpak/haskell](https://img.shields.io/docker/pulls/magicpak/haskell)](https://hub.docker.com/r/magicpak/haskell) | [library/haskell](http://hub.docker.com/_/haskell) with `magicpak` |
| [magicpak/rust ![magicpak/rust](https://img.shields.io/docker/pulls/magicpak/rust)](https://hub.docker.com/r/magicpak/rust) | [library/rust](http://hub.docker.com/_/rust) with `magicpak` |

### Example

The following is a dockerfile using `magicpak` for a docker image of [`brittany`](https://github.com/lspitzner/brittany), a formatter for Haskell. The resulting image size is just 15.6MB. ([example/brittany](/example/brittany))

```dockerfile
FROM magicpak/haskell:8

RUN cabal new-update
RUN cabal new-install brittany

RUN magicpak $(which brittany) /bundle -v  \
      --dynamic                            \
      --dynamic-stdin "a = 1"              \
      --compress                           \
      --upx-arg -9                         \
      --upx-arg --brute                    \
      --test                               \
      --test-stdin "a= 1"                  \
      --test-stdout "a = 1"                \
      --install-to /bin/

FROM scratch
COPY --from=0 /bundle /.

CMD ["/bin/brittany"]
```

## Disclaimer

`magicpak` comes with absolutely no warranty. There's no guarantee that the processed bundle works properly and identically to the original executable. Although I had no problem using `magicpak` for building various kinds of images, it is recommended to use this with caution and make a careful examination of the resulting bundle.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
