FROM rust:1.95 AS builder 
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates
COPY api/ ./api
COPY cli/ ./cli

RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
  cargo build --bin rebound --release && \
  cp -v target/release/rebound service

FROM debian

RUN apt-get update && \
  apt-get install openssl ca-certificates -y

COPY --from=builder /app/service /service

ENTRYPOINT ["/service"]
