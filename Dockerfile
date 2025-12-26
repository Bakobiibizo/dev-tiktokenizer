# Tiktokenizer Proxy
# Pure Rust service using tiktoken_rs - no backend subprocess needed

FROM rust:1.83-bookworm AS builder

WORKDIR /build
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Rust binary
COPY --from=builder /build/target/release/dev-tiktokenizer /app/proxy

# Environment defaults
ENV API_HOST=0.0.0.0
ENV API_PORT=7105
ENV DEFAULT_TOKENIZER_MODEL=cl100k_base
ENV DEFAULT_EMBEDDING_MODEL=text-embedding-ada-002
ENV PRELOAD=true

EXPOSE 7105

CMD ["/app/proxy"]
