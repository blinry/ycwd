#!/bin/sh

set -eu

cargo build -q

SCRIPTS_DIR="$(dirname "$0")"
cd "${SCRIPTS_DIR}"
SCRIPTS_DIR="$(pwd)"
cd - > /dev/null

EXIT_CODE=0

for file in "${SCRIPTS_DIR}"/dirs/*
do
    echo "+ ${SCRIPTS_DIR}/test_path.sh '${file}'" >&2
    "${SCRIPTS_DIR}/test_path.sh" "${file}" || EXIT_CODE=1
done

exit "$EXIT_CODE"
