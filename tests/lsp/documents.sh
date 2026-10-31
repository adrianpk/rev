#!/bin/sh
# A bounded local peer records the real client frames; no source is read or saved.
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
        case "$line" in 'Content-Length: '*) length=${line#Content-Length: } ;; esac
    done
    test "$length" -gt 0 || return 1
    payload=$(dd bs=1 count="$length" 2>/dev/null)
    printf '%s\n' "$payload" >> "$log.frames"
}
read_frame || exit 1
if test "$mode" = paused; then
    printf 'waiting\n' > "$log.waiting"
    IFS= read -r ready < "$log.gate"
fi
encoding='"positionEncoding":"utf-8",'
change=2
save='{"includeText":true}'
case "$mode" in
    utf16) encoding='' ;;
    full) encoding=''; change=1; save=true ;;
    no-save) save=false ;;
esac
frame '{"jsonrpc":"2.0","id":1,"result":{"capabilities":{'"$encoding"'"textDocumentSync":{"openClose":true,"change":'"$change"',"save":'"$save"'},"completionProvider":{}}}}'
while read_frame; do
    case "$payload" in
        *'"method":"shutdown"'*) frame '{"jsonrpc":"2.0","id":2,"result":null}' ;;
        *'"method":"exit"'*) exit 0 ;;
    esac
done
