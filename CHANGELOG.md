# Changelog

All notable changes to Solar Grabber will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://git.luon.net/paul/solar-grabber/compare/v0.2.0...HEAD
[0.2.0]: https://git.luon.net/paul/solar-grabber/compare/v0.1.1...v0.2.0
[0.1.1]: https://git.luon.net/paul/solar-grabber/src/tag/v0.1.1
