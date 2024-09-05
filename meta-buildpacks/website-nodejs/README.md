# Heroku Cloud Native Website/Node.js Buildpack

Build & run a static web app that requires Node.js for build.

* Detects specific frameworks in the app's `package.json` dependencies.
* At build:
  * Creates Build Plan `[requires.metadata]` for Static Web Server, setting specific values to support the detected framwork:
    * the framework's root & index document
    * client-side routing
    * the framework's `build` command.

## Supported Frameworks

See the component buildpack for framework-specific information:

* [Ember.js](../../buildpacks/website-ember/README.md)
