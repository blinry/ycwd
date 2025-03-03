#!/bin/sh

set -eu

SET_CWD="$(dirname "$0")/set_cwd.sh"

CWD="${1:-/tmp}"
mkdir -p "$CWD"

sudo sh -c ''

"$SET_CWD" "$CWD" &

sleep 1

RESULT="$(cargo run -q "$$")"

if [ "$RESULT" = "$CWD" ]
then
    echo "Got cwd: '$RESULT'"
else
    echo "Expected: '$CWD'"
    echo "Got:      '$RESULT'"
    exit 1
fi
