#!/bin/sh
set -eu

command_name=""
for argument in "$@"; do
    case "$argument" in
        rev-parse|ls-tree|cat-file) command_name="$argument" ;;
    esac
done

oid=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
blob_oid=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb

case "$command_name" in
    rev-parse)
        printf '%s\n' "$oid"
        ;;
    ls-tree)
        printf '100644 blob %s\tsrc/lib.rs\000' "$blob_oid"
        ;;
    cat-file)
        # Consume the adapter request so a write-side failure cannot mask the
        # lifecycle behavior under test.
        while IFS= read -r request; do
            [ "$request" = "flush" ] && break
        done
        printf '%s\n' "$$" > leader.pid
        if [ -f scenario-oversized ]; then
            printf '%s blob 4194305\n' "$blob_oid"
            /bin/sleep 60 &
            printf '%s\n' "$!" > descendant.pid
            /bin/sleep 60
        elif [ -f scenario-partial-pipes ]; then
            printf '%s blob 4\nxy' "$blob_oid"
            /bin/sleep 60 &
            printf '%s\n' "$!" > descendant.pid
            exit 0
        elif [ -f scenario-detached-pipes ]; then
            /bin/sleep 60 >/dev/null 2>&1 &
            printf '%s\n' "$!" > descendant.pid
            printf '%s blob 4\ndata\n' "$blob_oid"
            exit 0
        elif [ -f scenario-transcript-cap ]; then
            /usr/bin/head -c 33600001 /dev/zero
            /bin/sleep 60 &
            printf '%s\n' "$!" > descendant.pid
            /bin/sleep 60
        else
            exit 97
        fi
        ;;
    *)
        exit 96
        ;;
esac
