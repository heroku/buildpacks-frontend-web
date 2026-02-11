# Heroku Cloud Native Website (Public HTML) Buildpack

* At build:
  * Detects a `public/index.html` file.
  * Configures `heroku/static-web-server` to serve the website.

## Usage

Create a repo containing at least a `public/index.html` file.

Now, the app should be ready to build, with the public HTML auto-detected by `heroku/builder`:

```bash
pack build \
  --builder heroku/builder:24 \
  <APP_NAME>
```

To be explicit with the buildpacks required, create a [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/) containing:

```toml
[_]
schema-version = "0.2"

[[io.buildpacks.group]]
  id = "heroku/static-web-server"
[[io.buildpacks.group]]
  id = "heroku/website-public-html"
```

See [Static Web Server](../static-web-server/README.md) for all capabilities and configuration options.
