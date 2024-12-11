# Heroku Front-end Web Builder

*This is an early internal release, this builder configuration is temporary. Front-end Web will eventually be included in Heroku's default builder.*

Use the builder in a Heroku Fir app's `project.toml`:

```bash
[_]
schema-version = "0.2"

[io.buildpacks]
builder = "ghcr.io/heroku/builder-test-public:frontend-web-builder-latest_linux-arm64"
```

Optionally, [configure static-web-server](../buildpacks/static-web-server/README.md) for the app.

Then, commit and `git push heroku` to a Fir app.

## Internal Release Process

*Once these Front-end Web CNBs are public, open-sourced, use the [automated release workflow](../README.md#releasing-a-new-version).*

*Buildpack version numbers should be set in the following commands, after running [Prepare Release workflow](../README.md#releasing-a-new-version).*

Generate an internal preview builder for these buildpacks:

```bash
cargo libcnb package --release --target aarch64-unknown-linux-musl
pack buildpack package website --config packaged/aarch64-unknown-linux-musl/release/heroku_website/package.toml  --target "linux/arm64" --format file
pack buildpack package website-nodejs --config packaged/aarch64-unknown-linux-musl/release/heroku_website-nodejs/package.toml  --target "linux/arm64" --format file
pack buildpack package static-web-server --config packaged/aarch64-unknown-linux-musl/release/heroku_static-web-server/package.toml  --target "linux/arm64" --format file
pack builder create frontend-web-builder --config builder/builder.toml --target "linux/arm64"
```

Example push to internal registry:

Using https://github.com/heroku/builder-test-public

```bash
export CR_PAT=XXXXX
echo $CR_PAT | docker login ghcr.io -u mars --password-stdin

# push the specific version
docker tag frontend-web-builder ghcr.io/heroku/builder-test-public:frontend-web-builder-1.1.1_linux-arm64
docker push ghcr.io/heroku/builder-test-public:frontend-web-builder-1.1.1_linux-arm64

# also push as "latest"
docker tag frontend-web-builder ghcr.io/heroku/builder-test-public:frontend-web-builder-latest_linux-arm64
docker push ghcr.io/heroku/builder-test-public:frontend-web-builder-latest_linux-arm64 
```
