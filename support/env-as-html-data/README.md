# Javascript Module to Inject Environment Variables as HTML Data Attributes

*Supports local development of Front-end Web JavaScript apps, providing the same [runtime configuration strategy as the buildpacks/static web server](../../buildpacks/static-web-server/README.md#runtime-app-configuration), without requiring the local CNB `pack build` and `docker run` workflow for runtime configuration.*

This module injects the current environment variables as [HTML `data-*` global attributes](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/data-*) into the app's HTML files. These variables can be updated every time the app starts.

HTML files are parsed and serialized while updating the `<head>` element. This can normalize invalid HTML and reformat the document.

## Using this Module

The general strategy to use this module in a JavaScript web app is to invoke it as part of the build process or immediately before dev-server start-up.

Install to the JavaScript project:  
```shell
npm install @heroku/env-as-html-data@">= 2.1.0"
```

Configure the target HTML files in the app's `project.toml`, using the same settings as [`heroku/static-web-server`](../../buildpacks/static-web-server/README.md#runtime-app-configuration):

```toml
[com.heroku.static-web-server]
root = "public"
index = "index.html"

[com.heroku.static-web-server.runtime_config]
html_files = ["index.html", "subsection/index.html"]
```

### Rewrites HTML Files

By default, the module rewrites the configured index document, or `public/index.html` when no `project.toml` settings are present. Paths in `html_files` are relative to `root` and may include `*` or `**` glob patterns.

**Avoid committing or deploying the rewritten HTML documents.** This will result in build-time environment values being captured into the document. For developer tooling/scripts, exclude `env-as-html-data` from the production build command. For example, only execute `env-as-html-data` for `npm start` (the dev server), not `npm build` (the production app).

### Invoking env-to-html-data

Invoke it before starting a local development server or as part of a build command:

```shell
npx @heroku/env-as-html-data
```

Or invoke it programmatically:

```javascript
const { injectEnvToHtmlFiles } = require('@heroku/env-as-html-data');

await injectEnvToHtmlFiles(process.env, process.cwd());
```

## Using Runtime Environment Variables

**Do not set secret values into these environment variables.** They will be injected into the website, where anyone on the internet can see the values. As a precaution, only environment variables prefixed with `PUBLIC_WEB_` prefix will be exposed.

**Use uppercase `PUBLIC_WEB_` environment-variable names and access their HTML data attributes in lowercase.** Although environment variables are colloquially uppercased, the resulting HTML data attributes are set and accessed in lowercase because [they are case-insensitive XML names](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/data-*).

For example, if this app is started:

```shell
export PUBLIC_WEB_API_URL=https://api.example.com
export PUBLIC_WEB_RELEASE_VERSION=v42
export PORT=3000
npm start
```

When the app is loaded in the web browser's JavaScript environment, these can be accessed using the [HTML data attributes](https://developer.mozilla.org/en-US/docs/Web/HTML/How_to/Use_data_attributes):

```javascript
const head = document.head

// These contain the env vars' values
head.dataset.public_web_api_url
head.dataset.public_web_release_version

// PORT is not set, because it isn't prefixed with PUBLIC_WEB_
head.dataset.port == null
```

## Using Build-time Variables

Environment variables used to configure the build, such as Webpack configuration, should be accessed using the normal Node.js `process.env` object.

## Development

This is an npm workspace in this repository. It requires Node.js 20 or later, Rust, the `wasm32-unknown-unknown` Rust target, and `wasm-pack` 0.15.0.

Install workspace dependencies from the repository root:

```shell
npm install
```

Install the WebAssembly toolchain if needed:

```shell
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
```

Build the Node.js WebAssembly package:

```shell
npm run build --workspace @heroku/env-as-html-data
```

The generated Node.js binding and `.wasm` module are placed in `pkg/` and are intentionally not committed. Run the package tests and inspect publish contents with:

```shell
npm test --workspace @heroku/env-as-html-data
npm pack --dry-run --workspace @heroku/env-as-html-data
```

The repository CI job performs the same Node package verification, plus:

```shell
cargo check -p env_as_html_data_wasm --target wasm32-unknown-unknown --locked
```

## How does the runtime variable injection work?

When injection is invoked:
1. reads all `PUBLIC_WEB_*` environment variables
2. reads `project.toml` to determine the document root and target HTML files
3. rewrites the HTML files, injecting these env vars as `<head data-*>` attributes.

## Breaking Changes in v2.0

Version 2.0 of this module closely aligns its behavior with the implementation in [`heroku/static-web-server`](../../buildpacks/static-web-server/README.md#runtime-app-configuration).

+ reads its own configuration from `project.toml` (instead of `ENV_AS_HTML_DATA_` env vars)
+ reads env vars with `PUBLIC_WEB_` prefix (instead of `PUBLIC_`)
+ writes HTML Data attributes to `<head>` element (instead of `<body>`)
+ safely rewrites HTML files.

All of this ensures that v2 behavior matches the Runtime Configuration behavior of `heroku/static-web-server` CNB, supporting local app dev without needing to run the full `pack build` and `docker run` CNB lifecycle.

# Contributor Notes

## Release process

Releases are managed with [release-please](https://github.com/googleapis/release-please) and npm trusted publishing. Start the **Release env-as-html-data** GitHub Actions workflow from `main`. Use its `dry_run` input to create or update a release PR without publishing.

Release-please derives the next SemVer version from conventional commits that touch `support/env-as-html-data/`. Changes to the Rust implementation directories must include a corresponding package-visible change so they are eligible for an npm release:

- `common/env_as_html_data/`
- `common/env_as_html_data_wasm/`

After a non-dry run publishes the release PR's package version, merge the release PR. The push-triggered workflow creates the component-specific GitHub release and tag, `env-as-html-data-vX.Y.Z`.

The trusted publisher must be configured in npm for `@heroku/env-as-html-data`, repository `heroku/buildpacks-frontend-web`, and workflow `.github/workflows/release-env-as-html-data.yml`.
