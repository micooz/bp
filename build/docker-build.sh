#!/bin/bash

DOCKER_IMAGE="rust:1.55.0-buster"
TARGET_NAME="bp"
WORK_DIR="/root/${TARGET_NAME}"

docker pull ${DOCKER_IMAGE}
docker run --rm --user "$(id -u)":"$(id -g)" -v "$PWD":${WORK_DIR} -w ${WORK_DIR} ${DOCKER_IMAGE} cargo build --release
