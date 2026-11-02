#!/bin/sh
# Real local Git storage, controlled upstream identity and installed compiler.
set -eu
root=$REV_OPTIONAL_FIXTURE
printf '%s %s\n' "$0" "$*" >> "$root/calls"
revision=$(/bin/cat "$root/revision")
case "${0##*/}" in
git)
    test -z "${GIT_DIR-}"
    test -z "${GIT_OBJECT_DIRECTORY-}"
    test "$GIT_CONFIG_NOSYSTEM" = 1
    test "$GIT_CONFIG_GLOBAL" = /dev/null
    if test "$1" = ls-remote; then
        if test -f "$root/bad-revision"; then echo 'floating refs/heads/master'; else printf '%s\trefs/heads/master\n' "$revision"; fi
    elif test "$1" = init; then exec /usr/bin/git "$@"
    else
        dir=$1; work=$2; shift 2
        if test "$1" = -c; then shift 2; fi
        if test "$1" = fetch; then
            test "$2" = --depth=1
            test "$4" = https://github.com/DanielGavin/ols.git
            test "$5" = "$revision"
            exec /usr/bin/git "$dir" "$work" -c protocol.file.allow=always fetch --depth=1 --no-tags "$root/upstream" "$revision"
        fi
        if test -f "$root/wrong-head" && test "$1" = rev-parse; then echo 0000000000000000000000000000000000000000; else exec /usr/bin/git "$dir" "$work" -c core.hooksPath=/dev/null "$@"; fi
    fi;;
odin)
    if test "$1" = version; then echo 'controlled Odin 2026-08'; exit 0; fi
    test "$OLS_VERSION" = "$revision"
    if test -f "$root/reject-build"; then echo 'incompatible Odin compiler' >&2; exit 1; fi
    /bin/cp "$root/server" "$PWD/ols"
    if test -f "$root/no-builtin"; then /bin/rm -r "$PWD/builtin"; fi;;
*) exit 88;;
esac
