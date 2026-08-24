# WebAssembly Module to Inject Environment Variables as HTML Data Attributes

`@heroku/env-as-html-data-wasm` supports local development of front-end web applications by applying the same HTML runtime-configuration transformation as `heroku/static-web-server`, without running the CNB lifecycle.

The package uses the Rust `html5ever` implementation compiled to WebAssembly. It writes matching environment variables as HTML `data-*` attributes on the document's `<head>` element.

HTML is parsed and serialized during this process. Invalid HTML may be normalized and document formatting can change.

## Use

Install the package in a Node.js application:

```shell
npm install @heroku/env-as-html-data-wasm
```

Invoke it before starting a local development server or as part of a build command:

```shell
npx env-as-html-data
```

The command reads the current directory's `project.toml` and environment. For example:

```shell
PUBLIC_WEB_API_URL=https://localhost:3001 npx env-as-html-data
```

Or invoke it programmatically:

```javascript
const { injectEnvToHtmlFiles, transformHtml } = require('@heroku/env-as-html-data-wasm');

await injectEnvToHtmlFiles(process.env, process.cwd());

const html = transformHtml(
  '<html><head></head><body></body></html>',
  { PUBLIC_WEB_API_URL: 'https://localhost:3001' },
);
```

`transformHtml` is synchronous and does not read or write files. It returns the supplied HTML unchanged when the environment contains no `PUBLIC_WEB_` or `public_web_` variables.

## Configuration

Configure the target HTML files in `project.toml` using the same runtime-configuration settings as [`heroku/static-web-server`](../../buildpacks/static-web-server/README.md#runtime-app-configuration):

```toml
[com.heroku.static-web-server]
root = "public"
index = "index.html"

[com.heroku.static-web-server.runtime_config]
html_files = ["index.html", "subsection/**/*.html"]
```

Without configuration, the package rewrites `public/index.html`. `html_files` paths are relative to `root` and support `*` and `**` globs. Setting `runtime_config.enabled = false` skips rewriting.

## Runtime Variables

**Never use secrets in these environment variables.** Every matched value is written to public HTML and is visible to application users.

Only variables with the `PUBLIC_WEB_` or `public_web_` prefix are included. HTML attribute names are lowercased:

```shell
export PUBLIC_WEB_API_URL=https://api.example.com
export PUBLIC_WEB_RELEASE_VERSION=v42
export PORT=3000
```

```javascript
document.head.dataset.public_web_api_url;
document.head.dataset.public_web_release_version;
document.head.dataset.port; // undefined
```

## Development

This is an npm workspace in this repository. It requires Node.js 20 or later, Rust, the `wasm32-unknown-unknown` Rust target, and `wasm-pack` 0.15.0.

Install workspace dependencies from the repository root:

```shell
npm ci
```

Install the WebAssembly toolchain if needed:

```shell
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
```

Build the Node.js WebAssembly package:

```shell
npm run build --workspace @heroku/env-as-html-data-wasm
```

The generated Node.js binding and `.wasm` module are placed in `pkg/` and are intentionally not committed. Run the package tests and inspect publish contents with:

```shell
npm test --workspace @heroku/env-as-html-data-wasm
npm pack --dry-run --workspace @heroku/env-as-html-data-wasm
```

The repository CI job performs the same Node package verification, plus:

```shell
cargo check -p env_as_html_data_wasm --target wasm32-unknown-unknown --locked
```

## Release

This package is not yet wired into the buildpack release workflow. The existing reusable buildpack release integration publishes CNB artifacts only; it does not publish npm packages.

Before publishing this package, add a dedicated npm release workflow using npm trusted publishing and GitHub OIDC. The package must be manually published once and configured for trusted publishing before automated releases can use OIDC. Keep its versioning separate from the buildpack release until a deliberate version-sync policy is introduced.
