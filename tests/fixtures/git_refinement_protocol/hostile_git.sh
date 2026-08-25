#!/bin/sh

# Controlled Git executable for P0100/B1. It intentionally uses only shell
# builtins because the adapter's normative environment has no PATH.
log=protocol.log
{
    printf 'BEGIN\n'
    printf 'ARGV'
    for arg in "$@"; do
        printf ' <%s>' "$arg"
    done
    printf '\n'
    printf 'ENV GIT_TERMINAL_PROMPT=<%s> GIT_NO_LAZY_FETCH=<%s> GIT_OPTIONAL_LOCKS=<%s>\n' \
        "$GIT_TERMINAL_PROMPT" "$GIT_NO_LAZY_FETCH" "$GIT_OPTIONAL_LOCKS"
    printf 'ENV GIT_NO_REPLACE_OBJECTS=<%s> GIT_CONFIG_NOSYSTEM=<%s> GIT_CONFIG_GLOBAL=<%s> LC_ALL=<%s>\n' \
        "$GIT_NO_REPLACE_OBJECTS" "$GIT_CONFIG_NOSYSTEM" "$GIT_CONFIG_GLOBAL" "$LC_ALL"
    printf 'ABSENT PATH=<%s> HOME=<%s> XDG_CONFIG_HOME=<%s> GIT_DIR=<%s>\n' \
        "${PATH+x}" "${HOME+x}" "${XDG_CONFIG_HOME+x}" "${GIT_DIR+x}"
} >> "$log"

command=
for arg in "$@"; do
    case "$arg" in
        rev-parse|ls-tree|cat-file) command=$arg ;;
    esac
done

oid40=1111111111111111111111111111111111111111
oid64=2222222222222222222222222222222222222222222222222222222222222222
blob_a=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
blob_b=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb

case "$command" in
    rev-parse)
        if [ -e scenario_missing_ref ]; then
            exit 1
        elif [ -e scenario_oid64 ]; then
            printf '%s\n' "$oid64"
        else
            printf '%s\n' "$oid40"
        fi
        ;;
    ls-tree)
        if [ -e scenario_types ]; then
            printf '100644 blob %s\ta.rs\0' "$blob_a"
            printf '160000 commit %s\tgitlink\0' "$blob_b"
            printf '120000 blob %s\tlink\0' "$blob_b"
        elif [ -e scenario_budget ]; then
            printf '100644 blob %s\tlarge.bin\0' "$blob_a"
        elif [ -e scenario_bad_framing ]; then
            printf '100644 blob %s\ta.rs' "$blob_a"
        else
            printf '100755 blob %s\t-odd.rs\0' "$blob_a"
        fi
        ;;
    cat-file)
        while IFS= read -r line; do
            printf 'STDIN <%s>\n' "$line" >> "$log"
            case "$line" in
                "contents $blob_a")
                    if [ -e scenario_budget ]; then
                        printf '%s blob 4194305\n' "$blob_a"
                    else
                        printf '%s blob 3\nabc\n' "$blob_a"
                    fi
                    ;;
                flush) ;;
                *) exit 7 ;;
            esac
        done
        if [ -e scenario_extra_bytes ]; then
            printf 'x'
        fi
        ;;
    *) exit 9 ;;
esac
