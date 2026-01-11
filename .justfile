set export := true
set dotenv-load := true

POSTGRES_PORT := '5432'
POSTGRES_USER := 'postgres'
POSTGRES_PASSWORD := 'postgres'
DATABASE_EXTERNAL_DSN := 'postgres://' + POSTGRES_USER + ':' + POSTGRES_PASSWORD + '@localhost:' + POSTGRES_PORT
DATABASE_INTERNAL_DSN := 'postgres://' + POSTGRES_USER + ':' + POSTGRES_PASSWORD + '@postgres:' + POSTGRES_PORT
DATABASE_URL := DATABASE_EXTERNAL_DSN
MIGRATIONS_PATH := "./src/facts/migrations"
PASSWORD_HASH := '$argon2i$v=19$m=16,t=2,p=1$V0hsSTM0NGliMFBqaWdqeg$B/A24H9SHPO9+ll0geym3Q' # in fact is a hashed `password` string (`dXNlcjpwYXNzd29yZA==` for basic auth)
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
    --database-url "${DATABASE_EXTERNAL_DSN}" \
    -- \
    --all-targets \
    --all-features \
    --tests
  just down