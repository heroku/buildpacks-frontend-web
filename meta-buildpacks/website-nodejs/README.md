# Heroku Cloud Native Website/Node.js Buildpack

Build & run a static web app that requires Node.js for build.

* Detects specific frameworks in the app's `package.json` dependencies:
  * [Ember.js](../../buildpacks/website-ember/README.md)
* At build:
  * Creates Build Plan `[requires.metadata]` for Static Web Server, setting specific values to support the detected framwork:
    * the framework's root & index document
    * client-side routing
    * the framework's `build` command.
* At launch:
  * [static-web-server](../../static-web-server/README.md) runs with config generated during build.
