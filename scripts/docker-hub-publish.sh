#!/usr/bin/env bash

VERSION=$1

if [[ -z "$1" ]] ; then
    echo "Usage: ./scripts/docker-hub-publish.sh VERSION"
    exit 1
fi

docker build . -t zeitgeistpm/zeitgeist-node:$1 -t zeitgeistpm/zeitgeist-node:latest --build-arg PROFILE=production
docker push zeitgeistpm/zeitgeist-node:$1
docker push zeitgeistpm/zeitgeist-node:latest
