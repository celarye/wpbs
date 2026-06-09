# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-rc.2] - 2026-06-14

### Added

#### Services

- Reimplement minimal Discord gateway caching. (#10)

### Fixed

#### Runtime

- Correct the WASI guest path of the plugin workspace. (#11)

### Changed

#### Core

- Remove redundant initialization function. (#11)
- Remove early shutdown support. (#11)
- Split post setup logic up. (#11)

#### Runtime

- The new Twilight API changes how forms are constructed, this is reflected in the
Discord WIT interface. (#10)

#### Miscellaneous

- Add debug logs and prevent unnecessary logs. (#10)
- Update dependencies. (#10)
- Add info and debug logs to improve status reporting. (#11)

## [0.1.0-rc.1] - 2026-06-05

### Added

#### Core

- Configuration system supporting system services, as well as per-plugin environment
variables, custom settings, and permissions.
- Registry support for resolving, downloading, and caching plugins.
- Key-value database ([`fjall`](https://github.com/fjall-rs/fjall)) for persistent
plugin state storage.

#### Runtime

- WebAssembly Component Model-based plugin system (using [`wasmtime`](https://github.com/bytecodealliance/wasmtime)).

#### Services

- Discord service supporting Gateway events, HTTP requests, and interaction registration.
- Job scheduler service for executing cron-based tasks.

[0.1.0-rc.2]: https://github.com/wpbs-rs/wpbs/compare/0.1.0-rc.1...0.1.0-rc.2
[0.1.0-rc.1]: https://github.com/wpbs-rs/wpbs/releases/tag/0.1.0-rc.1
