# Heroku Cloud Native Static Web Server Buildpack

This buildpack implements www hosting support for a static web app.

* Defines [`project.toml` configuration](#configuration), `[com.heroku.static-web-server]`
* At build:
  * Installs a static web server (currently [Caddy](https://caddyserver.com/)).
  * Includes [heroku/release-phase buildpack](https://github.com/heroku/buildpacks-release-phase) to enable Release Phase Build & Static Artifacts.
  * [Inherits configuration](#inherited-configuration) from the Build Plan `[requires.metadata]` of other buildpacks.
  * Transforms the configuration into native configuration for the web server.
  * Optionally, runs a `build` command, such as `npm build` for minification & bundling of a Javascript app.
* At launch, the default `web` process:
  * Loads static artifacts for the release using [heroku/release-phase buildpack](https://github.com/heroku/buildpacks-release-phase)
  * Starts the web server listing on the `PORT`, using the server's native config generated during build.
  * Honors process signals for graceful shutdown.

## Configuration

In the app source code, create a [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/) for custom configuration.

### Release Build Command

*Default: (none)*

The command to generate static artifacts for a website, such as a JavaScript compiler/bundler. It is automatically executed during [Heroku Release Phase](https://devcenter.heroku.com/articles/release-phase), for changes to config vars, pipeline promotions, and rollbacks.

This command must write its output to the `static-artifacts/` directory (`/workspace/static-artifacts/` in the container). The generated `static-artifacts/` from each release are saved in an object store (AWS S3), separate from the container image itself, and loaded into `web` containers as they start-up.

Any dependencies to run this build command should be installed by an earlier buildpack, such as Node & npm engines for JavaScript.

```toml
[com.heroku.release-build]
command = ["sh"]
args = ["-c", "npm build"]
```

If the output is sent to a different directory, for example `dist/`, it should be copied to the expected location:

```toml
[com.heroku.release-build]
command = ["sh"]
args = ["-c", "npm run build && mkdir -p static-artifacts && cp -rL dist/* static-artifacts/"]
```

### Static Build Command

*Default: (none)*

This buildpack also supports a executing a build command during CNB Build process. The output of this command is saved in the container image, and will not be re-built during release, versus the [Release Build Command](#release-build-command)).

```toml
[com.heroku.static-web-server.build]
command = ["sh"]
args = ["-c", "npm build"]
```

This static build command does not have access to Heroku app config vars, but still can be configured using CNB Build variables in `project.toml`:

```toml
 [[io.buildpacks.build.env]]
 name = "API_URL"
 value = "https://test.example.com/api/v7"
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
