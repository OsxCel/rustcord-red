FROM rust:bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim

RUN groupadd --system --gid 10001 rustcord \
    && useradd --system --uid 10001 --gid rustcord --home-dir /app rustcord \
    && mkdir -p /app/Client /data \
    && chown -R rustcord:rustcord /app /data

WORKDIR /app
COPY --from=builder --chown=rustcord:rustcord /build/target/release/MortexApp_backend /usr/local/bin/rustcord
COPY --chown=rustcord:rustcord Client/ /app/Client/

COPY --chown=rustcord:rustcord entrypoint.sh /app/entrypoint.sh

RUN apt-get update && apt-get install -y dos2unix \
    && dos2unix /app/entrypoint.sh \
    && apt-get purge -y dos2unix \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/* \
    && chmod +x /app/entrypoint.sh

ENV PORT=3000
ENV APP_DATA_FILE=/data/app-data.json
EXPOSE 3000
USER rustcord

CMD ["/app/entrypoint.sh"]
