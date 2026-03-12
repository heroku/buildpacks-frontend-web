# Heroku Cloud Native Website (Public HTML) Buildpack

* At build:
  * Detects a `public/index.html` file.
  * Configures `heroku/static-web-server` to deliver the website in `public/`.

## Usage

Create a repo containing at least a `public/index.html` file.

Then, to configure the buildpacks required, create a [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/) containing:

```toml
[_]
schema-version = "0.2"

[[io.buildpacks.group]]
  id = "heroku/static-web-server"
[[io.buildpacks.group]]
  id = "heroku/website-public-html"
```

Now, the app should be ready to build with `heroku/builder`:

```bash
pack build \
  --builder heroku/builder:24 \
  <APP_NAME>
```

See [Static Web Server](../static-web-server/README.md) for all capabilities and configuration options.
