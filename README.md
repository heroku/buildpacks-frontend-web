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

### Document Root

The filesystem directory to serve over HTTP defaults to the `public` directory in the app's source code. This root can be overriden for other use-cases, for example:

```toml
[_.metadata.web-server]
root = "my_docroot"
```

### Response Headers

#### Global Headers

Respond with custom headers for any request path, the wildcard `*`.

```toml
[_.metadata.web-server.headers."*"]
X-Server = "hot stuff"
```

#### Path-matched Headers

Respond with custom headers. These match exactly against the request URL's path.

```toml
# The index page (index.html is not specified in the URL).
[_.metadata.web-server.headers."/"]
Cache-Control = "max-age=604800, stale-while-revalidate=86400, stale-if-error=86400"

# HTML pages.
[_.metadata.web-server.headers."/*.html"]
Cache-Control = "max-age=604800, stale-while-revalidate=86400, stale-if-error=86400"

# Contents of a subdirectory.
[_.metadata.web-server.headers."/images/*"]
Cache-Control = "max-age=31536000, immutable"

# Set multiple headers for a match.
[_.metadata.web-server.headers."/downloads/*"]
Cache-Control = "public, max-age=604800"
Content-Disposition = "attachment"
```

### Custom Errors

Respond with a custom Not Found HTML page.

```toml
[_.metadata.web-server.errors]
404 = "public/error-404.html"
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
