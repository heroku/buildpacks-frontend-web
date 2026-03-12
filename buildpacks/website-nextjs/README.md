# Heroku Cloud Native Website (Next.js) Buildpack

* At build:
  * Detects `next` in the app's `package.json` dependencies.
  * Requires `heroku/nodejs` installation and build.
  * Configures `heroku/static-web-server` to deliver a clean-url, multi-page app from `next`'s output, including `404.html` for not found responses.

## Usage

Create an app with [Next.js](https://nextjs.org/):

```bash
npx create-next-app@latest
```

Configure the Next.js app to for [static exports](https://nextjs.org/docs/app/guides/static-exports). Set the output mode in `next.config.js`:

```javascript
const nextConfig = {
  output: 'export'
}
```

Then, to configure the buildpacks, create a [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/) containing:

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

Now, the app should be ready to build with `heroku/builder`:

```bash
pack build \
  --builder heroku/builder:24 \
  <APP_NAME>
```

See [Static Web Server](../static-web-server/README.md) for all capabilities and configuration options.
