#!/bin/sh
# Controlled acquisition tools: no Ruby, Python or network is used by regressions.
set -eu
root=$REV_OPTIONAL_FIXTURE
printf '%s %s\n' "$0" "$*" >> "$root/calls"
if test -f "$root/flood-stderr"; then /usr/bin/head -c 65536 /dev/zero | /usr/bin/tr '\000' '\377' >&2; exit 42; fi
if test -f "$root/reject-tool"; then echo 'incompatible optional runtime' >&2; exit 42; fi
case "${0##*/}" in
curl)
    output=
    while test "$#" -gt 0; do
        case "$1" in --output) shift; output=$1;; esac
        shift
    done
    if test -n "$output"; then /bin/cp "$root/server" "$output"; else /bin/cat "$root/release.json"; fi;;
gem)
    test "$1" = install
    test "$2" = --norc
    test "$GEM_HOME" = "$GEM_PATH"
    test "$TMPDIR" != /outside/tmp
    test "$MAKEFLAGS" = '-j1 --jobserver-style=pipe'
    test -z "$RUBYOPT"
    test "$BUNDLE_GEMFILE" = "${GEM_HOME%/gems}/Gemfile"
    home=$GEM_HOME; bindir=
    while test "$#" -gt 0; do
        case "$1" in --bindir) shift; bindir=$1;; --install-dir) shift; test "$1" = "$home";; esac
        shift
    done
    /bin/mkdir -p "$home/cache" "$bindir"
    /bin/cp "$root/server" "$bindir/ruby-lsp"
    printf fixture-gem > "$home/cache/ruby-lsp-0.26.1.gem";;
ruby)
    case "$2" in
        *Net::HTTP*) /bin/cat "$root/ruby.json";;
        *Bundler::CLI*) printf 'GEM\n  remote: https://rubygems.org/\n' > "${GEM_HOME%/gems}/Gemfile.lock";;
        *Gem::Specification*)
            lock=$(/usr/bin/sha256sum "${GEM_HOME%/gems}/Gemfile.lock"); lock=${lock%% *}
            gem=$(/usr/bin/sha256sum "$GEM_HOME/cache/ruby-lsp-0.26.1.gem"); gem=${gem%% *}
            if test -f "$root/reject-evidence"; then gem=bad; fi
            printf '{"runtime":"controlled Ruby 3.4","packages":[{"name":"ruby-lsp","version":"0.26.1"}],"lock_sha256":"%s","primary_sha256":"%s"}' "$lock" "$gem";;
        *) exit 88;;
    esac;;
python3|python)
    test "$1" = -I
    case "$3" in
        *urllib.request*) printf '1.13.0\n';;
        venv)
            test "$4" = --copies
            /bin/mkdir -p "$5/bin"
            /bin/cp "$root/bin/python3" "$5/bin/python"
            /bin/cp "$root/server" "$5/bin/pylsp"
            printf '%s\n' "$5" >> "$root/venvs";;
        pip)
            test "$4" = --isolated
            report=
            while test "$#" -gt 0; do
                case "$1" in --report) shift; report=$1;; *'['*) exit 89;; esac
                shift
            done
            /bin/cp "$root/pip-report.json" "$report";;
        *importlib.metadata*)
            digest=$(/usr/bin/sha256sum "$4"); digest=${digest%% *}
            version=1.13.0
            if test -f "$root/reject-evidence"; then version=999; fi
            printf '{"version":"%s","runtime":"controlled Python 3.14","report_sha256":"%s"}' "$version" "$digest";;
        *) exit 88;;
    esac;;
*) exit 88;;
esac
