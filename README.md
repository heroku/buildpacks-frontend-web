# Heroku Cloud Native Buildpacks: Front-end Web

This repository is the home of Heroku Cloud Native Buildpacks for Front-end Web apps, also known as: static websites, static web apps (SWA), single-page apps (SPA), and browser apps. These buildpacks build source code into application images that serve a website with minimal configuration.

## Included Buildpacks

This repository contains multiple buildpacks:

### Composite Buildpacks

High-level buildpacks for zero-configuration deployment of specific static site technologies.

| ID                           | Name                                          |
|------------------------------|-----------------------------------------------|
| `heroku/ember`               | [Ember.js](meta-buildpacks/ember/README.md)   |
| `heroku/website`             | [Website](meta-buildpacks/website/README.md)  |

### Buildpacks

Lower-level buildpacks that provide specific capabilities. Typically require manual configuration.

| ID                           | Name                                                              |
|------------------------------|-------------------------------------------------------------------|
| `heroku/static-web-server`   | [Static Web Server](buildpacks/static-web-server/README.md)       |
| `heroku/website-ember`       | [Website (Ember.js)](buildpacks/website-ember/README.md)          |
| `heroku/website-public-html` | [Website (Public HTML)](buildpacks/website-public-html/README.md) |

## Configuration

[All configuration](buildpacks/static-web-server/README.md#configuration) is implemented by Static Web Server.

## Dev Notes

### Run Tests

```bash
cargo test -- --include-ignored
```
