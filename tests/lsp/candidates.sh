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
case "$mode" in resolve*) provider=',"completionProvider":{"triggerCharacters":["."],"resolveProvider":true}' ;; esac
encoding='utf-16'
case "$mode" in utf8|resolve-utf8) encoding='utf-8' ;; esac
test "$mode" != unsupported || provider=''
frame '{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"positionEncoding":"'"$encoding"'","textDocumentSync":2'"$provider"'}}}'
old=''
late_result="$log.result"
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
        *'"method":"completionItem/resolve"'*)
            id=$(printf '%s' "$payload" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
            case "$mode" in
                resolve-timeout) continue ;;
                resolve-eof) exit 0 ;;
                resolve-late) old=$id; late_result="$log.resolved"; continue ;;
                resolve-error) frame '{"jsonrpc":"2.0","id":'"$id"',"error":{"code":-32603,"message":"fixture failure"}}'; continue ;;
            esac
            result=$(cat "$log.resolved")
            frame '{"jsonrpc":"2.0","id":'"$id"',"result":'"$result"'}'
            ;;
        *'"method":"$/cancelRequest"'*)
            if test -n "$old"; then
                result=$(cat "$late_result")
                frame '{"jsonrpc":"2.0","id":'"$old"',"result":'"$result"'}'
                printf 'sent\n' >> "$log.late"
                old=''
            fi
            ;;
        *'"method":"shutdown"'*) frame '{"jsonrpc":"2.0","id":2,"result":null}' ;;
        *'"method":"exit"'*) exit 0 ;;
    esac
done
