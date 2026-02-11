# Heroku Cloud Native Website (Vite) Buildpack

* At build:
  * Detects `vite` in the app's `package.json` dependencies.
  * Requires `heroku/nodejs` installation and build.
  * Configures `heroku/static-web-server` for `vite`'s output.

## Usage

Create an app with [vite](https://vite.dev/guide/#scaffolding-your-first-vite-project):

```bash
npm create vite@latest
```

Now, the app should be ready to build, with Vite auto-detected by `heroku/builder`:

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
  id = "heroku/nodejs"
[[io.buildpacks.group]]
  id = "heroku/static-web-server"
[[io.buildpacks.group]]
  id = "heroku/website-vite"
```

See [Static Web Server](../static-web-server/README.md) for all capabilities and configuration options.
