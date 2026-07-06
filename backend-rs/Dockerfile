# syntax=docker/dockerfile:1

# ---------- builder ----------
# buildpack-deps base (in the rust image) provides gcc, which libwebp-sys needs
# to compile the bundled libwebp from source.
FROM rust:1-bookworm AS builder
WORKDIR /app

# Cache dependency compilation: build against a dummy main first, so a source
# change doesn't re-download/re-compile the (heavy) dependency tree.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
# Bust the cached dummy binary, then build the real one.
RUN touch src/main.rs && cargo build --release

# ---------- runtime ----------
FROM debian:bookworm-slim AS runtime
# ca-certificates for outbound TLS (CivitAI / Gelbooru via reqwest+rustls).
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/promptbox /usr/local/bin/promptbox

# Run as a non-root user; import/storage are bind-mounted at runtime.
RUN useradd -m appuser
USER appuser

ENV LISTEN_ADDR=0.0.0.0:8000
EXPOSE 8000

CMD ["promptbox"]
