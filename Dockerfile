# 1. Build environment
FROM rust:1.85-bullseye AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    cmake \
    clang \
    llvm \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates crates

# Build the frappe-net binary
RUN cargo build --release -p frappe-net

# 2. Runtime environment
FROM debian:bullseye-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
RUN useradd -u 10001 -m -s /bin/bash appuser

COPY --from=builder /app/target/release/frappe-net /usr/local/bin/frappe-net
USER appuser

EXPOSE 8080
EXPOSE 4433/udp

HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["frappe-net"]
