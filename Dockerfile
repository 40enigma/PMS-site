# ── Stage 1: Build binary ──
FROM rust:1-bookworm AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest and dependencies
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Build optimized release binary
RUN cargo build --release

# ── Stage 2: Runtime image ──
FROM debian:bookworm-slim AS runner

# Install ca-certificates and sqlite runtime library
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root unprivileged service user
RUN groupadd -g 10001 pms && \
    useradd -u 10001 -g pms -m -s /bin/bash pmsuser

WORKDIR /app

# Prepare directories with proper non-root ownership
RUN mkdir -p /app/data /app/uploads && \
    chown -R pmsuser:pms /app

# Copy release binary and runtime assets
COPY --from=builder /usr/src/app/target/release/pms-platform /app/pms-platform
COPY --chown=pmsuser:pms frontend /app/frontend
COPY --chown=pmsuser:pms migrations /app/migrations

USER pmsuser:pms

ENV HOST=0.0.0.0 \
    PORT=3000 \
    RUST_ENV=production \
    DATABASE_URL=sqlite:///app/data/pms.db \
    UPLOAD_DIR=/app/uploads

EXPOSE 3000

CMD ["/app/pms-platform"]

