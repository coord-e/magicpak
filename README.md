# magicpak

[![Actions Status](https://github.com/coord-e/magicpak/workflows/Test%20and%20Lint/badge.svg)](https://github.com/coord-e/r53ddns/actions?workflow=Test+and+Lint)
[![Actions Status](https://github.com/coord-e/magicpak/workflows/Release/badge.svg)](https://github.com/coord-e/r53ddns/actions?workflow=Release)

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

That's it! The resulting image only contains what your executable requires at runtime. You can find some useful examples of `magicpak` under [example/](/example).

## Feature

`magicpak` is a command-line utility that analyzes and bundles runtime dependencies of the executable.  `magicpak` basically collects all shared object dependencies that are required by a dynamic linker at runtime. Additionally, `magicpak`'s contributions are summarized as follows:

- **Simple**. You can build a minimal image just by adding a few lines to your `Dockerfile`.
- **Full featured**. You can bundle, test, and compress your executable at once. You can focus on your business because `magicpak` handles all `Dockerfile`-specific matters to decrease image size.
- **Dynamic analysis**. `--dynamic` flag enables a dynamic analysis that can discover dependencies other than dynamically linked libraries.
- **Flexible**. We expose a full control of resulting bundle with a family of options like `--include` and  `--exclude`. You can deal with dependencies that cannot be detected automatically.
- **Stable**. We don't parse undocumented and sometimes inaccurate `ldd (1)` outputs. Instead, we use `dlopen (3)` and `dlinfo (3)` in glibc to query shared library locations to `ld.so (8)`.

## Docker images

We provide some base images that contains `magicpak` and its optional dependencies to get started.

| name                                                         | description                                                  |
| ------------------------------------------------------------ | ------------------------------------------------------------ |
| [magicpak/debian ![magicpak/debian](https://img.shields.io/docker/image-size/magicpak/debian?sort=date)](https://hub.docker.com/r/magicpak/debian) | [library/debian](http://hub.docker.com/_/debian) with `magicpak` |
| [magicpak/cc ![magicpak/cc](https://img.shields.io/docker/image-size/magicpak/cc?sort=date)](https://hub.docker.com/r/magicpak/cc) | [library/debian](http://hub.docker.com/_/debian) with `build-essential`, `clang`, and `magicpak` |
| [magicpak/haskell ![magicpak/haskell](https://img.shields.io/docker/image-size/magicpak/haskell?sort=date)](https://hub.docker.com/r/magicpak/haskell) | [library/haskell](http://hub.docker.com/_/haskell) with `magicpak` |
| [magicpak/stack-build ![magicpak/stack-build](https://img.shields.io/docker/image-size/magicpak/stack-build?sort=date)](https://hub.docker.com/r/magicpak/stack-build) | [fpco/stack-build](http://hub.docker.com/r/fpco/stack-build) with `magicpak` |
| [magicpak/rust ![magicpak/rust](https://img.shields.io/docker/image-size/magicpak/rust?sort=date)](https://hub.docker.com/r/magicpak/rust) | [library/rust](http://hub.docker.com/_/rust) with `magicpak` |

## Usage

```
  magicpak [OPTIONS] <INPUT> <OUTPUT>

    -r, --install-to <PATH>          Specify the installation path of the executable in the bundle
    -e, --exclude <GLOB>...          Exclude files/directories from the resulting bundle with glob patterns
    -i, --include <GLOB>...          Additionally include files/directories with glob patterns
        --mkdir <PATH>...            Make directories in the resulting bundle
    -t, --test                       Enable testing
        --test-command <COMMAND>     Specify the test command to use in --test
        --test-stdin <CONTENT>       Specify stdin content supplied to the test command in --test
        --test-stdout <CONTENT>      Test stdout of the test command
    -d, --dynamic                    Enable dynamic analysis
        --dynamic-arg <ARG>...       Specify arguments passed to the executable in --dynamic
        --dynamic-stdin <CONTENT>    Specify stdin content supplied to the executable in --dynamic
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
