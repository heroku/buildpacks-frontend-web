# Heroku Cloud Native Ember Buildpack

Build & run an Ember.js static web app.

* Detects `ember-cli` in the app's `package.json` dependencies.
* At build:
  * Creates Build Plan `[requires.metadata]` for Static Web Server, defining:
    * Ember's `dist` root
    * support for client-side routing
    * the framework's `build` command.

## Configuration

[All configuration](../../buildpacks/static-web-server/README.md#configuration) is implemented by Static Web Server.

## Usage

Create an app with [ember-cli](https://cli.emberjs.com/release/basic-use/):

```bash
ember new <APP_NAME>
```

And then build & run it locally:

```bash
cargo libcnb package

pack build <APP_NAME> \
  --buildpack packaged/x86_64-unknown-linux-musl/debug/heroku_ember \
  --builder heroku/builder:22 \
  --path <APP_NAME>

docker run --env PORT=8888 -p 8888:8888 <APP_NAME>
```
