# Heroku Cloud Native Static Web Server Buildpack

This buildpack implements www hosting support for a static web app.

* Defines [`project.toml` configuration](#configuration), `[com.heroku.static-web-server]`
* At build:
  * Installs a static web server (currently [Caddy](https://caddyserver.com/)).
  * [Inherits configuration](#inherited-configuration) from the Build Plan `[requires.metadata]` of other buildpacks.
  * Transforms the configuration into native configuration for the web server.
  * Optionally, runs a `build` command, such as `npm build` for minification & bundling of a Javascript app.
* At launch:
  * Starts the web server listing on the `PORT`, using the server's native config generated during build.
  * Honors process signals for graceful shutdown.

## Configuration

In the app source code, create a [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/) for custom configuration.

### Build Variables

Set CNB build environment:

```toml
 [[io.buildpacks.build.env]]
 name = "API_URL"
 value = "https://test.example.com/api/v7"

 [[io.buildpacks.build.env]]
 name = "CHECK_HELLO"
 value = "true"
```

🚧  Build env is separate from runtime env (Heroku config vars), when an app's processes are running.

🤐  **Do not set secrets into website code or source code repo!**

### Build Command

*Default: (none)*

A command to execute during CNB build, such as a JavaScript compiler/bundler.

For typical build tools, execute the shell `sh` with a command `-c` argument containing the build command:

```toml
[com.heroku.static-web-server]
command = ["sh"]
args = ["-c", "npm build"]
```

Any dependencies to run this build command should be installed by an earlier buildpack, such as Node & npm engines for JavaScript.

### Release Build Command

*Default: (none)*

> [!IMPORTANT]
> Release Phase capabilities are not yet supported. See [RFC: `heroku/release-phase` CNB](https://salesforce.quip.com/qViZA7facMoT).

A build command to execute during Heroku Release Phase.

```toml
[com.heroku.release-build]
command = ["sh"]
args = ["-c", "npm build"]
```

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

## Inherited Configuration

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
