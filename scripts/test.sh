#!/bin/sh

set -eu

SCRIPTS_DIR="$(dirname "$0")"
cd "${SCRIPTS_DIR}"
SCRIPTS_DIR="$(pwd)"
cd - > /dev/null

for file in "${SCRIPTS_DIR}"/dirs/*
do
    "${SCRIPTS_DIR}/test_path.sh" "${file}"
done
