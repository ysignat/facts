set export := true
set dotenv-load := true

CONTAINER_NAME := 'api'
NETWORK_NAME := 'app-network'

default:
  @just --list

start:
  #!/usr/bin/env sh
  set -eu

  docker network create \
    --driver bridge \
    "${NETWORK_NAME}"

  WORKDIR='/app'
  RUST_VERSION="$(grep 'rust-version' Cargo.toml | sed 's/rust-version = \"\(.*\)\"/\1/')"
  ALPINE_VERSION='3.21'
  PORT='8080'
  TAG="$(
    docker build \
      --quiet \
      --file dev.dockerfile \
      --build-arg "RUST_VERSION=${RUST_VERSION}" \
      --build-arg "ALPINE_VERSION=${ALPINE_VERSION}" \
      .
  )"
  docker run \
    --rm \
    --detach \
    --user "$(id -u):$(id -g)" \
    --volume "${PWD}:${WORKDIR}" \
    --publish "${PORT}:${PORT}" \
    --workdir "${WORKDIR}" \
    --name "${CONTAINER_NAME}" \
    --network "${NETWORK_NAME}" \
    --env "HOST=0.0.0.0" \
    --env "PORT=${PORT}" \
    --env "LOG_LEVEL=TRACE" \
    "${TAG}"

stop:
  #!/usr/bin/env sh
  set -eu

  docker container rm \
    --force \
    --volumes \
    "${CONTAINER_NAME}"

  docker network rm \
    --force \
    "${NETWORK_NAME}" 

restart:
  just stop
  just start
