#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
output="$repo_root/target/userguide"

command -v spore >/dev/null || { printf 'Spore is required to build the guide.\n' >&2; exit 1; }
mkdir -p "$repo_root/target"
build_root=$(mktemp -d "$repo_root/target/userguide-build.XXXXXX")

cleanup() {
  if [[ -d "$build_root/previous" && ! -e "$output" ]]; then
    mv -- "$build_root/previous" "$output"
  fi
  rm -rf -- "$build_root"
}
trap cleanup EXIT

project="$build_root/source"
mkdir -p "$project"
cp -R -- "$repo_root/publication/userguide/." "$project/"

# Content-based asset names avoid stale styles and appearance defaults at the CDN.
for theme_asset in scripts/appearance.js styles/book.css; do
  asset_name=${theme_asset##*/}
  asset_digest=$(sha256sum "$project/themes/edition/$theme_asset")
  asset_digest=${asset_digest:0:12}
  asset_output="${asset_name%.*}.$asset_digest.${asset_name##*.}"
  sed -i "s@output = \"assets/edition/$asset_name\"@output = \"assets/edition/$asset_output\"@" \
    "$project/themes/edition/theme.toml"
done

cargo run --quiet --manifest-path "$repo_root/Cargo.toml" --example userguide-publication -- "$project"

spore --project "$project" check --strict --links
spore --project "$project" render --force --recursive
cargo run --quiet --manifest-path "$repo_root/Cargo.toml" --example userguide-publication -- "$project" --finish
cp -R -- "$project/downloads/." "$project/public/"

if [[ -e "$output" || -L "$output" ]]; then
  mv -- "$output" "$build_root/previous"
fi
mv -- "$project/public" "$output"
printf 'User Guide built: %s\n' "$output"
