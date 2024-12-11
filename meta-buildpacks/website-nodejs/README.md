# Heroku Cloud Native Website/Node.js Buildpack

Deploy a static web app that requires Node.js for build.

* Detects specific frameworks in the app's `package.json` dependencies:
  * [Ember.js](../../buildpacks/website-ember/README.md)
* At build:
  * Creates Build Plan `[requires.metadata]` for Static Web Server, setting specific values to support the detected framwork:
    * the framework's root & index document
    * client-side routing
    * the framework's `build` command.
* At launch:
  * [static-web-server](../../buildpacks/static-web-server/README.md) runs with config generated during build.

## Usage

In the app source repo, add the buildpack to [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/):

```toml
[[io.buildpacks.group]]
id = "heroku/website-nodejs"
```

### Local

Build & run it locally:

```bash
# Build the container image
pack build <APP_NAME> \
  --buildpack heroku/website-nodejs \
  --builder heroku/builder:24 \
  --path <SOURCE_DIR>

# Execute Release Build
docker run --entrypoint release \
  --env STATIC_ARTIFACTS_URL=file://<WEBSITE_DIR>/static-artifacts \
  --env RELEASE_ID=v1 \
  --volume <WEBSITE_DIR>/static-artifacts:/workspace/static-artifacts \
  <APP_NAME>

# Launch Web Server
docker run  \
  --env STATIC_ARTIFACTS_URL=file://<WEBSITE_DIR>/static-artifacts \
  --env RELEASE_ID=v1 \
  --env PORT=8888 -p 8888:8888 \
  --volume <WEBSITE_DIR>/static-artifacts:/workspace/static-artifacts \
  <APP_NAME>
```

## Configuration

[All configuration](buildpacks/static-web-server/README.md#configuration) is documented with the Static Web Server.
