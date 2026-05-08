FROM rust:1.87-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY examples ./examples

RUN cargo build --release --bin smoke_instance_principal

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/smoke_instance_principal /usr/local/bin/smoke_instance_principal

ENV OCI_AUTH_MODE=instance_principal
ENV OCI_SMOKE_KEEP_ALIVE=true

CMD ["smoke_instance_principal"]
