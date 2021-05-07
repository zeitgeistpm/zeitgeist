#!/usr/bin/env bash

VERSION=$1
FEATURES=$2 || "default"

if [[ -z "$1" ]] ; then
    echo "Usage: ./scripts/docker-hub-publish.sh VERSION"
    exit 1
fi

docker build --build-arg FEATURES=$FEATURES . -t zeitgeistpm/zeitgeist-node-$2:$1 -t zeitgeistpm/zeitgeist-node:latest
docker build --build-arg FEATURES=$FEATURES . -t zeitgeistpm/zeitgeist-node-$2:latest
docker push zeitgeistpm/zeitgeist-node-$2:$1
docker push zeitgeistpm/zeitgeist-node-$2:latest
