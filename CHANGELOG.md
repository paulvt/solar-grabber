# Changelog

All notable changes to Solar Grabber will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.8] - 2025-04-09

### Fixed

* Switch Hoymiles temporarily to the old/previous API

### Security

* Updated dependencies, fixes security advisory:
  * [RUSTSEC-2025-00022](https://rustsec.org/advisories/RUSTSEC-2025-0022)

## [0.3.7] - 2025-03-23

### Added

* Add Renovate config with recommended settings

### Changed

* Update the dependencies on `reqwest` and `thiserror`

### Fixed

* Fix typos in documentation and comments
* Fix some clippy issues

### Security

* Updated dependencies, fixes security advisories:
  * [RUSTSEC-2024-0421](https://rustsec.org/advisories/RUSTSEC-2024-0421)
  * [RUSTSEC-2025-0004](https://rustsec.org/advisories/RUSTSEC-2025-0004)
  * [RUSTSEC-2025-0009](https://rustsec.org/advisories/RUSTSEC-2025-0009)

## [0.3.6] - 2024-07-27

### Changed

* Switch to Forgejo Actions; add audit workflow
* Switch build dependency on `vergen` to `vergen-git2`

### Security
 
* Update dependencies; this fixes several security advisories:
  * [RUSTSEC-2024-0019](https://rustsec.org/advisories/RUSTSEC-2024-0019)
  * [RUSTSEC-2024-0332](https://rustsec.org/advisories/RUSTSEC-2024-0332)
  * [RUSTSEC-2024-0357](https://rustsec.org/advisories/RUSTSEC-2024-0357)

## [0.3.5] - 2024-02-27

### Fixed

* Fix clippy issue

## [0.3.4] - 2024-02-27

### Security

* Updated dependencies, fixes security advisories:
  * [RUSTSEC-2024-0003](https://rustsec.org/advisories/RUSTSEC-2024-0003)
  * [RUSTSEC-2023-0072](https://rustsec.org/advisories/RUSTSEC-2024-0072)

## [0.3.3] - 2023-11-03

### Security

* Update dependencies ([RUSTSEC-2020-0071](https://rustsec.org/advisories/RUSTSEC-2020-0071.html))

### Changed

* Switch to Rocket 0.5 RC4

## [0.3.2] - 2023-08-27

### Fixed

* Switch to Debian bookworm Docker image for runtime; fixes Docker image

## [0.3.1] - 2023-08-26

### Changed

* Fix and improve Gitea Actions workflow

### Security

* Update dependencies ([RUSTSEC-2023-0044](https://rustsec.org/advisories/RUSTSEC-2023-0044))

## [0.3.0] - 2023-04-15

### Added

* Implement backoff for login/update API call failures (#8)

### Changed

* Update dependencies
* Speed up Docker image builds by using sparse Cargo index for crates.io

### Fixed

* Fix login errors not being detected for My Autarco
* Fix missing build script/git repository during Docker image build

## [0.2.2] - 2023-03-22

### Added

* Implement error catchers for all endpoints (#5)
* Print the version on lift off (#6)
* Add `/version` endpoint to the API (#6)
* Add Gitea Actions workflow for cargo

### Fixed

* Fixed/tweaked documentation

### Security

* Update dependencies ([RUSTSEC-2023-0018](https://rustsec.org/advisories/RUSTSEC-2023-0018.html))

## [0.2.1] - 2023-01-16

### Changed

* Change poll interval for Hoymiles to 5 minutes
* Catch and raise error when Hoymiles API data responses cannot be deserialized
* Use stderr for error messages (and change prefix emoji)
* Use the `serde` crate via Rocket,; drop depend on the `serde` crate itself

### Fixed

* Also set the state class in HA sensors example
* Improve deserialization of Hoymiles API responses (#7)
* Prevent total energy reported decreasing for Hoymiles (#7)
* Set correct `last_updated` field in status report for Hoymiles (#7)
* Set cookie to configure Hoymiles API language to English (#7)
* Detect when Hoymiles (login/data) API response are not correct (#7)
* Small formatting, error message and documentation fixes

## [0.2.0] - 2023-01-13

### Added

* Add support for multiple services (#3)
* Add support for the Hoymiles service (#2)
* Add `Dockerfile` (and `.dockerignore`) for building Docker images
* Add `docker-compose-yml` for running using Docker Compose
* Add Debian packaging via cargo-deb (#4)
* Add documentation for how to use it with Home Assistant

### Changed

* Change the example port the webservice runs at to 2399
* Update documentation for Docker (Compose) support
* Split off a library crate
* Split off My Autarco support as a separate service

## [0.1.1] - 2023-01-08

Rename Autarco Scraper project to Solar Grabber.

[Unreleased]: https://git.luon.net/paul/solar-grabber/compare/v0.3.8...HEAD
[0.3.8]: https://git.luon.net/paul/solar-grabber/compare/v0.3.7...v0.3.8
[0.3.7]: https://git.luon.net/paul/solar-grabber/compare/v0.3.6...v0.3.7
[0.3.6]: https://git.luon.net/paul/solar-grabber/compare/v0.3.5...v0.3.6
[0.3.5]: https://git.luon.net/paul/solar-grabber/compare/v0.3.4...v0.3.5
[0.3.4]: https://git.luon.net/paul/solar-grabber/compare/v0.3.3...v0.3.4
[0.3.3]: https://git.luon.net/paul/solar-grabber/compare/v0.3.2...v0.3.3
[0.3.2]: https://git.luon.net/paul/solar-grabber/compare/v0.3.1...v0.3.2
[0.3.1]: https://git.luon.net/paul/solar-grabber/compare/v0.3.0...v0.3.1
[0.3.0]: https://git.luon.net/paul/solar-grabber/compare/v0.2.2...v0.3.0
[0.2.2]: https://git.luon.net/paul/solar-grabber/compare/v0.2.1...v0.2.2
[0.2.1]: https://git.luon.net/paul/solar-grabber/compare/v0.2.0...v0.2.1
[0.2.0]: https://git.luon.net/paul/solar-grabber/compare/v0.1.1...v0.2.0
[0.1.1]: https://git.luon.net/paul/solar-grabber/src/tag/v0.1.1
