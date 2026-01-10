set export := true
set dotenv-load := true

PORT := '8080'
POSTGRES_PORT := '5432'
POSTGRES_USER := 'postgres'
POSTGRES_PASSWORD := 'postgres'
DATABASE_EXTERNAL_DSN := 'postgres://' + POSTGRES_USER + ':' + POSTGRES_PASSWORD + '@localhost:' + POSTGRES_PORT
DATABASE_INTERNAL_DSN := 'postgres://' + POSTGRES_USER + ':' + POSTGRES_PASSWORD + '@postgres:' + POSTGRES_PORT
MIGRATIONS_PATH := "./src/facts/migrations"

default:
  @just --list

start:
  #!/usr/bin/env sh
  set -eu

  export RUST_VERSION="$(grep 'rust-version' Cargo.toml | sed 's/rust-version = \"\(.*\)\"/\1/')"

  printf '[*] Starting infrastructure\n'
  docker-compose up \
    --detach \
    --wait \
    --remove-orphans \
    postgres

  docker-compose run \
    --build \
    --remove-orphans \
    --rm \
    migrations

  printf '[*] Starting app\n'
  cargo run -- \
    --log-level 'trace' \
    --bind-port "${PORT}" \
    --storage-dsn "${DATABASE_EXTERNAL_DSN}"

stop:
  #!/usr/bin/env sh
  set -eu

  docker-compose down

restart:
  just stop
  just start
