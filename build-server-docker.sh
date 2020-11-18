#!/usr/bin/env bash

cross build --target x86_64-unknown-linux-musl --release

docker build --rm --no-cache -t pegasis0/pick_9_server:latest .
