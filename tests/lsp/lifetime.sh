#!/bin/sh
set -eu
mode=$1
log=$2
printf '%s\n' "$$" >> "$log.pids"
frame() {
    printf 'Content-Length: %s\r\n\r\n%s' "${#1}" "$1"
}
read_frame() {
    length=0
    while IFS= read -r line; do
        line=$(printf '%s' "$line" | tr -d '\r')
        test -n "$line" || break
        case "$line" in 'Content-Length: '*) length=${line#Content-Length: } ;; esac
    done
    test "$length" -gt 0 || return 1
    payload=$(dd bs=1 count="$length" 2>/dev/null)
    printf '%s\n' "$payload" >> "$log.frames"
}
read_frame || exit 1
if test "$mode" = fail-once && ! test -e "$log.failed"; then
    : > "$log.failed"
    exit 0
fi
if test "$mode" = initialize-stall; then
    : > "$log.waiting"
    read -r release < "$log.gate"
fi
frame '{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"textDocumentSync":1,"completionProvider":{}}}}'
while read_frame; do
    case "$payload" in
        *'"method":"textDocument/didOpen"'*)
            if test "$mode" = no-read; then
                : > "$log.waiting"
                read -r release < "$log.gate"
            fi
            if test "$mode" = requests; then
                while IFS= read -r request; do frame "$request"; done < "$log.requests"
            fi
            ;;
        *'"method":"textDocument/completion"'*)
            id=$(printf '%s' "$payload" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
            frame '{"jsonrpc":"2.0","id":'"$id"',"result":[]}'
            ;;
        *'"method":"shutdown"'*)
            test "$mode" = shutdown-stall || frame '{"jsonrpc":"2.0","id":2,"result":null}'
            ;;
        *'"method":"exit"'*) exit 0 ;;
    esac
done
