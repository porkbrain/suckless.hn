#!/bin/bash

## Deploys binary to raspberry pi. Depending on the private key, a passphrase
## prompt might be shown.

set -e

if test -f ".env.deploy"; then
    echo "(source .env.deploy)"
    source .env.deploy
fi

if [ -z "${PI_HOST}" ]; then
    echo "PI_HOST env must be provided."
    exit 1
fi

if [ -z "${SSH_PRIVATE_KEY_PATH}" ]; then
    echo "SSH_PRIVATE_KEY_PATH env must be provided."
    exit 1
fi

# https://chacin.dev/blog/cross-compiling-rust-for-the-raspberry-pi
readonly ARM_TARGET="armv7-unknown-linux-gnueabihf"
readonly BIN_NAME="suckless_hn"
readonly REGEX_FIND_BIN_NAME="name = \"(\w+)\""

if [[ "$(cat Cargo.toml)" =~ $REGEX_FIND_BIN_NAME ]];
then
    readonly bin_name="${BASH_REMATCH[1]}"
    readonly bin_path="target/${ARM_TARGET}/release/${bin_name}"

    echo "Deploying binary '${bin_name}'..."
    echo
else
    echo "Cargo.toml must include project name."
    exit 1
fi

# build and copy suckless.hn binary
cross build --target "${ARM_TARGET}" --release
rsync -e "ssh -p ${SSH_PORT:-22} -i ${SSH_PRIVATE_KEY_PATH}" \
    "${bin_path}" \
    ${PI_HOST}:/home/pi/suckless.hn
