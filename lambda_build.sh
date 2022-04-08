#!/usr/bin/env bash

VERSION=$1
if [ -z "$VERSION" ]; then
  echo "Please supply version (e.g. 0.1.3) as the first argument to this script"
  exit 1
fi

ARCH=$2
if [ -z "$ARCH" ]; then
  echo "Please supply architecture (x64, arm64) as the first argument to this script"
  exit 1
fi

docker build --build-arg VERSION=$VERSION --build-arg ARCH=$ARCH -t full_text_search_demo:latest -f Dockerfile.lambda_build ./

docker run --rm --detach --name builder --entrypoint tail full_text_search_demo:latest -f /dev/null

docker cp builder:/var/task/build/stage/$VERSION/index-v$VERSION-linux-$ARCH.tar.gz ./

docker kill builder

ls -lh ./index-v$VERSION-linux-$ARCH.tar.gz