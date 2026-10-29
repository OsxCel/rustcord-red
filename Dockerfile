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

# --- ENTRYPOINT DOSYASINI DOĞRUDAN DOCKER İÇİNDE ÜRETİYORUZ ---
RUN cat << 'EOF' > /app/entrypoint.sh
#!/bin/bash

if [ -z "${DATABASE_URL:-}" ] && [ ! -f /data/app-data.json ]; then
    echo "app-data.json bulunamadı, varsayılan şablon oluşturuluyor..."
    cat << 'EOF_JSON' > /data/app-data.json
{
  "users": [],
  "friend_requests": [],
  "friendships": [],
  "stories": [],
  "messages": [],
  "direct_messages": [],
  "groups": [],
  "sessions": {},
  "video_collections": []
}
EOF_JSON
    echo "app-data.json başarıyla oluşturuldu!"
fi

exec /usr/local/bin/rustcord
EOF

RUN chmod +x /app/entrypoint.sh && chown rustcord:rustcord /app/entrypoint.sh

ENV PORT=3000
ENV APP_DATA_FILE=/data/app-data.json
EXPOSE 3000
USER rustcord

CMD ["/app/entrypoint.sh"]
