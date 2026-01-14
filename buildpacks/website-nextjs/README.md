# Heroku Cloud Native Website (Next.js) Buildpack

* At build:
  * Detects `Next.js` in the app's `package.json` dependencies.
  * Performs heroku/nodejs install and runs build script.
  * Configures heroku/static-web-server for the detected framework.
* At launch:
  * Performs [runtime app configuration](../../buildpacks/static-web-server/README.md#runtime-app-configuration).
  * [static-web-server](../../buildpacks/static-web-server/README.md) runs with config generated during build.

## Usage

Create an app with [Next.js](https://nextjs.org/):

```bash
npx create-next-app@latest
```

Then, use [heroku/website-nodejs](../../meta-buildpacks/website-nodejs/README.md) meta-buildpack.

Note that this buildpack will only work with the nextJS [static export](https://nextjs.org/docs/app/guides/static-exports) option enabled. If you wish to use a server-side NextJS application, instead use the nodejs buildpack.
