# Changelog

All notable changes to this project will be documented in this file.

## [1.2.0] - 2021-01-11

- Fixed infinite recursion caused by mutually dependent shared libraries. (#[3](https://github.com/coord-e/magicpak/pulls/3))
- Fixed Clippy warnings. (#[4](https://github.com/coord-e/magicpak/pulls/4))
- Updated dependencies.
- Changed how magicpak images are tagged.

## [1.1.0] - 2020-05-05

- Fixed the order of `-ldl` option in resolver compilation. (#[1](https://github.com/coord-e/magicpak/pulls/1))

## [1.0.3] - 2020-04-14

- Fixed `--test` behavior when the resulting bundle contains `/bin/`.

## [1.0.2] - 2020-04-14

- Fixed `--compress` when the executable is symlinked.
- Added many test cases.

## [1.0.1] - 2020-04-11

- Fixed a problem on CI.

## [1.0.0] - 2020-04-11

- Added detailed explanation of usage to README.
- Fixed bundled executable path when it is symlinked.

## [0.1.0] - 2020-04-03

- Initial release.
