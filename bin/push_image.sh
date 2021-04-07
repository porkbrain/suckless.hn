#!/bin/bash

## Builds and pushes the image which is ran on k8s cluster and update the
## version in the k8s description config in ../k8s/cron.yml.

# gets the version from cargo toml
readonly REGEX_FIND_VERSION="version = \"([a-zA-Z0-9\-\.]+)\""
if [[ "$(cat Cargo.toml)" =~ $REGEX_FIND_VERSION ]];
then
    readonly bin_version="${BASH_REMATCH[1]}"
    readonly image_name="porkbrain/suckless.hn:${bin_version}"
else
    echo "Cargo.toml must include version."
    exit 1
fi

echo "Building and pushing docker image '${image_name}'..."
echo

docker build . --tag "${image_name}"
docker push "${image_name}"

echo
echo "Changing the version in k8s/cron.yml"
