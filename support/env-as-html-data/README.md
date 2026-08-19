# Javascript Module to Inject Environment Variables as HTML Data Attributes

*Supports local development of Front-end Web JavaScript apps, providing the same [runtime configuration strategy as the buildpacks/static web server](../../buildpacks/static-web-server/README.md#runtime-app-configuration), without requiring the local CNB `pack build` and `docker run` workflow for runtime configuration.*

This module will inject the current environment variables as [HTML `data-*` global attributes](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/data-*) into the app's HTML files. These variables can be updated everytime the app starts.

## Using this Module

The general strategy to use this module in a JavaScript web app, is to invoke this module as part of the build process or immediately before dev server start-up.

Install to the JavaScript project:  
```shell
npm install @heroku/env-as-html-data@">= 2.0.0"
```

Configuration options (set as shell/environment variables):
+ `ENV_AS_HTML_DATA_DIR` (default `public`) the directory to search for HTML files to process.
+ `ENV_AS_HTML_DATA_FILE_EXT` (default `.html`) the file extension to match for files to process.

Execute HTML injection with `npx env-as-html-data`.

## Using Runtime Environment Variables

**Do not set secret values into these environment variables.** They will be injected into the website, where anyone on the internet can see the values. As a precaution, only environment variables prefixed with `PUBLIC_WEB_` prefix will be exposed.

**The variable names are case-insensitive, accessed as lowercase.** Although enviroment variables are colloquially uppercased, the resulting HTML Data Attributes are set & accessed lowercased, because [they are case-insensitive XML names](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/data-*).

For example, if this app is started:

```
export PUBLIC_WEB_API_URL=https://localhost:3001
export PUBLIC_WEB_RELEASE_VERSION=v42
export PORT=3000
npm start
```

When the app is loaded in the web browser's javascript environment, these can be accessed using the [HTML Data Attribtes](https://developer.mozilla.org/en-US/docs/Web/HTML/How_to/Use_data_attributes):

```javascript
const body = document.querySelector("body")

// These contain the env vars' values
body.dataset.public_web_api_url
body.dataset.public_web_release_version

// PORT is not set, because it isn't prefixed with PUBLIC_WEB_
body.dataset.port == null
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
2. updates each `public/*.html` file, writing these env vars as `<body data-*>` attributes
3. serves the `public/` directory as static files
4. the body data attributes are available within javascript & CSS running in the pages.
