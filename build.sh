#!/bin/bash

# Use this script like cargo.
# i.e. `./build.sh run` == `cargo run`
# Moves files in the resources foler to the executable directory.

set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd $DIR

npm run dev

cargo "$@"

[ -d "${DIR}/target/debug" ] && cp -R "${DIR}/resources/." "${DIR}/target/debug/"
[ -d "${DIR}/target/release" ] && cp -R "${DIR}/resources/." "${DIR}/target/release/"
