#!/bin/sh
set -eu

IFS= read -r scenario <.scenario
subcommand=""
for argument in "$@"; do
    case "$argument" in
        rev-parse|ls-tree|cat-file)
            subcommand=$argument
            ;;
    esac
done

case "$scenario:$subcommand" in
    status:*)
        printf '%s' 'hostile status' >&2
        exit 23
        ;;
    partial:rev-parse)
        printf '%s' '0123456789012345678901234567890123456789'
        exit 0
        ;;
    timeout:*)
        printf '%s\n' "$$" >.hostile-parent-pid
        exec /bin/sleep 60
        ;;
    descendant:*)
        /bin/sleep 60 &
        descendant=$!
        printf '%s\n' "$$" >.hostile-parent-pid
        printf '%s\n' "$descendant" >.hostile-descendant-pid
        wait "$descendant"
        ;;
    *)
        exit 97
        ;;
esac
