# Heroku Cloud Native Buildpacks: Front-end Web

This repository is the home of Heroku Cloud Native Buildpacks for Front-end Web apps, also known as: static websites, static web apps (SWA), single-page apps (SPA), and browser apps. These buildpacks build source code into application images that serve a website with minimal configuration.

## Included Buildpacks

This repository contains multiple buildpacks:

### Composite Buildpacks

High-level buildpacks for zero-configuration deployment of specific static site technologies.

| ID                           | Name                         | Readme                                             |
|------------------------------|------------------------------|----------------------------------------------------|
| `heroku/ember`               | Ember.js                     | [Readme](meta-buildpacks/ember/README.md)          |
| `heroku/website`             | Website                      | [Readme](meta-buildpacks/website/README.md)        |

### Buildpacks

Lower-level buildpacks that provide specific capabilities. Typically require manual configuration.

| ID                           | Name                         | Readme                                             |
|------------------------------|------------------------------|----------------------------------------------------|
| `heroku/static-web-server`   | Static Web Server            | [Readme](buildpacks/static-web-server/README.md)   |
| `heroku/website-ember`       | Website (Ember.js)           | [Readme](buildpacks/website-ember/README.md)       |
| `heroku/website-public-html` | Website (Public HTML)        | [Readme](buildpacks/website-public-html/README.md) |

## Configuration

In the app source code, create a [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/) for custom configuration.

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

This example sets a doc root & index, but any [configuration](#configuration) options are supported:

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

## Dev Notes

### Run Tests

```bash
cargo test -- --include-ignored
```

### Local Usage

```bash
cargo libcnb package

pack build my-cnb-website \
  --buildpack packaged/x86_64-unknown-linux-musl/debug/heroku_website \
  --builder heroku/builder:22 \
  --path buildpacks/website-public-html/tests/fixtures/public_html

docker run --env PORT=8888 -p 8888:8888 my-cnb-website
```
