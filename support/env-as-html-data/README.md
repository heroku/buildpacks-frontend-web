# Javascript Module to Inject Environment Variables as HTML Data Attributes

*Supports local development of Front-end Web JavaScript apps, providing the same [runtime configuration strategy as the buildpacks/static web server](../../buildpacks/static-web-server/README.md#runtime-app-configuration), without requiring the local CNB `pack build` and `docker run` workflow for runtime configuration.*

This module injects the current environment variables as [HTML `data-*` global attributes](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/data-*) into the app's HTML files. These variables can be updated every time the app starts.

HTML files are parsed and serialized while updating the `<head>` element. This can normalize invalid HTML and reformat the document.

## Using this Module

The general strategy to use this module in a JavaScript web app is to invoke it as part of the build process or immediately before dev-server start-up.

Install to the JavaScript project:  
```shell
npm install @heroku/env-as-html-data@">= 2.0.0"
```

Configure the target HTML files in the app's `project.toml`, using the same settings as [`heroku/static-web-server`](../../buildpacks/static-web-server/README.md#runtime-app-configuration):

```toml
[com.heroku.static-web-server]
root = "public"
index = "index.html"

[com.heroku.static-web-server.runtime_config]
html_files = ["index.html", "subsection/index.html"]
```

By default, the module rewrites the configured index document, or `public/index.html` when no `project.toml` settings are present. Paths in `html_files` are relative to `root` and may include `*` or `**` glob patterns.

### Invoking env-to-html-data

Invoke via CLI/shell with `npx @heroku/env-as-html-data`.

Or, import the module to invoke programmatically:
```javascript
const { injectEnvToHtmlFiles } = require('@heroku/env-to-html-data');
await injectEnvToHtmlFiles();
```

## Using Runtime Environment Variables

**Do not set secret values into these environment variables.** They will be injected into the website, where anyone on the internet can see the values. As a precaution, only environment variables prefixed with `PUBLIC_WEB_` prefix will be exposed.

**Use uppercase `PUBLIC_WEB_` environment-variable names and access their HTML data attributes in lowercase.** Although environment variables are colloquially uppercased, the resulting HTML data attributes are set and accessed in lowercase because [they are case-insensitive XML names](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/data-*).

For example, if this app is started:

```
export PUBLIC_WEB_API_URL=https://localhost:3001
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

From the repository root, install the workspace dependencies:

```bash
npm install
```

Run this workspace's Mocha test suite:

```bash
npm test --workspace @heroku/env-as-html-data
```

## How does the runtime variable injection work?

When this module runs during app start-up, it:
1. reads all `PUBLIC_WEB_*` environment variables
2. reads `project.toml` to determine the document root and target HTML files
3. updates the selected HTML files, writing these env vars as `<head data-*>` attributes
4. leaves serving static files to the application's web server
5. makes the head data attributes available within JavaScript and CSS running in the pages.

## Breaking Changes in v2.0

Version 2.0.0 of this module closely aligns its behavior with the implementation in [`heroku/static-web-server`](../../buildpacks/static-web-server/README.md#runtime-app-configuration).

+ reads its own configuration from `project.toml` (instead of `ENV_AS_HTML_DATA_` env vars)
+ reads env vars with `PUBLIC_WEB_` prefix (instead of `PUBLIC_`)
+ writes HTML Data attributes to `<head>` element (instead of `<body>`)
+ safely rewrites HTML files.

All of this ensures that v2 behavior matches the Runtime Configuration behavior of `heroku/static-web-server` CNB, supporting local app dev without needing to run the full `pack build` and `docker run` CNB lifecycle.
