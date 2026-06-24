# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Report the `cnb.static-web-server.config.runtime_config_enabled` metric with its actual boolean value, so a disabled runtime config emits `false` instead of being omitted.
- Report the `cnb.static-web-server.config.caddy_server_opts_basic_auth` metric with its actual boolean value, so disabled basic auth emits `false` instead of being omitted.
- Report the `cnb.static-web-server.config.caddy_server_opts_templates` metric with its actual boolean value, so disabled templates emits `false` instead of being omitted.
- Report the `cnb.static-web-server.config.caddy_server_opts_clean_urls` metric with its actual boolean value, so disabled clean URLs emits `false` instead of being omitted.
- Report the `cnb.static-web-server.config.caddy_server_opts_access_logs` metric with its actual boolean value, so disabled access logs emits `false` instead of being omitted.
- Report the `cnb.static-web-server.config.response_headers_enabled` metric on every build, emitting `false` when no response headers are configured instead of being omitted.
- Report the `cnb.static-web-server.config.caddy_server_opts_static_responses` metric on every build, emitting `false` when no static responses are configured instead of being omitted.
- Update Caddy web server version to 2.11.4.

## [3.3.1] - 2026-05-29

- Update Caddy web server version to 2.11.3.

## [3.3.0] - 2026-05-27

- No changes.

## [3.2.2] - 2026-04-21

- No changes.

## [3.2.1] - 2026-04-13

- No changes.

## [3.2.0] - 2026-03-05

- No changes.

## [3.1.0] - 2026-02-24

- No changes.

## [3.0.0] - 2026-02-13

- No changes.

## [2.2.0] - 2026-01-27

- No changes.

## [2.1.0] - 2026-01-05

- No changes.

## [2.0.0] - 2025-12-15

- No changes.

## [1.0.8] - 2025-09-08

- No changes.

## [1.0.7] - 2025-02-17

- No changes.

## [1.0.6] - 2024-12-19

- No changes.

## [1.0.5] - 2024-12-17

- No changes.

## [1.0.4] - 2024-12-12

- No changes.

## [1.0.3] - 2024-12-11

- No changes.

## [1.0.2] - 2024-11-08

- No changes.

## [0.1.1] - 2024-09-05

- No changes.

[unreleased]: https://github.com/heroku/buildpacks-frontend-web/compare/v3.3.1...HEAD
[3.3.1]: https://github.com/heroku/buildpacks-frontend-web/compare/v3.3.0...v3.3.1
[3.3.0]: https://github.com/heroku/buildpacks-frontend-web/compare/v3.2.2...v3.3.0
[3.2.2]: https://github.com/heroku/buildpacks-frontend-web/compare/v3.2.1...v3.2.2
[3.2.1]: https://github.com/heroku/buildpacks-frontend-web/compare/v3.2.0...v3.2.1
[3.2.0]: https://github.com/heroku/buildpacks-frontend-web/compare/v3.1.0...v3.2.0
[3.1.0]: https://github.com/heroku/buildpacks-frontend-web/compare/v3.0.0...v3.1.0
[3.0.0]: https://github.com/heroku/buildpacks-frontend-web/compare/v2.2.0...v3.0.0
[2.2.0]: https://github.com/heroku/buildpacks-frontend-web/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/heroku/buildpacks-frontend-web/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/heroku/buildpacks-frontend-web/compare/v1.0.8...v2.0.0
[1.0.8]: https://github.com/heroku/buildpacks-frontend-web/compare/v1.0.7...v1.0.8
[1.0.7]: https://github.com/heroku/buildpacks-frontend-web/compare/v1.0.6...v1.0.7
[1.0.6]: https://github.com/heroku/buildpacks-frontend-web/compare/v1.0.5...v1.0.6
[1.0.5]: https://github.com/heroku/buildpacks-frontend-web/compare/v1.0.4...v1.0.5
[1.0.4]: https://github.com/heroku/buildpacks-frontend-web/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/heroku/buildpacks-frontend-web/compare/v1.0.2...v1.0.3
[1.0.2]: https://github.com/heroku/buildpacks-frontend-web/compare/v0.1.1...v1.0.2
[0.1.1]: https://github.com/heroku/buildpacks-frontend-web/releases/tag/v0.1.1
