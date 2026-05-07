FROM rust:1.80-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY sdk/ ./sdk/
COPY provider-core/ ./provider-core/
COPY schemas/ ./schemas/
COPY modules/ ./modules/
COPY tools/ ./tools/
COPY kernel/ ./kernel/
COPY adapter/ ./adapter/
COPY config/ ./config/
COPY examples/ ./examples/
COPY providers/ ./providers/
COPY tests/ ./tests/

RUN cargo build --release -p wireframe-ai-tenant-core

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/wireframe-ai-tenant-core /usr/local/bin/

ENV WIREFRAME_AI_NATS_URL=nats://nats:4222
ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/wireframe-ai-tenant-core"]
