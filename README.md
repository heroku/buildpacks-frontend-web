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

## Dev Notes

### Local Usage

```bash
cargo libcnb package

pack build my-cnb-website \
  --buildpack packaged/x86_64-unknown-linux-musl/debug/heroku_static-web-server \
  --buildpack packaged/x86_64-unknown-linux-musl/debug/heroku_website \
  --buildpack packaged/x86_64-unknown-linux-musl/debug/heroku_website-public-html \
  --builder heroku/builder:22 \
  --path buildpacks/website-public-html/tests/fixtures/public_html

docker run --env PORT=8888 -p 8888:8888 my-cnb-website
```
