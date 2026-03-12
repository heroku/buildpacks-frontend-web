# Heroku Cloud Native Website (Vite) Buildpack

* At build:
  * Detects `vite` in the app's `package.json` dependencies.
  * Requires `heroku/nodejs` installation and build.
  * Configures `heroku/static-web-server` to deliver a single-page app from `vite`'s output.

## Usage

Create an app with [vite](https://vite.dev/guide/#scaffolding-your-first-vite-project):

```bash
npm create vite@latest
```

Then, to configure the buildpacks required, create a [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/) containing:

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

Now, the app should be ready to build with `heroku/builder`:

```bash
pack build \
  --builder heroku/builder:24 \
  <APP_NAME>
```

*Vite supports many frameworks and configuration options. Depending on these individual choices, additional static web server configuration may be required to support multi-page sites, error handling, and other customizations.*

See [Static Web Server](../static-web-server/README.md) for all capabilities and configuration options.
