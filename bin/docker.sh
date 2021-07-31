#!/bin/bash

set -e

# update version
tag="$(git rev-parse HEAD | head -c 7 | awk '{ printf "%s", $0 }')"
reg=docker.jaemk.me
app=nihondrill

echo "building images... latest, $tag "

docker build -t $reg/$app:$tag .
docker build -t $reg/$app:latest .

ports="-p 3003:3003"

# set envs from csv env var
if [[ -z "$ENVS" ]]; then
    envs="$envs"
else
    for e_str in $(echo $ENVS | tr "," "\n")
    do
        envs="-e $e_str $envs"
    done
fi

# set key-value pairs if there's a .env
if [[ -z "$ENVFILE" ]]; then
    if [ -f .env ]; then
        envfile="--env-file .env"
    fi
else
    envfile="--env-file $ENVFILE"
fi

root=$(git rev-parse --show-toplevel)

if [ "$1" = "run" ]; then
    echo "running..."
    set -x
    docker run --rm -it --init --net=host $ports $envs $envfile $reg/$app:latest
elif [ "$1" = "push" ]; then
    echo "pushing images..."
    set -x
    docker push $reg/$app:$tag
    docker push $reg/$app:latest
fi
