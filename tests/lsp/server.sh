#!/bin/sh
# Deterministic stdio peer; the integration test owns its process and directory.
set -eu
mode=$1
log=$2
printf '%s\n' "$$" >> "$log"
frame() {
    bytes=$(printf '%s' "$1" | LC_ALL=C wc -c)
    printf 'Content-Length: %s\r\n\r\n%s' "$bytes" "$1"
}
read_frame() {
    length=0
    while IFS= read -r line; do
        line=$(printf '%s' "$line" | tr -d '\r')
        test -n "$line" || break
        case "$line" in
            'Content-Length: '*) length=${line#Content-Length: } ;;
        esac
    done
    test "$length" -gt 0 || return 1
    payload=$(dd bs=1 count="$length" 2>/dev/null)
}
read_frame || exit 1
printf '%s\n' "$payload" > "$log.initialize"
case "$mode" in
    timeout) exec sleep 30 ;;
    eof) exit 0 ;;
    header) printf 'Content-Length: bad\r\n\r\n'; exec sleep 30 ;;
    oversized) printf 'Content-Length: 8388609\r\n\r\n'; exec sleep 30 ;;
    long-header) head -c 16385 /dev/zero | tr '\000' x; exec sleep 30 ;;
    bad-json) frame '{broken'; exec sleep 30 ;;
    failure) frame '{"jsonrpc":"2.0","id":1,"error":{"code":-32603,"message":"rejected"}}'; exec sleep 30 ;;
    encoding) frame '{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"positionEncoding":"utf-32","textDocumentSync":1}}}'; exec sleep 30 ;;
    no-sync) frame '{"jsonrpc":"2.0","id":1,"result":{"capabilities":{}}}'; exec sleep 30 ;;
    stderr) head -c 131072 /dev/zero >&2 ;;
esac
# Split headers and combine frames. A non-ASCII notification checks byte lengths.
printf 'Content-Len'
printf 'gth: 124\r\n\r\n'
printf '%s' '{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"positionEncoding":"utf-8","textDocumentSync":2,"completionProvider":{}}}}'
frame '{"jsonrpc":"2.0","method":"window/logMessage","params":{"message":"界😀"}}'
while read_frame; do
    case "$payload" in
        *'"method":"initialized"'*) printf 'initialized\n' >> "$log.events" ;;
        *'"method":"shutdown"'*)
            printf 'shutdown\n' >> "$log.events"
            frame '{"jsonrpc":"2.0","id":2,"result":null}' ;;
        *'"method":"exit"'*) printf 'exit\n' >> "$log.events"; exit 0 ;;
    esac
done
