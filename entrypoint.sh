if [ ! -f /data/app-data.json ]; then
    echo "app-data.json bulunamadı, varsayılan şablon oluşturuluyor..."
    cat << 'EOF' > /data/app-data.json
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
EOF
    echo "app-data.json başarıyla oluşturuldu!"
fi

exec /usr/local/bin/rustcord
