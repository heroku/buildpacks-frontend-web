# Heroku Cloud Native Buildpacks: Front-end Web

This repository is the home of Heroku Cloud Native Buildpacks for Front-end Web apps, also known as: static websites, static web apps (SWA), single-page apps (SPA), and browser apps. These buildpacks build source code into application images that serve a website with minimal configuration.

> [!IMPORTANT]
> This repo is not yet functional, and is under active development.

## Included Buildpacks

This repository contains multiple buildpacks:

| ID                           | Name                        | Readme                                             |
|------------------------------|-----------------------------|----------------------------------------------------|
| `heroku/website`             | Website Composite Buildpack | [Readme](meta-buildpacks/website/README.md)        |
| `heroku/website-public-html` | Website (Public HTML)       | [Readme](buildpacks/website-public-html/README.md) |
| `heroku/static-web-server`   | Static Web Server           | [Readme](buildpacks/static-web-server/README.md)   |

## Configuration

In the app source code, create a [`project.toml`](https://buildpacks.io/docs/reference/config/project-descriptor/) for custom configuration.

### Build Command

*Default: (none)*

A command to execute during build, such as a JavaScript compiler/bundler.

```toml
[com.heroku.static-web-server]
build = "npm build"
```

Any dependencies to run this build command should be installed by an earlier buildpack, such as Node & npm for JavaScript.

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
