#!/bin/sh

set -eu

CWD="$1"
COUNTER="${2:-20}"

if [ "$COUNTER" -eq "0" ]
then
    cd "$CWD"
    sleep 10
elif [ "$COUNTER" -eq "30" ]
then
    sudo env "USER=$USER" "$0" "$CWD" "$(($COUNTER - 1))"
elif [ "$COUNTER" -eq "25" ]
then
    sudo -u "$USER" "$0" "$CWD" "$(($COUNTER - 1))"
else
    "$0" "$CWD" "$(($COUNTER - 1))"
fi
