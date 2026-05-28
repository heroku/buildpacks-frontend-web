#!/usr/bin/env bash
#
# Refresh buildpacks/static-web-server/inventory.toml against the latest
# Caddy release on GitHub. Also updates the WEB_SERVER_VERSION constant
# in src/main.rs and writes a one-line changelog entry.
#
# The pinned checksums come from GitHub's release-asset `digest` field,
# which GitHub computes server-side at upload. That gives us a trust
# anchor independent of upstream's own `caddy_<v>_checksums.txt` file.
#
# Always rewrites inventory.toml as a pure projection of upstream state.
# When nothing changed, all three edited files are byte-identical to
# their committed versions and the downstream caller (e.g.
# peter-evans/create-pull-request) opens no PR.
#
# Required tools: gh, jq, awk.
# Optional env: GH_TOKEN (or GITHUB_TOKEN), to avoid the unauthenticated
# GitHub API rate limit. gh picks these up automatically.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INVENTORY="${REPO_ROOT}/buildpacks/static-web-server/inventory.toml"
MAIN_RS="${REPO_ROOT}/buildpacks/static-web-server/src/main.rs"
CHANGELOG="${REPO_ROOT}/buildpacks/static-web-server/CHANGELOG.md"
UPSTREAM_REPO="caddyserver/caddy"

# Architectures we ship in inventory.toml. The order is preserved when
# the file is rewritten.
ARCHES=(amd64 arm64)

main() {
    local release_json new
    release_json="$(gh api "repos/${UPSTREAM_REPO}/releases/latest")"
    new="$(jq -er '.tag_name | sub("^v"; "")' <<<"$release_json")" || {
        echo "Failed to read latest Caddy version from upstream." >&2
        exit 1
    }

    echo "Latest upstream Caddy is v${new}."

    # Rewrite inventory.toml from upstream. For each arch we ship,
    # extract the matching asset's browser_download_url + digest.
    {
        local first=1
        for arch in "${ARCHES[@]}"; do
            local tarball="caddy_${new}_linux_${arch}.tar.gz"
            local url digest
            read -r url digest < <(jq -er --arg name "$tarball" '
                .assets[]
                | select(.name == $name)
                | "\(.browser_download_url) \(.digest)"
            ' <<<"$release_json") || {
                echo "No release asset named ${tarball} with both browser_download_url and digest." >&2
                exit 1
            }
            if (( first )); then first=0; else echo; fi
            cat <<EOF
[[artifacts]]
version = "${new}"
os = "linux"
arch = "${arch}"
url = "${url}"
checksum = "${digest}"
EOF
        done
    } > "$INVENTORY"

    # Update the WEB_SERVER_VERSION constant in main.rs to match.
    # Idempotent: if the constant already equals ${new}, the rewrite is
    # a no-op.
    local updated_main_rs
    updated_main_rs="$(awk -v ver="$new" '
        /^pub\(crate\) const WEB_SERVER_VERSION:/ {
            print "pub(crate) const WEB_SERVER_VERSION: &str = \"" ver "\";"
            next
        }
        { print }
    ' "$MAIN_RS")"
    printf '%s\n' "$updated_main_rs" > "$MAIN_RS"

    # Add a changelog entry under [Unreleased]. Each run starts from
    # main (the workflow's PR action force-pushes update-caddy-inventory
    # on every run), so we never see a previous entry to dedupe against.
    local updated_changelog
    updated_changelog="$(awk -v ver="$new" '
        /^## \[Unreleased\]$/ && !done {
            print
            print ""
            print "- Update Caddy web server version to " ver "."
            done = 1
            next
        }
        { print }
    ' "$CHANGELOG")"
    printf '%s\n' "$updated_changelog" > "$CHANGELOG"
}

main "$@"
