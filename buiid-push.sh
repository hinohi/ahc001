#!/bin/bash

set -eu

aws ecr get-login-password --region ap-northeast-1 | docker login --username AWS --password-stdin 169698630369.dkr.ecr.ap-northeast-1.amazonaws.com

docker build -t ahc001 .
tag=$(date '+%Y%m%d-%H%M%S')
ecr="169698630369.dkr.ecr.ap-northeast-1.amazonaws.com/ahc001:$tag"
echo "$ecr"
docker tag ahc001 "$ecr"
docker push "$ecr"
