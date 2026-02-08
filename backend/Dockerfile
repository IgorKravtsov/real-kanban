# ---- Build stage ----
FROM rust:1.85-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy schema and create a compile-time DB for sqlx macro verification
COPY schema.sql ./
RUN sqlite3 /tmp/build.db < schema.sql

# Set DATABASE_URL at build time so sqlx macros can verify queries
ENV DATABASE_URL=sqlite:///tmp/build.db

# Cache dependencies — copy manifests first
COPY Cargo.toml Cargo.lock ./

# Create a dummy main/lib to build deps only
RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    echo '' > src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Copy real source and rebuild
COPY src/ src/

# Touch main.rs so cargo doesn't skip rebuild
RUN touch src/main.rs src/lib.rs && \
    cargo build --release && \
    strip target/release/kanban-board

# ---- Runtime stage ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1000 appuser && \
    useradd --uid 1000 --gid appuser --shell /bin/sh --create-home appuser

WORKDIR /app

COPY --from=builder /app/target/release/kanban-board /app/kanban-board

RUN mkdir -p /app/data && chown -R appuser:appuser /app

USER appuser

ENV DATABASE_URL=sqlite:///app/data/kanban.db \
    PORT=30100 \
    RUST_LOG=info

EXPOSE 30100

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:30100/health || exit 1

ENTRYPOINT ["/app/kanban-board"]
