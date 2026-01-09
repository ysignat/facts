ARG RUST_VERSION
ARG ALPINE_VERSION
FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION}

RUN apk add musl-dev

ARG SQLX_CLI_VERSION
RUN \
  cargo install sqlx-cli@${SQLX_CLI_VERSION} \
  --no-default-features \
  --features rustls,postgres

ENTRYPOINT [ "cargo", "sqlx" ]