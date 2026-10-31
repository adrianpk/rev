#!/bin/sh
# Owned peer: records frames and returns test-selected JSON without source IO.
set -eu
mode=$1
log=$2
printf '%s\n' "$$" > "$log.pid"
frame() {
    bytes=$(printf '%s' "$1" | LC_ALL=C wc -c)
    printf 'Content-Length: %s\r\n\r\n%s' "$bytes" "$1"
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
provider=',"completionProvider":{"triggerCharacters":["."]}'
encoding='utf-16'
test "$mode" != utf8 || encoding='utf-8'
test "$mode" != unsupported || provider=''
frame '{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"positionEncoding":"'"$encoding"'","textDocumentSync":2'"$provider"'}}}'
old=''
while read_frame; do
    case "$payload" in
        *'"method":"textDocument/completion"'*)
            id=$(printf '%s' "$payload" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
            case "$mode" in
                timeout) continue ;;
                crash) exit 0 ;;
                late) old=$id; continue ;;
            esac
            result=$(cat "$log.result")
            frame '{"jsonrpc":"2.0","id":'"$id"',"result":'"$result"'}'
            ;;
        *'"method":"$/cancelRequest"'*)
            if test -n "$old"; then
                result=$(cat "$log.result")
                frame '{"jsonrpc":"2.0","id":'"$old"',"result":'"$result"'}'
                printf 'sent\n' >> "$log.late"
                old=''
            fi
            ;;
        *'"method":"shutdown"'*) frame '{"jsonrpc":"2.0","id":2,"result":null}' ;;
        *'"method":"exit"'*) exit 0 ;;
    esac
done
