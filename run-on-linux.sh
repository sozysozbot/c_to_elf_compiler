#!/usr/bin/env bash
set -eu

if [ "$(uname -ms)" = "Linux x86_64" ]; then
    $@
else 
    docker run --rm --platform linux/x86_64 -v $PWD:/workdir -w /workdir alpine:3 /workdir/$@
fi
