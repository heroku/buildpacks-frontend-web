# Heroku Cloud Native Buildpacks: Front-end Web

🚧 **This repo is experimental.** Use at your own risk. 🚧

This repository is the home of Heroku Cloud Native Buildpacks for Front-end Web apps, also known as: static websites, static web apps (SWA), single-page apps (SPA), and browser apps.

## Included Buildpacks

This repository contains multiple buildpacks:

### Composite Buildpacks

High-level buildpacks for zero-configuration deployment of specific static site technologies.

| ID                           | Name                                                          |
|------------------------------|---------------------------------------------------------------|
| `heroku/website`             | [Website](meta-buildpacks/website/README.md)                  |
| `heroku/website-nodejs`      | [Website/Node.js](meta-buildpacks/website-nodejs/README.md)   |

### Buildpacks

Lower-level buildpacks that provide specific capabilities. Typically require manual configuration.

| ID                           | Name                                                              |
|------------------------------|-------------------------------------------------------------------|
| `heroku/static-web-server`   | [Static Web Server](buildpacks/static-web-server/README.md)       |
| `heroku/website-ember`       | [Website (Ember.js)](buildpacks/website-ember/README.md)          |
| `heroku/website-public-html` | [Website (Public HTML)](buildpacks/website-public-html/README.md) |

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
