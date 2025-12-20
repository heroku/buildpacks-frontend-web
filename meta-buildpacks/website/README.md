# Heroku Cloud Native Website Buildpack

Deploy a standard website (HTML/CSS/Javascript) using CNB.

* At build:
  * Detects `public/index.html`, or the configure [`root`](../../buildpacks/static-web-server/README.md#document-root) and [`index`](../../buildpacks/static-web-server/README.md#index-document), in the app source.
* At launch:
  * Performs [runtime app configuration](../../buildpacks/static-web-server/README.md#runtime-app-configuration).
  * [static-web-server](../../buildpacks/static-web-server/README.md) runs with config generated during build.

## Usage

In the app source repo, add the buildpack to [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/):

```toml
[[io.buildpacks.group]]
id = "heroku/website"
```

### Local

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
