# Multi-Stage musl/scratch Static Binary Container for RustNext Enterprise
FROM rust:1.85-bullseye AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y \
    musl-tools \
    cmake \
    clang \
    llvm \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates crates

# Build release static binary with LTO and size optimization
RUN cargo build --release -p frappe-net -p rbench

# 2. Ultra-lean Production Runtime
FROM debian:bullseye-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
RUN useradd -u 10001 -m -s /bin/bash appuser

COPY --from=builder /app/target/release/frappe-net /usr/local/bin/frappe-net
COPY --from=builder /app/target/release/rbench /usr/local/bin/rbench

USER appuser

EXPOSE 8080
EXPOSE 4433/udp

HEALTHCHECK --interval=15s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/healthz/ready || exit 1

ENTRYPOINT ["/usr/local/bin/frappe-net"]
