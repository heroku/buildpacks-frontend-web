# Heroku Cloud Native Website (Next.js) Buildpack

* At build:
  * Detects `next` in the app's `package.json` dependencies.
  * Requires `heroku/nodejs` installation and build.
  * Configures `heroku/static-web-server` for `next`'s output.

## Usage

Create an app with [Next.js](https://nextjs.org/):

```bash
npx create-next-app@latest
```

Now, the app should be ready to build, with Next.js auto-detected by `heroku/builder`:

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
  id = "heroku/website-nextjs"
```

See [Static Web Server](../static-web-server/README.md) for all capabilities and configuration options.
