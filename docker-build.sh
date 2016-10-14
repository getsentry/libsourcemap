#!/bin/bash

set -eu
cd -P -- "$(dirname -- "$0")"

SYMSYND_MANYLINUX=${SYMSYND_MANYLINUX:-0}

# Maybe we can do better here.  This has a high enough chance of being
# an unsafe race condition but this one is portable :P
CIDFILE=$(mktemp -u)

# Clean up after outselves on the way out.
cleanup() {
  if [ -f "$CIDFILE" ]; then
    CID=$(cat "$CIDFILE")
    docker rm "$CID" 2> /dev/null
  fi
  rm -f "$CIDFILE"
}

# trigger a build
build() {
  cleanup
  docker rmi -f libsourcemap:$1 2> /dev/null || true
  docker build -t libsourcemap:$1 -f $2 .
  docker run --cidfile="$CIDFILE" libsourcemap:$1 wheel
  CID=$(cat "$CIDFILE")
  docker cp "$CID:/usr/src/libsourcemap/dist/." dist
}

# Make sure we clean up before we exit in any case
trap cleanup EXIT

mkdir -p dist

if [ x$SYMSYND_MANYLINUX == x1 ]; then
  build dev32 Dockerfile.manylinux.32
  build dev64 Dockerfile.manylinux.64
else
  build dev64 Dockerfile.manylinux.64
fi

ls -alh dist
