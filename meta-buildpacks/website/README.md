# Heroku Cloud Native Website Buildpack

Deploy a standard website (HTML/CSS/Javascript) using CNB.

* Detects `public/index.html` in the app source.
* At build:
  * Creates Build Plan `[requires.metadata]` for Static Web Server, defining the `root` and `index` values.
* At launch:
  * [static-web-server](../../buildpacks/static-web-server/README.md) runs with config generated during build.

## Usage

### Heroku Fir

In the app source repo, add the buildpack to [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/):

```toml
[[io.buildpacks.group]]
id = "heroku/website"
```

### Local

Build & run it locally:

```bash
pack build <APP_NAME> \
  --buildpack heroku/website \
  --builder heroku/builder:24 \
  --path <SOURCE_DIR>

docker run --env PORT=8888 -p 8888:8888 <APP_NAME>
```

## Configuration

The detected website directory can be customized with [`root`](../../buildpacks/static-web-server/README.md#document-root) and [`index`](../../buildpacks/static-web-server/README.md#index-document) configurations in `project.toml`.

[All configuration](buildpacks/static-web-server/README.md#configuration) is documented with the Static Web Server.
