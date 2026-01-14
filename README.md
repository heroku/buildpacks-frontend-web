# Heroku Cloud Native Buildpacks: Front-end Web

🚧 **This repo is experimental.** Use at your own risk. 🚧

This repository is the home of Heroku Cloud Native Buildpacks for Front-end Web apps, also known as: static websites, static web apps (SWA), single-page apps (SPA), and browser apps.

## Included Buildpacks

This repository contains multiple buildpacks:

### Composite Buildpacks

High-level buildpacks for zero-configuration deployment of specific static site technologies.

| ID                           | Name                                                          | Detects                |
|------------------------------|---------------------------------------------------------------|------------------------|
| `heroku/website`             | [Website](meta-buildpacks/website/README.md)                  | Public HTML            |
| `heroku/website-nodejs`      | [Website/Node.js](meta-buildpacks/website-nodejs/README.md)   | Ember.js, more to come |

### Component Buildpacks

Buildpacks that provide a specific server-side component.

| ID                           | Name                                                              | Provides |
|------------------------------|-------------------------------------------------------------------|----------|
| `heroku/static-web-server`   | [Static Web Server](buildpacks/static-web-server/README.md)       | Web Server supporting build and runtime configuration, as well as configuration inheritance from other buildpacks |

### Framework Buildpacks

Lower-level buildpacks that detect specific source layouts, frameworks, or tools, to automate configuration of build process and heroku/static-web-server.

| ID                           | Name                                                              | Provides                            |
|------------------------------|-------------------------------------------------------------------|-------------------------------------|
| `heroku/website-ember`       | [Website (Ember.js)](buildpacks/website-ember/README.md)          | auto-detect for ember-cli           |
| `heroku/website-nextjs`      | [Website (Next.js)](buildpacks/website-nextjs/README.md)          | auto-detect for Next.js             |
| `heroku/website-public-html` | [Website (Public HTML)](buildpacks/website-public-html/README.md) | auto-detect for `public/index.html` |
| `heroku/website-vite`        | [Website (Vite)](buildpacks/website-vite/README.md)               | auto-detect for vite                |
| More frameworks to come…     |                                                                   |                                     |

To implement support for additional frameworks, start from one these framework buildpacks as a template, and then combine it with other buildpacks such as heroku/nodejs and heroku/static-web-server in a [meta-buildpack](meta-buildpacks/).

## Usage

See the individual buildpack documentation, linked in the above table.

## Dev Notes

### Run Tests

```bash
cargo test -- --include-ignored
```

### Updating Node.js

These buildpacks include the heroku/nodejs buildpack, packaged at a specific version. To update the Node.js buildpack:

1. check the [Docker Hub listing](https://hub.docker.com/r/heroku/buildpack-nodejs/tags) for the latest version
1. update the version specified in the website-nodejs [buildpack](meta-buildpacks/website-nodejs/buildpack.toml) & [package](meta-buildpacks/website-nodejs/package.toml) specifications
1. ensure integration tests still pass, see [Run Tests](#run-tests)

### Releasing A New Version

[Action workflows](https://github.com/heroku/buildpacks-frontend-web/actions) are used to automate the release process:

1. Run **Prepare Buildpack Releases**.
1. Await completion of the preparation step.
1. Run **Release Buildpacks**.
