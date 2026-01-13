# Heroku Cloud Native Website (Vite) Buildpack

* At build:
  * Detects `vite` in the app's `package.json` dependencies.
  * Performs heroku/nodejs install and runs build script.
  * Configures heroku/static-web-server for the detected framework.
* At launch:
  * Performs [runtime app configuration](../../buildpacks/static-web-server/README.md#runtime-app-configuration).
  * [static-web-server](../../buildpacks/static-web-server/README.md) runs with config generated during build.

## Usage

Create an app with [vite](https://vite.dev/guide/#scaffolding-your-first-vite-project):

```bash
npm create vite@latest
```

Then, use [heroku/website-nodejs](../../meta-buildpacks/website-nodejs/README.md) meta-buildpack.
