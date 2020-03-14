# magicpak

`magicpak` magically enables you to build minimal docker images without any bothersome requirements such as static linking.

```dockerfile
RUN curl -sSf https://get.coord-e.com/magicpak | sh

# You prepare /bin/your_executable here

RUN magicpak /bin/your_executable /bundle

FROM scratch
COPY --from=0 /bundle /.

RUN ["/bin/your_executable"]
```

That's it! The resulting image only contains what your executable requires at runtime.

## Usage

```
magicpak [OPTIONS...] PATH

  -t  --test COMMAND      enable testing
  -d  --dynamic [COMMAND] enable dynamic analysis (command defaults to -t)
  -i  --include GLOB      additionally include file/directory
  -e  --exclude GLOB      exclude file/directory from the distribution
  --mkdir PATH            create file/directory in the distribution
  -u  --upx [OPTIONS..]   run upx on executables (requires upx)
  -l  --include-locales [LOCALE...]  include locale files
  -r  --install-to PATH   specify installation path of the binary in the distribution
  --toralent              error toralent mode
  -d  --dry-run           change nothing in wild but log them
  -v  --verbose           verbose mode
```

## Disclaimer

`magicpak` comes with absolutely no warranty. There's no guarantee that the processed bundle works properly and identically to the original executable. Although I had no problem using `magicpak` for building various kinds of images, it is recommended to use this with caution and make careful examination of the resulting bundle.

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
