# Heroku Cloud Native Website (create-react-app) Buildpack

* At build:
  * Detects `react-scripts` in the app's `package.json` dependencies.
  * Requires `heroku/nodejs` installation and build.
  * Configures `heroku/static-web-server` for `next`'s default output to `dist/`.

## Legacy Notice

In February 2025, [create-react-app was sunset](https://react.dev/blog/2025/02/14/sunsetting-create-react-app), to receive no more updates. This buildpack's support for create-react-app is provided to support existing apps developed with this tooling.

Please, consider migrating your legacy create-react-app project to [Vite](https://www.robinwieruch.de/vite-create-react-app/), for the best web development experience.

**For new apps,** use React as part of a more comprehensive app framework, like [Vite](../website-vite/README.md) or [Next.js](../website-nextjs/).

## Usage

Create an app with [create-react-app](https://create-react-app.dev):

```bash
npx create-react-app my-app
```

To set the buildpacks required, create a [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/) containing:

```toml
[_]
schema-version = "0.2"

[[io.buildpacks.group]]
  id = "heroku/nodejs"
[[io.buildpacks.group]]
  id = "heroku/static-web-server"
[[io.buildpacks.group]]
  id = "heroku/website-cra"
```

See [Static Web Server](../static-web-server/README.md) for all capabilities and configuration options.
