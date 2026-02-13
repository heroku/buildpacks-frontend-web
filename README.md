# Heroku Cloud Native Buildpacks: Front-end Web

This repository is the home of Heroku Cloud Native Buildpacks for Front-end Web apps, also known as: websites, static web apps (SWA), single-page apps (SPA), progressive web apps (PWA), and browser apps.

## What It Is

These Front-end Web buildpacks produce an OCI (container image) that launches a web server to host a directory of HTML, CSS, and Javascript files. The build process can run Javascript or other programs to produce the website, or simply read a directory of files as the web root. All front-end frameworks are supported, while some popular frameworks are optimized with preset web server configuration.

The result is a static website:
* high-efficiency, high-performance server
* secure by default with minimal, purpose-built features
* separates front-end development (web UIs) from server-side programming (backend APIs).

### What It Is Not

This static web server does not run custom code on the server. If an app requires a custom server-side web process, then use the appropriate [language-specific CNBs](https://github.com/heroku/buildpacks).

## Included Buildpacks

This repository contains multiple buildpacks:

### Component Buildpacks

Buildpacks that provide a specific server-side component.

| ID                           | Name                                                              | Provides |
|------------------------------|-------------------------------------------------------------------|----------|
| `heroku/static-web-server`   | [Static Web Server](buildpacks/static-web-server/README.md)       | Web Server supporting build and runtime configuration, as well as configuration inheritance from other buildpacks |

### Framework Buildpacks

Lower-level buildpacks that detect specific source layouts, frameworks, or tools, to automate configuration of build process and `heroku/static-web-server`.

| ID                           | Name                                                              | Provides                            |
|------------------------------|-------------------------------------------------------------------|-------------------------------------|
| `heroku/website-ember`       | [Website (Ember.js)](buildpacks/website-ember/README.md)          | auto-detect for [Ember.js](https://cli.emberjs.com) |
| `heroku/website-nextjs`      | [Website (Next.js)](buildpacks/website-nextjs/README.md)          | auto-detect for [Next.js](https://nextjs.org) ([static exports](https://nextjs.org/docs/app/guides/static-exports)) |
| `heroku/website-public-html` | [Website (Public HTML)](buildpacks/website-public-html/README.md) | auto-detect for `public/index.html` |
| `heroku/website-vite`        | [Website (Vite)](buildpacks/website-vite/README.md)               | auto-detect for [Vite](https://vite.dev) |

## Usage

[Configure the Static Web Server](buildpacks/static-web-server/README.md) as the last buildpack for your app, along with other language buildpacks the build might require.

The framework buildpacks are optional. They automate detection of static web apps in source code when using [`heroku/builder`](https://github.com/heroku/cnb-builder-images), such as when building a CNB app on Heroku, but are not required when manually configuring buildpacks.

## Dev Notes

### Run Tests

```bash
cargo test -- --include-ignored
```

### Releasing A New Version

[Action workflows](https://github.com/heroku/buildpacks-frontend-web/actions) are used to automate the release process:

1. Run **Prepare Buildpack Releases**.
1. Await completion of the preparation step.
1. Run **Release Buildpacks**.
