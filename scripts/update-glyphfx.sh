#!/usr/bin/env bash
set -euo pipefail

usage() {
    printf 'Usage: %s vMAJOR.MINOR.PATCH\n' "${0##*/}"
}

if [[ $# == 1 && $1 == --help ]]; then
    usage
    exit 0
fi
if [[ $# != 1 || ! $1 =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    usage >&2
    exit 2
fi

tag=$1
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
dest=vendor/glyphfx
upstream=https://github.com/blackopsrepl/glyphfx.git

require_clean_vendor() {
    local changes
    changes=$(git status --porcelain --untracked-files=all --ignored -- "$dest")
    if [[ -n $changes ]]; then
        printf 'Refusing to overwrite local changes in %s:\n%s\n' "$dest" "$changes" >&2
        exit 1
    fi
}

require_clean_vendor
stage=$(mktemp -d 'vendor/.glyphfx-update.XXXXXXXX')
cleanup() {
    # Restore the previous snapshot if the second rename did not complete.
    if [[ -d $stage/previous && ! -e $dest ]]; then
        mv -- "$stage/previous" "$dest" || return
    fi
    rm -rf -- "$stage"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

git init --quiet "$stage/upstream"
git -C "$stage/upstream" fetch --quiet --depth=1 "$upstream" "refs/tags/$tag"
commit=$(git -C "$stage/upstream" rev-parse 'FETCH_HEAD^{commit}')
mkdir "$stage/snapshot"
git -C "$stage/upstream" archive "$commit" src LICENSE NOTICE VERSION |
    tar -xf - -C "$stage/snapshot"

# demofx compiles the engine directly; it does not link the upstream CLI entry point.
rm -f -- "$stage/snapshot/src/main.c"
if [[ $(<"$stage/snapshot/VERSION") != "${tag#v}" ]]; then
    printf 'Upstream VERSION does not match tag %s.\n' "$tag" >&2
    exit 1
fi

require_clean_vendor
if diff -qr "$dest" "$stage/snapshot" >/dev/null; then
    printf 'glyphfx %s (%s) is already vendored.\n' "$tag" "$commit"
else
    status=$?
    if [[ $status != 1 ]]; then
        exit "$status"
    fi
    mv -- "$dest" "$stage/previous"
    mv -- "$stage/snapshot" "$dest"
    printf 'Vendored glyphfx %s (%s).\n' "$tag" "$commit"
fi
