#!/bin/bash

## Builds the binary for arm.

set -e

# https://chacin.dev/blog/cross-compiling-rust-for-the-raspberry-pi
readonly ARM_TARGET="armv7-unknown-linux-musleabihf"
readonly BIN_NAME="suckless_hn"
readonly REGEX_FIND_BIN_NAME="name = \"(\w+)\""

if [[ "$(cat Cargo.toml)" =~ $REGEX_FIND_BIN_NAME ]];
then
    readonly bin_name="${BASH_REMATCH[1]}"
    readonly bin_path="target/${ARM_TARGET}/release/${bin_name}"

    echo "Building binary '${bin_name}'..."
    echo
else
    echo "Cargo.toml must include project name."
    exit 1
fi

# build docker image for the target so that cross can use it to compile
docker build -t "${ARM_TARGET}" "${ARM_TARGET}"

cross build --release --target "${ARM_TARGET}"

