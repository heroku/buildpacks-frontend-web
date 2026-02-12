# Heroku Cloud Native Static Web Server Buildpack

This buildpack implements www hosting support for a static web app.

* Defines [`project.toml` configuration](#build-time-configuration), `[com.heroku.static-web-server]`
* At build:
  * Installs a static web server (currently [Caddy](https://caddyserver.com/)).
  * [Inherits configuration](#inherited-build-time-configuration) from the Build Plan `[requires.metadata]` of other buildpacks.
  * Transforms the configuration into native configuration for the web server.
  * Optionally, runs a [static build command](#static-build-command).
* At launch, the default `web` process:
  * Performs [runtime app configuration](#runtime-app-configuration), `PUBLIC_WEB_*` environment variables are written into `<head data-*>` attributes of the default HTML file in the document root.
  * Starts the web server listing on the `PORT`, using the server's native config generated during build.
  * Honors process signals for graceful shutdown.

## Usage

In the app source repo, add the buildpack to [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/):

```toml
[[io.buildpacks.group]]
id = "heroku/static-web-server"
```

## Runtime App Configuration

_Dynamic config used by the static web app at runtime, to support different app instances, such as a backend API URL that differs between Staging and Production._

These are set in the container's environment variables ([Heroku Config Vars](https://devcenter.heroku.com/articles/config-vars)) and during CNB launch, written into the default HTML document. To access runtime app config, the javascript app's source code must read configuration values from the global `document.head.dataset`, [HTML data-* attributes](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/data-*).

**Do not set secret values into these environment variables.** They will be injected into the website, where anyone on the internet can see the values. As a precaution, only environment variables prefixed with `PUBLIC_WEB_` prefix will be exposed.

**This feature parses and rewrites the HTML document.** If the document's HTML syntax is invalid, the parser ([Servo's html5ever](https://github.com/servo/html5ever)) will correct the document using the same heuristics as web browsers.

This Runtime App Configuration feature can be [disabled through Build-time Configuration](#runtime-configuration-enabled).

### Runtime Config Usage

*Default: runtime config is written into `public/index.html`, unless [document root](#document-root) or [index document](#index-document) are custom configured.*

For example, an app is started with the environment:

```
PUBLIC_WEB_API_URL=https://localhost:3001
PUBLIC_WEB_RELEASE_VERSION=v42
PORT=3000
HOME=/workspace
```

When the default HTML document is fetched by a web browser, loading the app, the `PUBLIC_WEB_*` vars can be accessed from javascript using the [HTML Data Attribtes](https://developer.mozilla.org/en-US/docs/Web/HTML/How_to/Use_data_attributes) via `document.head.dataset`:

```javascript
document.head.dataset.public_web_api_url
// → "https://api-staging.example.com"
document.head.dataset.public_web_release_version
// → "v42"

// Not exposed because not prefixed with PUBLIC_WEB_
document.head.dataset.port
// → null
document.head.dataset.home
// → null
```

**The variable names are case-insensitive, accessed as lowercase.** Although enviroment variables are colloquially uppercased, the resulting HTML Data Attributes are set & accessed lowercased, because [they are case-insensitive XML names](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/data-*).

For example, the `public_web_api_url` might be used for a `fetch()` call:

```javascript
// If the PUBLIC_WEB_API_URL variable is not set, default to the production API host.
const apiUrl = document.head.dataset.public_web_api_url || 'https://api.example.com';
const response = await fetch(apiUrl, {
  method: "POST",
  // …
});
```

Alternatively, default values can be preset in the HTML document's head element:

```html
<html>
<!-- If the PUBLIC_WEB_API_URL variable is set, this value in the document will be overwritten -->
<head data-public_web_api_url="https://api.example.com">
  <title>Example</title>
</head>
<body>
  <h1>Example</h1>
</body>
</html>  
```

Then, the javascript does not need a default value specified.

```javascript
const response = await fetch(document.head.dataset.public_web_api_url, {
  method: "POST",
  // …
});
```

## Build-time Configuration

_Static config that controls how the app is built, and how the web server delivers it._

This is set in the app source repo [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/) file and processed during CNB build. Rebuild is necessary to apply any changes.

### Static Build Command

*Default: (none)*

This buildpack supports a executing a build command during CNB Build process. The output of this command is saved in the container image.

For apps built with Node.js, execution of the build command is typically handled automatically by [heroku/nodejs CNB's build script hooks](https://github.com/heroku/buildpacks-nodejs/blob/main/README.md#build-script-hooks), and does not need to be configured here.

If your static web app is a static site generator built in a language other than JS, then you may need to configure the static site build command here. For example, [Hugo](https://gohugo.io) written in Go:

```toml
[com.heroku.static-web-server.build]
command = "sh"
args = ["-c", "hugo"]
```

This static build command does not have access to Heroku app config vars, but still can be configured using CNB Build variables in `project.toml`:

```toml
 [[io.buildpacks.build.env]]
 name = "HUGO_ENABLE_ROBOTS_TXT"
 value = "true"
```

When dependent on another language's compiled program like this, ensure that the app's buildpacks are ordered with `heroku/static-web-server` last, after the language buildpack.

```toml
[[io.buildpacks.group]]
id = "heroku/go"

[[io.buildpacks.group]]
id = "heroku/static-web-server"
```

### Runtime Configuration Enabled

*Default: true*

The [Runtime App Configuration](#runtime-app-configuration) feature may be disabled, such as when it is completely uneccesary or undesirable for a specific app.

```toml
[com.heroku.static-web-server.runtime_config]
enabled = false
```

### Runtime Configuration HTML Files

*Default: the set [index document](#index-document), or else its default `index.html`*

List of HTML files to rewrite with `data-*` attributes from [Runtime App Configuration](#runtime-app-configuration).

The files must be located within the [document root](#document-root), `public/` by default.

```toml
[com.heroku.static-web-server.runtime_config]
html_files = ["index.html", "subsection/index.html"]
```

`*` wildcards (globbing) are supported for websites that include many HTML files.

```toml
[com.heroku.static-web-server.runtime_config]
html_files = ["*.html"]
```

Recursive globbing is also supported, for websites that include many HTML files nested within subdirectories.

```toml
[com.heroku.static-web-server.runtime_config]
html_files = ["**/*.html"]
```

If a website contains an extremely large number (thousands) of globbed filenames, it's possible that the runtime configuration process could cause noticeable delays launching the web process.

### Document Root

*Default: `public`*

The directory in the app's source code to serve over HTTP.

```toml
[com.heroku.static-web-server]
root = "my_docroot"
```

### Index Document

*Default: `index.html`*

The file to respond with, when a request does not specify a document, such as requests to a bare hostame like `https://example.com`.

```toml
[com.heroku.static-web-server]
index = "main.html"
```

### Response Headers

*Default: (server's built-in headers)*

#### Global Headers

Respond with custom headers for any request path, the wildcard `*`.

```toml
[com.heroku.static-web-server.headers."*"]
X-Server = "hot stuff"
```

#### Path-matched Headers

Respond with custom headers. These match exactly against the request URL's path.

```toml
# The index page (index.html is not specified in the URL).
[com.heroku.static-web-server.headers."/"]
Cache-Control = "max-age=604800, stale-while-revalidate=86400, stale-if-error=86400"

# HTML pages.
[com.heroku.static-web-server.headers."/*.html"]
Cache-Control = "max-age=604800, stale-while-revalidate=86400, stale-if-error=86400"

# Contents of a subdirectory.
[com.heroku.static-web-server.headers."/images/*"]
Cache-Control = "max-age=31536000, immutable"

# Set multiple headers for a match.
[com.heroku.static-web-server.headers."/downloads/*"]
Cache-Control = "public, max-age=604800"
Content-Disposition = "attachment"
```

### Custom Errors

*Default: (server's built-in errors)*

#### 404 Not Found

Respond with a custom Not Found HTML page.

The path to this file is relative to the [Document Root](#document-root). The file should be inside the doc root.

```toml
[com.heroku.static-web-server.errors.404]
file_path = "error-404.html"
```

#### Replacement Status Code

Change the error response's HTTP status code.

Single-page app (SPA) client-side routing, where not found request URLs should respond with a single page (the app),

```toml
[com.heroku.static-web-server.errors.404]
file_path = "index.html"
status = 200
```

## Server-specific Configuration

Beyond pure static website delivery, some use-cases require dynamic server-side capabilities. This buildpack offers some server-specific configuration options, which tie the app to the specific server. Currently, only one web server is implemented: [Caddy](https://caddyserver.com).

### Server-specific config: Caddy

#### Caddy: Access Logs

*Default: not enabled*

Per-request access logs may be enabled, sending them to stdout. These are normally disabled, because the Heroku router already emits request events to the app log. These access logs may be beneficial for other use-cases, running locally or on other hosts.

```toml
[com.heroku.static-web-server.caddy_server_opts.access_logs]
enabled = true
```

[Caddy's log sampling](https://caddyserver.com/docs/json/logging/logs/sampling/) may be configured as well, to reduce logging load on a high traffic server.

```toml
[com.heroku.static-web-server.caddy_server_opts.access_logs]
enabled = true
sampling_interval = 60_000_000_000 # sixty-seconds
sampling_first = 1000
sampling_thereafter = 1000
```

#### Caddy: Clean URLs

*Default: not enabled*

Support for pretty, extensionless URLs, leaving `.html` off of the request path.

```toml
[com.heroku.static-web-server.caddy_server_opts.clean_urls]
enabled = true
```

For example, a request to `example.com/support` will be tried in the document root:
1. the literal path, `support`
2. the path with HTML extension, `support.html`
3. the path as a directory, `support/`

#### Caddy: Basic Authorization

*Default: not enabled*

Password protect all requests to the web server using [HTTP Basic Authorization](https://caddyserver.com/docs/modules/http.authentication.providers.http_basic).

```toml
[com.heroku.static-web-server.caddy_server_opts]
basic_auth = true
```

##### Caddy: Basic Auth: Required Env Vars

+ `WEB_BASIC_AUTH_USERNAME` any name you wish, for example `visitor`
+ `WEB_BASIC_AUTH_PASSWORD_BCRYPT` see [Generating hashed password](#caddy-generating-hashed-passwords)

For example, to set username `visitor` and password `geniuspass`:

```bash
# As a local shell configuration
export WEB_BASIC_AUTH_USERNAME=visitor
export WEB_BASIC_AUTH_PASSWORD_BCRYPT="$(htpasswd -bnBC 10 "" geniuspass | tr -d ':\n')"

# As a Heroku App config
heroku config:set \
  WEB_BASIC_AUTH_USERNAME=visitor \
  WEB_BASIC_AUTH_PASSWORD_BCRYPT="$(htpasswd -bnBC 10 "" geniuspass | tr -d ':\n')"
```

Without these env vars, the server will crash with an error:

> provision http.authentication.providers.http_basic: account 0: username and password are required

##### Caddy: Basic Auth: Disable at Runtime

`WEB_BASIC_AUTH_DISABLED=true`: after Basic Auth has been enabled at build-time using `basic_auth = true`, you may need to disable it later, at runtime, such as when a password-protected staging site is promoted to production.

```bash
# As a local shell configuration
export WEB_BASIC_AUTH_DISABLED=true

# As a Heroku App config
heroku config:set WEB_BASIC_AUTH_DISABLED=true
```

As long as `basic_auth = true`, the required env vars `WEB_BASIC_AUTH_USERNAME` and `WEB_BASIC_AUTH_PASSWORD_BCRYPT` are still required to run the server.

##### Caddy: Generating hashed passwords

Install `htpasswd`:
```
apt-get install apache2-utils     # Debian/Ubuntu
yum install httpd-tools           # RHEL/CentOS
brew install httpd                # macOS
```

Use `htpassword`, for example:

```bash
htpasswd -bnBC 10 "" password | tr -d ':\n'

# Explanation:
# -b: batch mode (password on command line)
# -n: display on stdout instead of updating file
# -B: use bcrypt
# -C 10: cost factor of 10
# "": empty username (we just want the hash)
# tr -d ':\n': removes the colon and newline
```

#### Caddy: Templates

*Default: false*

Enables [Caddy's server-side template rendering](https://caddyserver.com/docs/json/apps/http/servers/routes/handle/templates/), to support per-request dynamic values.

To avoid stale content being displayed in browsers and served through CDNs, dynamic content may require different cache control headers than static files.

```toml
[com.heroku.static-web-server.caddy_server_opts]
templates = true
```

#### Caddy: Nonces for Content-Security-Policy

*Requires: [Templates](#caddy-templates) enabled*

Use [CSP nonces](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy#nonce-nonce_value) by way of [template tags](https://caddyserver.com/docs/json/apps/http/servers/routes/handle/templates/) in HTML files. In an HTML file where inline scripts should be allowed:

1. Generate a nonce with [`uuidv4`](https://masterminds.github.io/sprig/uuid.html)
2. Declare the nonce in a CSP header
3. Set the nonce on script element `nonce` attributes.

For example:

```html
{{ $nonce := uuidv4 }}
{{ .RespHeader.Add "Content-Security-Policy" (print "nonce-" $nonce) }}

<!DOCTYPE html>
<html lang="en">

<head>
  <script nonce="{{ $nonce }}">alert('Load me with a strict CSP')</script>
</head>

</html>
```

## Inherited Build-time Configuration

Other buildpacks can return a [Build Plan](https://github.com/buildpacks/spec/blob/main/buildpack.md#build-plan-toml) from `detect` for Static Web Server configuration.

Configuration defined in an app's `project.toml` takes precedence over this inherited Build Plan configuration.

This example sets `root` & `index` in the build plan, using supported [configuration](#configuration) options:

```toml
[[requires]]
name = "static-web-server"

[requires.metadata]
root = "wwwroot"
index = "index.htm"
```

Example using [libcnb.rs](https://github.com/heroku/libcnb.rs):

```rust
fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
    let mut static_web_server_req = Require::new("static-web-server");
    let _ = static_web_server_req.metadata(toml! {
        root = "wwwroot"
        index = "index.htm"
    });
    let plan_builder = BuildPlanBuilder::new()
        .requires(static_web_server_req);

    DetectResultBuilder::pass()
        .build_plan(plan_builder.build())
        .build()
}
```

## Launching the Server

*Requires [pack](https://buildpacks.io/docs/for-platform-operators/how-to/integrate-ci/pack/) and [docker](https://docs.docker.com/engine/install/).*

Build and run the server container image locally, or on any OCI-compatible host.

```bash
# Build the container image
pack build <APP_NAME> \
  --builder heroku/builder:24 \
  --path <SOURCE_DIR>

# Launch Web Server
docker run \
  --env PORT=8888 -p 8888:8888 \
  <APP_NAME>

# Interactively explore the container from a shell
docker run \
  -it --entrypoint bash \
  <APP_NAME>
```
