# Heroku Cloud Native Website (Ember.js) Buildpack

Use with [heroku/website-nodejs](../../meta-buildpacks/website-nodejs/) meta-buildpack.

* Detects `ember-cli` in the app's `package.json` dependencies.
* At build:
  * Creates Build Plan `[requires.metadata]` for static-web-server, defining:
    * Ember's `dist` root
    * support for client-side routing
    * the framework's `build` command.
* At launch:
  * [static-web-server](../static-web-server/README.md) runs with config generated during build.

## Configuration

[All configuration](../../buildpacks/static-web-server/README.md#configuration) is implemented by Static Web Server.

## Usage

Create an app with [ember-cli](https://cli.emberjs.com/release/basic-use/):

```bash
ember new <APP_NAME>
```

### Heroku Fir

Set the [Front-end Web builder](../../builder/README.md) in `project.toml`.

### Local

And then build & run it locally:

```bash
cargo libcnb package

pack build <APP_NAME> \
  --builder ghcr.io/heroku/builder-test-public:frontend-web-builder-0.1.1_linux-arm64 \
  --path <APP_NAME>

docker run --env PORT=8888 -p 8888:8888 <APP_NAME>
```
