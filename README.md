# Heroku Cloud Native Buildpacks: Front-end Web

This repository is the home of Heroku Cloud Native Buildpacks for Front-end Web apps, also known as: static websites, static web apps (SWA), single-page apps (SPA), and browser apps. These buildpacks build source code into application images that serve a website with minimal configuration.

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

[Set the Front-end Web builder in `project.toml`](../../builder/README.md).

## Configuration

[All configuration](buildpacks/static-web-server/README.md#configuration) is implemented by Static Web Server.

## Dev Notes

### Run Tests

```bash
cargo test -- --include-ignored
```

### Releasing A New Version

[Action workflows](https://github.com/heroku/buildpacks-frontend-web/actions) are used to automate the release process:

1. Run **Prepare Buildpack Releases**.
1. Await completion of the preparation step.
1. ~~Run **Release Buildpacks**.~~ (This will not work until the repo is public, open-source. Until then, manually pack & release the builder)
