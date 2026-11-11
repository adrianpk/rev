#!/usr/bin/env bash
set -euo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
root=$(git rev-parse --show-toplevel)
[[ "$PWD" == "$root" && ! -L .githooks && -f .githooks/pre-commit && ! -L .githooks/pre-commit && -x .githooks/pre-commit ]] || {
    printf '%s\n' 'Refusing unsafe or non-executable repository hook path.' >&2
    exit 1
}
if existing=$(git config --get core.hooksPath); then
    if [[ "$existing" != .githooks ]]; then
        printf 'Preserving conflicting core.hooksPath: %s\n' "$existing" >&2
        exit 1
    fi
else
    hooks=$(git rev-parse --path-format=absolute --git-path hooks)
    if [[ -L "$hooks" ]] || { [[ -d "$hooks" ]] && [[ -n $(find "$hooks" -mindepth 1 -maxdepth 1 ! -name '*.sample' -print -quit) ]]; }; then
        printf '%s\n' 'Preserving existing hooks; resolve the installation conflict explicitly.' >&2
        exit 1
    fi
fi
git config --local core.hooksPath .githooks
printf '%s\n' 'Installed repository-local core.hooksPath=.githooks (shared by linked worktrees).'
