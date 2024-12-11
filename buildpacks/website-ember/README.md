# Heroku Cloud Native Website (Ember.js) Buildpack

* Detects `ember-cli` in the app's `package.json` dependencies.
* At build:
  * Creates Build Plan `[requires.metadata]` for static-web-server, defining:
    * Ember's `dist` root
    * support for client-side routing
    * the framework's `build` command.
* At launch:
  * [static-web-server](../../buildpacks/static-web-server/README.md) runs with config generated during build.

## Usage

Create an app with [ember-cli](https://cli.emberjs.com/release/basic-use/):

```bash
ember new <APP_NAME>
```

Then, use [heroku/website-nodejs](../../meta-buildpacks/website-nodejs/) meta-buildpack.
