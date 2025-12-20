# Heroku Cloud Native Website (Ember.js) Buildpack

* At build:
  * Detects `ember-cli` in the app's `package.json` dependencies.
  * Performs heroku/nodejs install and runs build script.
  * Configures heroku/static-web-server for the detected framework.
* At launch:
  * Performs [runtime app configuration](../../buildpacks/static-web-server/README.md#runtime-app-configuration).
  * [static-web-server](../../buildpacks/static-web-server/README.md) runs with config generated during build.

## Usage

Create an app with [ember-cli](https://cli.emberjs.com/release/basic-use/):

```bash
ember new <APP_NAME>
```

Then, use [heroku/website-nodejs](../../meta-buildpacks/website-nodejs/README.md) meta-buildpack.
