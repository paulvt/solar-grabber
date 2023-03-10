# Changelog

All notable changes to Solar Grabber will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://git.luon.net/paul/solar-grabber/compare/v0.2.1...HEAD
[0.2.1]: https://git.luon.net/paul/solar-grabber/compare/v0.2.0...v0.2.1
[0.2.0]: https://git.luon.net/paul/solar-grabber/compare/v0.1.1...v0.2.0
[0.1.1]: https://git.luon.net/paul/solar-grabber/src/tag/v0.1.1
