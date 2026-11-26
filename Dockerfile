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

# Install runtime CA certificates
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
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
    DATABASE_URL=sqlite:///app/data/pms.db?mode=rwc \
    UPLOAD_DIR=/app/uploads

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["bash", "-c", "exec 3<>/dev/tcp/127.0.0.1/${PORT:-3000} && printf 'GET /healthz HTTP/1.0\\r\\nHost: localhost\\r\\n\\r\\n' >&3 && head -n 1 <&3 | grep -q '200 OK'"]

CMD ["/app/pms-platform"]

