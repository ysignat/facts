set export := true
set dotenv-load := true

PORT := '8080'
POSTGRES_PORT := '5432'
POSTGRES_USER := 'postgres'
POSTGRES_PASSWORD := 'postgres'
DATABASE_EXTERNAL_DSN := 'postgres://' + POSTGRES_USER + ':' + POSTGRES_PASSWORD + '@localhost:' + POSTGRES_PORT
DATABASE_INTERNAL_DSN := 'postgres://' + POSTGRES_USER + ':' + POSTGRES_PASSWORD + '@postgres:' + POSTGRES_PORT
MIGRATIONS_PATH := "./src/facts/migrations"
RUST_VERSION := `grep 'rust-version' Cargo.toml | sed 's/rust-version = \"\(.*\)\"/\1/'`

default:
  @just --list  

up:
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

run:
  just up
  cargo run -- \
    --log-level 'trace' \
    --bind-port "${PORT}" \
    --storage-dsn "${DATABASE_EXTERNAL_DSN}"

down:
  docker-compose down

restart:
  just down
  just up

test *args:
  just up
  cargo test {{ args }}
  just down

prepare:
  just up
  cargo sqlx prepare \
    --all \
    --database-url 'postgres://postgres:postgres@localhost:5432' \
    -- \
    --all-targets \
    --all-features \
    --tests
  just down