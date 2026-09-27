FROM rust:1.85-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

ENV CARGO_BUILD_JOBS=1
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    postgresql-client \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Download pre-compiled sqlx-cli
RUN curl -L https://github.com/launchbadge/sqlx-cli/releases/download/v0.7.4/sqlx-v0.7.4-x86_64-unknown-linux-musl.tar.gz | tar -xz -C /usr/local/bin

COPY --from=builder /app/target/release/libro_htmx /app/server
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static
COPY --from=builder /app/uploads /app/uploads
COPY migrations /app/migrations
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

ENV PORT=3000
EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
