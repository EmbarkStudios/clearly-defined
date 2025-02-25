<!-- markdownlint-disable blanks-around-headings blanks-around-lists no-duplicate-heading -->

# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate
## [0.3.1] - 2025-02-25
### Changed
- [PR#6](https://github.com/EmbarkStudios/clearly-defined/pull/6) updated crates.

## [0.3.0] - 2024-05-31
### Changed
- [PR#5](https://github.com/EmbarkStudios/clearly-defined/pull/5) updated http and reqwest dependencies, and got rid of the `native-tls` and `rustls` features in favor of always using `rustls-tls`.

## [0.2.1] - 2022-02-04
### Changed
- [PR#3](https://github.com/EmbarkStudios/clearly-defined/pull/3) removed usage of `chrono`.

## [0.2.0] - 2021-10-25
### Added
- First pass implementation of basic support for retrieving the license information for one or more coordinates.
- Adds an optional blocking and asynchronous client implementation that can be activated with features if desired.

## [0.1.0] - 2019-02-15
### Added
- Initial add. This does absolutely nothing I just want to squat the name on crates.io

<!-- next-url -->
[Unreleased]: https://github.com/EmbarkStudios/cargo-about/compare/0.3.1...HEAD
[0.3.1]: https://github.com/EmbarkStudios/cargo-about/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/EmbarkStudios/cargo-about/compare/0.2.1...0.3.0
[0.2.1]: https://github.com/EmbarkStudios/cargo-about/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/EmbarkStudios/clearly-defined/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/EmbarkStudios/clearly-defined/releases/tag/0.1.0
