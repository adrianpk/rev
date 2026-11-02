#!/usr/bin/env bash
set -euo pipefail
printf 'curl\n' >> "$REV_INSTALLATION_FIXTURE/calls"
url= output=
while (($#)); do
    case "$1" in
        --url) shift; url=$1 ;;
        --output) shift; output=$1 ;;
    esac
    shift
done
case "$url" in
    https://api.github.com/repos/rust-lang/rust-analyzer/releases/latest)
        cat "$REV_INSTALLATION_FIXTURE/release.json" ;;
    https://github.com/rust-lang/rust-analyzer/releases/download/acceptance-local/rust-analyzer-x86_64-unknown-linux-gnu.gz)
        test -n "$output"
        cp -- "$REV_INSTALLATION_FIXTURE/server.gz" "$output" ;;
    *) echo 'unadmitted fixture request' >&2; exit 88 ;;
esac
