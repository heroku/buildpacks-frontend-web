# Heroku Cloud Native Website Buildpack

Deploy a standard website (HTML/CSS/Javascript) using CNB.

* Detects `public/index.html` in the app source.
* At build:
  * Creates Build Plan `[requires.metadata]` for Static Web Server, defining the `root` and `index` values.

## Configuration

The detected website directory can be customized with [`root`](../../buildpacks/static-web-server/README.md#document-root) and [`index`](../../buildpacks/static-web-server/README.md#index-document) configurations in `project.toml`.

See [all configuration](../../buildpacks/static-web-server/README.md#configuration) implemented by Static Web Server.

## Usage

Build & run it locally:

```bash
cargo libcnb package

pack build <APP_NAME> \
  --buildpack packaged/x86_64-unknown-linux-musl/debug/heroku_website \
  --builder heroku/builder:22 \
  --path <WEBSITE_DIR>

docker run --env PORT=8888 -p 8888:8888 <APP_NAME>
```
