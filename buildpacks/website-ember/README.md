# Heroku Cloud Native Website (Ember.js) Buildpack

* At build:
  * Detects `ember-cli` in the app's `package.json` dependencies.
  * Requires `heroku/nodejs` installation and build.
  * Configures `heroku/static-web-server` for `ember-cli`'s output.

## Usage

Generate an app with [ember-cli](https://cli.emberjs.com/release/basic-use/):

```bash
ember new <APP_NAME>
```

Now, the app should be ready to build, with Ember auto-detected by `heroku/builder`:

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
  id = "heroku/website-ember"
```

See [Static Web Server](../static-web-server/README.md) for all capabilities and configuration options.
