# Build stage
FROM rust:1.88-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace manifests first for layer caching
COPY Cargo.toml Cargo.lock* ./
COPY api/Cargo.toml api/
COPY entity/Cargo.toml entity/
COPY migration/Cargo.toml migration/

# Create dummy src files to cache dependencies
RUN mkdir -p api/src entity/src migration/src && \
    echo 'fn main() {}' > api/src/main.rs && \
    echo '' > entity/src/lib.rs && \
    echo 'pub use sea_orm_migration::prelude::*; pub struct Migrator; #[async_trait::async_trait] impl sea_orm_migration::MigratorTrait for Migrator { fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> { vec![] } }' > migration/src/lib.rs && \
    echo '#[tokio::main] async fn main() { sea_orm_migration::cli::run_cli(migration::Migrator).await; }' > migration/src/main.rs

RUN cargo build --release 2>/dev/null || true

# Copy real source
COPY api/src api/src
COPY entity/src entity/src
COPY migration/src migration/src

# Force rebuild of our crates
RUN touch api/src/main.rs entity/src/lib.rs migration/src/lib.rs && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/api /app/api

EXPOSE 8000

CMD ["/app/api"]
