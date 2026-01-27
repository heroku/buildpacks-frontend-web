# Heroku Cloud Native Website/Node.js Buildpack

Deploy a static web app that requires Node.js for build.

* At build:
  * Detects specific frameworks in the app's `package.json` dependencies:
    * [Ember.js](../../buildpacks/website-ember/README.md)
    * [Vite](../../buildpacks/website-vite/README.md)
    * [Next.js](../../buildpacks/website-nextjs/README.md)
    * *More frameworks are planned, still to come.*
  * Performs heroku/nodejs install and runs build script.
  * Configures heroku/static-web-server for the detected framwork.
* At launch:
  * Performs [runtime app configuration](../../buildpacks/static-web-server/README.md#runtime-app-configuration).Z
  * [static-web-server](../../buildpacks/static-web-server/README.md) runs with config generated during build.

## Usage

In the app source repo, add the buildpack to [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/):

```toml
[[io.buildpacks.group]]
id = "heroku/website-nodejs"
```

### Local

Requires [pack](https://buildpacks.io/docs/for-platform-operators/how-to/integrate-ci/pack/) and [docker](https://docs.docker.com/engine/install/).

Build & run it locally:

```bash
# Build the container image
pack build <APP_NAME> \
  --builder heroku/builder:24 \
  --path <SOURCE_DIR>

# Launch Web Server
docker run  \
  --env PORT=8888 -p 8888:8888 \
  <APP_NAME>
```

## Configuration

[All configuration](../../buildpacks/static-web-server/README.md#usage) is documented with the Static Web Server.
